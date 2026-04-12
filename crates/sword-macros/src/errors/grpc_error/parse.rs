use syn::{Attribute, Error, Ident, LitStr, Token, meta::ParseNestedMeta, spanned::Spanned};

use crate::errors::MessageValue;

#[derive(Debug, Clone, Default)]
pub struct GrpcErrorConfig {
    pub transparent: bool,
    pub code: Option<String>,
    pub message: Option<MessageValue>,
    pub tracing_level: Option<String>,
}

pub fn parse_enum_grpc_error_config(attrs: &[Attribute]) -> syn::Result<GrpcErrorConfig> {
    let mut config = GrpcErrorConfig::default();

    for attr in attrs
        .iter()
        .filter(|attr| attr.path().is_ident("grpc_error"))
    {
        config.parse_grpc_attr(attr, "grpc_error")?;
    }

    config.validate_container()?;
    Ok(config)
}

pub fn parse_variant_grpc_error_config(
    ident: &Ident,
    attrs: &[Attribute],
) -> syn::Result<GrpcErrorConfig> {
    let mut config = GrpcErrorConfig::default();

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("grpc")) {
        config.parse_grpc_attr(attr, "grpc")?;
    }

    for attr in attrs.iter().filter(|attr| attr.path().is_ident("tracing")) {
        config.parse_tracing_attr(attr)?;
    }

    config.validate_variant(ident)?;
    Ok(config)
}

impl GrpcErrorConfig {
    pub fn merged(self, defaults: &GrpcErrorConfig) -> Self {
        Self {
            transparent: self.transparent || defaults.transparent,
            code: self.code.or_else(|| defaults.code.clone()),
            message: self.message.or_else(|| defaults.message.clone()),
            tracing_level: self
                .tracing_level
                .or_else(|| defaults.tracing_level.clone()),
        }
    }

    pub fn default_code(&self) -> &str {
        self.code.as_deref().unwrap_or("unknown")
    }

    fn validate_container(&self) -> syn::Result<()> {
        if self.transparent {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "`transparent` is only valid inside #[grpc(...)] on enum variants",
            ));
        }

        Ok(())
    }

    fn validate_variant(&self, ident: &Ident) -> syn::Result<()> {
        if self.transparent
            && (self.code.is_some() || self.message.is_some() || self.tracing_level.is_some())
        {
            return Err(Error::new_spanned(
                ident,
                "`transparent` cannot be combined with `code`, `message`, or `tracing`",
            ));
        }

        Ok(())
    }

    fn parse_grpc_attr(&mut self, attr: &Attribute, attr_name: &str) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

            match ident.to_string().as_str() {
                "transparent" => {
                    if self.transparent {
                        return Err(Error::new(
                            ident.span(),
                            "duplicate `transparent` attribute",
                        ));
                    }

                    self.transparent = true;
                    Ok(())
                }
                "code" => self.set_code(ident, &meta),
                "message" => self.set_message(ident, &meta),
                "tracing" => self.set_tracing_level(ident, &meta),
                other => Err(Error::new(
                    ident.span(),
                    format!("unknown attribute `{other}` in #[{attr_name}(...)]"),
                )),
            }
        })
    }

    fn parse_tracing_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

            self.set_tracing_level_value(ident, ident.to_string())
        })
    }

    fn set_code(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if self.code.is_some() {
            return Err(Error::new(ident.span(), "duplicate `code` attribute"));
        }

        self.code = Some(parse_code_value(ident, meta)?);
        Ok(())
    }

    fn set_message(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if self.message.is_some() {
            return Err(Error::new(ident.span(), "duplicate `message` attribute"));
        }

        self.message = Some(parse_message_value(ident, meta)?);
        Ok(())
    }

    fn set_tracing_level(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if !meta.input.peek(Token![=]) {
            return Err(Error::new(ident.span(), "expected '=' after 'tracing'"));
        }

        meta.input.parse::<Token![=]>()?;

        if let Ok(level) = meta.input.parse::<Ident>() {
            return self.set_tracing_level_value(&level, level.to_string());
        }

        if let Ok(level) = meta.input.parse::<LitStr>() {
            return self.set_tracing_level_value(ident, level.value());
        }

        Err(Error::new(
            ident.span(),
            "expected tracing level identifier or string literal",
        ))
    }

    fn set_tracing_level_value(&mut self, ident: &Ident, level: String) -> syn::Result<()> {
        if self.tracing_level.is_some() {
            return Err(Error::new(ident.span(), "duplicate `tracing` attribute"));
        }

        match level.as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {
                self.tracing_level = Some(level);
                Ok(())
            }
            _ => Err(Error::new(
                ident.span(),
                "invalid tracing level, expected one of: trace, debug, info, warn, error",
            )),
        }
    }
}

fn parse_code_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<String> {
    if !meta.input.peek(Token![=]) {
        return Err(Error::new(ident.span(), "expected '=' after 'code'"));
    }

    meta.input.parse::<Token![=]>()?;
    Ok(meta.input.parse::<LitStr>()?.value())
}

fn parse_message_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<MessageValue> {
    if !meta.input.peek(Token![=]) {
        return Err(Error::new(ident.span(), "expected '=' after 'message'"));
    }

    meta.input.parse::<Token![=]>()?;

    if let Ok(lit) = meta.input.parse::<LitStr>() {
        let value = lit.value();

        if value.contains('{') || value.contains('}') {
            return Err(Error::new(
                lit.span(),
                "message string interpolation is not supported; build the final client message before constructing the error",
            ));
        }

        return Ok(MessageValue::Static(value));
    }

    if let Ok(field) = meta.input.parse::<Ident>() {
        return Ok(MessageValue::Field(field.to_string()));
    }

    Err(Error::new(
        ident.span(),
        "expected string literal or field identifier",
    ))
}
