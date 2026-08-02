use axum::http::StatusCode;
use syn::{Attribute, Error, Ident, LitInt, Token, meta::ParseNestedMeta, spanned::Spanned};

use crate::errors::MessageValue;

#[derive(Debug, Clone, Default)]
pub struct HttpErrorConfig {
    pub transparent: bool,
    pub code: Option<StatusCode>,
    pub message: Option<MessageValue>,
    pub error_field: Option<String>,
    pub errors_field: Option<String>,
    pub tracing_level: Option<String>,
}

impl HttpErrorConfig {
    pub fn from_attrs(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut config = HttpErrorConfig::default();

        for attr in attrs
            .iter()
            .filter(|attr| attr.path().is_ident("http_error"))
        {
            config.parse_http_attr(attr)?;
        }

        config.validate_container()?;

        Ok(config)
    }

    pub fn parse_enum_variant_config(ident: &Ident, attrs: &[Attribute]) -> syn::Result<Self> {
        let mut config = HttpErrorConfig::default();

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("http")) {
            config.parse_http_attr(attr)?;
        }

        for attr in attrs.iter().filter(|attr| attr.path().is_ident("tracing")) {
            config.parse_tracing_attr(attr)?;
        }

        config.validate_variant(ident)?;

        Ok(config)
    }

    pub fn merged(self, defaults: &HttpErrorConfig) -> Self {
        Self {
            transparent: self.transparent || defaults.transparent,
            code: self.code.or(defaults.code),
            message: self.message.or_else(|| defaults.message.clone()),
            error_field: self.error_field.or_else(|| defaults.error_field.clone()),
            errors_field: self.errors_field.or_else(|| defaults.errors_field.clone()),
            tracing_level: self
                .tracing_level
                .or_else(|| defaults.tracing_level.clone()),
        }
    }

    pub fn default_message(&self) -> String {
        self.code
            .as_ref()
            .map(|code| code.canonical_reason().unwrap_or("Unknown Error"))
            .unwrap_or("Unknown Error")
            .to_string()
    }

    fn validate_container(&self) -> syn::Result<()> {
        crate::errors::validate_transparent_container(self.transparent, "http")
    }

    fn validate_variant(&self, ident: &Ident) -> syn::Result<()> {
        let has_conflict = self.code.is_some()
            || self.message.is_some()
            || self.error_field.is_some()
            || self.errors_field.is_some()
            || self.tracing_level.is_some();

        crate::errors::validate_transparent_variant(
            self.transparent,
            has_conflict,
            ident,
            "`code`, `message`, `error`, `errors`, or `tracing`",
        )
    }

    fn parse_http_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

            match ident.to_string().as_str() {
                "transparent" => self.set_transparent(ident),
                "code" => self.set_code(ident, &meta),
                "message" => self.set_message(ident, &meta),
                "error" => self.set_error_field(ident, &meta),
                "errors" => self.set_errors_field(ident, &meta),
                "tracing" => self.set_tracing_level(ident, &meta),
                other => Err(Error::new(
                    ident.span(),
                    format!("unknown attribute `{other}` for this context"),
                )),
            }
        })
    }

    fn parse_tracing_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        crate::errors::parse_tracing_attr(&mut self.tracing_level, attr)
    }

    fn set_transparent(&mut self, ident: &Ident) -> syn::Result<()> {
        if self.transparent {
            return Err(Error::new(
                ident.span(),
                "duplicate `transparent` attribute",
            ));
        }

        self.transparent = true;
        Ok(())
    }

    fn set_code(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if self.code.is_some() {
            return Err(Error::new(ident.span(), "duplicate `code` attribute"));
        }

        self.code = Some(parse_status_code_value(ident, meta)?);
        Ok(())
    }

    fn set_message(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if self.message.is_some() {
            return Err(Error::new(ident.span(), "duplicate `message` attribute"));
        }

        self.message = Some(MessageValue::parse(ident, meta)?);
        Ok(())
    }

    fn set_error_field(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if self.error_field.is_some() {
            return Err(Error::new(ident.span(), "duplicate `error` attribute"));
        }

        self.error_field = Some(parse_field_ident(meta)?.to_string());
        Ok(())
    }

    fn set_errors_field(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        if self.errors_field.is_some() {
            return Err(Error::new(ident.span(), "duplicate `errors` attribute"));
        }

        self.errors_field = Some(parse_field_ident(meta)?.to_string());
        Ok(())
    }

    fn set_tracing_level(&mut self, ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<()> {
        crate::errors::set_tracing_level(&mut self.tracing_level, ident, meta)
    }
}

fn parse_field_ident(meta: &ParseNestedMeta) -> syn::Result<Ident> {
    if !meta.input.peek(Token![=]) {
        return Err(Error::new(meta.path.span(), "expected '=' after attribute"));
    }

    meta.input.parse::<Token![=]>()?;
    meta.input.parse::<Ident>()
}

fn parse_status_code_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<StatusCode> {
    if !meta.input.peek(Token![=]) {
        return Err(Error::new(ident.span(), "expected '=' after 'code'"));
    }

    meta.input.parse::<Token![=]>()?;

    let lit = meta.input.parse::<LitInt>()?;
    let code = lit.base10_parse::<u16>()?;

    StatusCode::from_u16(code).map_err(|_| Error::new(lit.span(), "invalid HTTP status code"))
}
