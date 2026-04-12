use axum::http::StatusCode;
use syn::{
    Attribute, Error, Ident, LitInt, LitStr, Token, meta::ParseNestedMeta, spanned::Spanned,
};

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
        if self.transparent {
            return Err(Error::new(
                proc_macro2::Span::call_site(),
                "`transparent` is only valid inside #[http(...)] on enum variants",
            ));
        }

        Ok(())
    }

    fn validate_variant(&self, ident: &Ident) -> syn::Result<()> {
        if self.transparent
            && (self.code.is_some()
                || self.message.is_some()
                || self.error_field.is_some()
                || self.errors_field.is_some()
                || self.tracing_level.is_some())
        {
            return Err(Error::new_spanned(
                ident,
                "`transparent` cannot be combined with `code`, `message`, `error`, `errors`, or `tracing`",
            ));
        }

        Ok(())
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
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

            self.set_tracing_level_value(ident, ident.to_string())
        })
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

impl MessageValue {
    fn parse(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<MessageValue> {
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
