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
        crate::errors::validate_transparent_container(self.transparent, "grpc")
    }

    fn validate_variant(&self, ident: &Ident) -> syn::Result<()> {
        let has_conflict =
            self.code.is_some() || self.message.is_some() || self.tracing_level.is_some();

        crate::errors::validate_transparent_variant(
            self.transparent,
            has_conflict,
            ident,
            "`code`, `message`, or `tracing`",
        )
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
        crate::errors::parse_tracing_attr(&mut self.tracing_level, attr)
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

        self.message = Some(MessageValue::parse(ident, meta)?);
        Ok(())
    }

    fn set_tracing_level(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        crate::errors::set_tracing_level(&mut self.tracing_level, ident, meta)
    }
}

fn parse_code_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<String> {
    if !meta.input.peek(Token![=]) {
        return Err(Error::new(ident.span(), "expected '=' after 'code'"));
    }

    meta.input.parse::<Token![=]>()?;
    Ok(meta.input.parse::<LitStr>()?.value())
}
