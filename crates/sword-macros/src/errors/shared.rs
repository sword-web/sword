use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Attribute, Error, Fields, FieldsNamed, Ident, LitStr, Token, meta::ParseNestedMeta,
    spanned::Spanned,
};

use super::{MessageValue, extract_template_fields, format_template};

// ===== Parse helpers =====

pub fn parse_tracing_attr(tracing_level: &mut Option<String>, attr: &Attribute) -> syn::Result<()> {
    attr.parse_nested_meta(|meta| {
        let ident = meta
            .path
            .get_ident()
            .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

        set_tracing_level_value(tracing_level, ident, ident.to_string())
    })
}

pub fn set_tracing_level(
    tracing_level: &mut Option<String>,
    ident: &Ident,
    meta: &ParseNestedMeta,
) -> syn::Result<()> {
    if !meta.input.peek(Token![=]) {
        return Err(Error::new(ident.span(), "expected '=' after 'tracing'"));
    }

    meta.input.parse::<Token![=]>()?;

    if let Ok(level) = meta.input.parse::<Ident>() {
        return set_tracing_level_value(tracing_level, &level, level.to_string());
    }

    if let Ok(level) = meta.input.parse::<LitStr>() {
        return set_tracing_level_value(tracing_level, ident, level.value());
    }

    Err(Error::new(
        ident.span(),
        "expected tracing level identifier or string literal",
    ))
}

pub fn set_tracing_level_value(
    tracing_level: &mut Option<String>,
    ident: &Ident,
    level: String,
) -> syn::Result<()> {
    if tracing_level.is_some() {
        return Err(Error::new(ident.span(), "duplicate `tracing` attribute"));
    }

    match level.as_str() {
        "trace" | "debug" | "info" | "warn" | "error" => {
            *tracing_level = Some(level);
            Ok(())
        }
        _ => Err(Error::new(
            ident.span(),
            "invalid tracing level, expected one of: trace, debug, info, warn, error",
        )),
    }
}

pub fn validate_transparent_container(transparent: bool, attr_name: &str) -> syn::Result<()> {
    if transparent {
        return Err(Error::new(
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
        return Err(Error::new_spanned(
            ident,
            format!("`transparent` cannot be combined with {conflict_desc}"),
        ));
    }

    Ok(())
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
            let (fmt, fields) = format_template(template);
            let field_idents: Vec<_> = fields
                .iter()
                .map(|f| Ident::new(f, proc_macro2::Span::call_site()))
                .collect();
            quote! { format!(#fmt, #(#field_idents),*) }
        }
        None => fallback(),
    }
}

pub fn generate_tracing_stmt(
    variant_name: &Ident,
    tracing_level: &Option<String>,
    fields: &Fields,
    code_key: &Ident,
    code_value: TokenStream,
    title: &str,
) -> TokenStream {
    let Some(level) = tracing_level else {
        return quote! {};
    };

    let tracing_macro = match level.as_str() {
        "trace" => quote! { ::sword::internal::tracing::trace },
        "debug" => quote! { ::sword::internal::tracing::debug },
        "info" => quote! { ::sword::internal::tracing::info },
        "warn" => quote! { ::sword::internal::tracing::warn },
        "error" => quote! { ::sword::internal::tracing::error },
        _ => return quote! {},
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

// ===== Validation helpers =====

/// Transparent variants must have exactly one unnamed field.
pub fn validate_transparent_single_unnamed(
    variant_name: &Ident,
    fields: &Fields,
) -> syn::Result<()> {
    let is_single_unnamed =
        matches!(fields, Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1);

    if !is_single_unnamed {
        return Err(Error::new_spanned(
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
        return Err(Error::new_spanned(
            variant_name,
            "non-transparent tuple variants are only supported with exactly one field",
        ));
    }

    Ok(())
}

/// Checks that `Field` and `Interpolated` message references point to real named fields.
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

/// Checks that a named variant has the fields referenced by `error`/`errors`-style attributes.
#[cfg(feature = "web-controllers")]
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

    Err(Error::new_spanned(
        variant_name,
        format!(
            "`{attr_name} = {field_name}` references a missing field on {enum_name}::{variant_name}`"
        ),
    ))
}
