use darling::{Error, FromMeta};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Expr, ExprLit, Fields, FieldsNamed, Ident, Lit, LitStr, Path, spanned::Spanned,
};

// ===== Shared value types (used by HttpError and GrpcError) =====

/// A client-facing message: a fixed string, a field reference, or an
/// interpolated template with `{field}` placeholders.
#[derive(Debug, Clone)]
pub enum MessageValue {
    Static(String),
    Field(String),
    Interpolated(String),
}

impl FromMeta for MessageValue {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => {
                let value = s.value();

                if value.contains('{') || value.contains('}') {
                    Ok(Self::Interpolated(value))
                } else {
                    Ok(Self::Static(value))
                }
            }
            Expr::Path(path) => Ok(Self::Field(path_ident(&path.path)?)),
            _ => Err(Error::unexpected_expr_type(expr)),
        }
    }
}

/// A reference to a named field of a variant (`error = detail`).
#[derive(Debug, Clone)]
pub struct FieldRef(pub String);

impl FieldRef {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromMeta for FieldRef {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(path) => Ok(FieldRef(path_ident(&path.path)?)),
            _ => Err(Error::custom("expected a field identifier").with_span(expr)),
        }
    }
}

/// A tracing level, from either `tracing = error` or `#[tracing(error)]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracingLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl TracingLevel {
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "trace" => Some(Self::Trace),
            "debug" => Some(Self::Debug),
            "info" => Some(Self::Info),
            "warn" => Some(Self::Warn),
            "error" => Some(Self::Error),
            _ => None,
        }
    }
}

impl FromMeta for TracingLevel {
    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        match expr {
            Expr::Path(path) => {
                let name = path_ident(&path.path)?;
                Self::parse(&name).ok_or_else(|| invalid_tracing_level(path))
            }
            Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }) => Self::parse(&s.value()).ok_or_else(|| invalid_tracing_level(s)),
            _ => Err(Error::unexpected_expr_type(expr)),
        }
    }
}

/// Reads the standalone `#[tracing(level)]` shorthand (darling does not see it).
pub fn read_tracing_attr(attrs: &[Attribute]) -> crate::shared::Result<Option<TracingLevel>> {
    let mut level = None;

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("tracing")) {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| syn::Error::new(meta.path.span(), "expected a tracing level"))?;

            let parsed = TracingLevel::parse(&ident.to_string()).ok_or_else(|| {
                syn::Error::new(
                    ident.span(),
                    "invalid tracing level, expected one of: trace, debug, info, warn, error",
                )
            })?;

            if level.is_some() {
                return Err(syn::Error::new(
                    ident.span(),
                    "duplicate `tracing` attribute",
                ));
            }

            level = Some(parsed);

            Ok(())
        })?;
    }

    Ok(level)
}

fn path_ident(path: &Path) -> darling::Result<String> {
    path.get_ident()
        .map(|ident| ident.to_string())
        .ok_or_else(|| Error::custom("expected a field identifier").with_span(path))
}

fn invalid_tracing_level(spanned: &impl Spanned) -> Error {
    Error::custom("invalid tracing level, expected one of: trace, debug, info, warn, error")
        .with_span(spanned)
}

// ===== Template helpers =====

/// Extracts `{field_name}` references from an interpolated template, skipping
/// `{{`/`}}` escapes.
pub fn extract_template_fields(template: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '{' {
            continue;
        }

        if chars.peek() == Some(&'{') {
            chars.next();
            continue;
        }

        let mut name = String::new();
        while let Some(&c) = chars.peek() {
            if c == '}' {
                break;
            }
            name.push(c);
            chars.next();
        }

        if !name.is_empty() {
            fields.push(name);
        }

        chars.next();
    }

    fields
}

// ===== Codegen helpers =====

pub fn generate_pattern(enum_name: &Ident, variant_name: &Ident, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let field_names: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
            quote! { #enum_name::#variant_name { #(#field_names),* } }
        }
        Fields::Unnamed(_) => quote! { #enum_name::#variant_name(_inner) },
        Fields::Unit => quote! { #enum_name::#variant_name },
    }
}

pub fn generate_message_expr(
    message: &Option<MessageValue>,
    fallback: impl FnOnce() -> TokenStream,
) -> TokenStream {
    match message {
        Some(MessageValue::Static(message)) => quote! { #message },
        Some(MessageValue::Field(field_name)) => {
            let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
            quote! { format!("{}", #field_ident) }
        }
        Some(MessageValue::Interpolated(template)) => {
            // Implicit capture: `format!("... {field} ...")` picks `field` from scope.
            let literal = LitStr::new(template, proc_macro2::Span::call_site());
            quote! { format!(#literal) }
        }
        None => fallback(),
    }
}

pub fn generate_tracing_stmt(
    variant_name: &Ident,
    level: Option<TracingLevel>,
    fields: &Fields,
    code_key: &Ident,
    code_value: TokenStream,
    title: &str,
) -> TokenStream {
    let Some(level) = level else {
        return quote! {};
    };

    let tracing = crate::shared::paths::tracing;
    let tracing_macro = match level {
        TracingLevel::Trace => quote! { #tracing::trace },
        TracingLevel::Debug => quote! { #tracing::debug },
        TracingLevel::Info => quote! { #tracing::info },
        TracingLevel::Warn => quote! { #tracing::warn },
        TracingLevel::Error => quote! { #tracing::error },
    };

    let variant_str = variant_name.to_string();

    match fields {
        Fields::Named(named) => {
            let field_logs = named.named.iter().map(|field| {
                let field_name = field.ident.as_ref().unwrap();
                quote! { #field_name = ?#field_name, }
            });

            quote! {
                #tracing_macro!(
                    error = %__sword_internal_error,
                    error_type = #variant_str,
                    #code_key = #code_value,
                    #(#field_logs)*
                    #title
                );
            }
        }
        Fields::Unnamed(_) => {
            quote! {
                #tracing_macro!(
                    error = %__sword_internal_error,
                    inner = ?_inner,
                    error_type = #variant_str,
                    #code_key = #code_value,
                    #title
                );
            }
        }
        Fields::Unit => {
            quote! {
                #tracing_macro!(
                    error = %__sword_internal_error,
                    error_type = #variant_str,
                    #code_key = #code_value,
                    #title
                );
            }
        }
    }
}

/// Default tracing level for a status/grpc code, matching the access logger:
/// 2xx/3xx → info, 4xx → warn, else → error.
pub fn default_tracing_level(code: u16) -> TracingLevel {
    match code {
        200..=399 => TracingLevel::Info,
        400..=499 => TracingLevel::Warn,
        _ => TracingLevel::Error,
    }
}

// ===== Validation helpers =====

pub fn validate_transparent_container(transparent: bool, attr_name: &str) -> syn::Result<()> {
    if transparent {
        return Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            format!("`transparent` is only valid inside #[{attr_name}(...)] on enum variants"),
        ));
    }

    Ok(())
}

pub fn validate_transparent_variant(
    transparent: bool,
    has_conflict: bool,
    ident: &Ident,
    conflict_desc: &str,
) -> syn::Result<()> {
    if transparent && has_conflict {
        return Err(syn::Error::new_spanned(
            ident,
            format!("`transparent` cannot be combined with {conflict_desc}"),
        ));
    }

    Ok(())
}

/// Transparent variants must have exactly one unnamed field.
pub fn validate_transparent_single_unnamed(
    variant_name: &Ident,
    fields: &Fields,
) -> syn::Result<()> {
    let is_single_unnamed =
        matches!(fields, Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1);

    if !is_single_unnamed {
        return Err(syn::Error::new_spanned(
            variant_name,
            "transparent variants must have exactly one unnamed field",
        ));
    }

    Ok(())
}

/// Non-transparent tuple variants are only supported with exactly one field.
pub fn validate_single_unnamed(
    variant_name: &Ident,
    unnamed: &syn::FieldsUnnamed,
) -> syn::Result<()> {
    if unnamed.unnamed.len() != 1 {
        return Err(syn::Error::new_spanned(
            variant_name,
            "non-transparent tuple variants are only supported with exactly one field",
        ));
    }

    Ok(())
}

/// Checks that `Field` and `Interpolated` message references point to real fields.
pub fn validate_message_references(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &FieldsNamed,
    message: &Option<MessageValue>,
) -> syn::Result<()> {
    if let Some(MessageValue::Field(field_name)) = message {
        ensure_named_field_exists(enum_name, variant_name, fields, field_name, "message")?;
    }

    if let Some(MessageValue::Interpolated(template)) = message {
        for field in extract_template_fields(template) {
            ensure_named_field_exists(enum_name, variant_name, fields, &field, "message")?;
        }
    }

    Ok(())
}

/// Checks that a named variant has the fields referenced by `error`/`errors`.
pub fn validate_named_field_refs(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &FieldsNamed,
    extra_refs: &[(&str, &str)],
) -> syn::Result<()> {
    for (attr_name, field_name) in extra_refs {
        ensure_named_field_exists(enum_name, variant_name, fields, field_name, attr_name)?;
    }

    Ok(())
}

fn ensure_named_field_exists(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &FieldsNamed,
    field_name: &str,
    attr_name: &str,
) -> syn::Result<()> {
    let exists = fields
        .named
        .iter()
        .filter_map(|field| field.ident.as_ref())
        .any(|ident| ident == field_name);

    if exists {
        return Ok(());
    }

    Err(syn::Error::new_spanned(
        variant_name,
        format!(
            "`{attr_name} = {field_name}` references a missing field on {enum_name}::{variant_name}`"
        ),
    ))
}
