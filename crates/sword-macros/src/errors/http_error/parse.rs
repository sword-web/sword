use axum::http::StatusCode;
use syn::{
    Attribute, Error, Ident, LitInt, LitStr, Token, meta::ParseNestedMeta, spanned::Spanned,
};

#[derive(Debug, Clone)]
pub enum MessageValue {
    Static(String),
    Field(String),
}

#[derive(Default)]
pub struct HttpErrorConfig {
    /// Delegate to inner type's `From<T> for Json`
    pub transparent: bool,
    /// HTTP status code
    pub code: Option<StatusCode>,
    /// Custom message (static string or field reference)
    pub message: Option<MessageValue>,
    /// Named field to include as "error" in response
    pub error_field: Option<String>,
    /// Named field to include as "errors" in response
    pub errors_field: Option<String>,

    /// Optional tracing level
    pub tracing_level: Option<String>,
}

impl HttpErrorConfig {
    pub fn from_attrs(ident: &Ident, attrs: &[Attribute]) -> syn::Result<Self> {
        let mut config = Self::default();

        for attr in attrs.iter().filter(|a| a.path().is_ident("http")) {
            config.parse_http_attr(attr)?;
        }

        for attr in attrs.iter().filter(|a| a.path().is_ident("tracing")) {
            attr.parse_nested_meta(|meta| {
                let level_ident = meta
                    .path
                    .get_ident()
                    .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

                let level_str = level_ident.to_string();

                match level_str.as_str() {
                    "debug" | "info" | "warn" | "error" | "trace" => {
                        config.tracing_level = Some(level_str);
                    }
                    _ => {
                        return Err(Error::new(
                            level_ident.span(),
                            "invalid tracing level, expected one of: debug, info, warn, error",
                        ));
                    }
                }

                Ok(())
            })?;
        }

        config.validate(ident)?;

        Ok(config)
    }

    fn parse_http_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
        attr.parse_nested_meta(|meta| {
            let ident = meta
                .path
                .get_ident()
                .ok_or_else(|| Error::new(meta.path.span(), "expected identifier"))?;

            match ident.to_string().as_str() {
                "transparent" => {
                    self.transparent = true;
                }
                "code" => {
                    self.code = Some(Self::parse_status_code_value(ident, &meta)?);
                }
                "message" => {
                    self.message = Some(Self::parse_message_value(ident, &meta)?);
                }
                "error" => {
                    let field = meta.value()?.parse::<Ident>()?;
                    self.error_field = Some(field.to_string());
                }
                "errors" => {
                    let field = meta.value()?.parse::<Ident>()?;
                    self.errors_field = Some(field.to_string());
                }
                other => {
                    return Err(Error::new(
                        ident.span(),
                        format!("unknown attribute `{other}`"),
                    ));
                }
            }
            Ok(())
        })
    }

    fn validate(&self, ident: &Ident) -> syn::Result<()> {
        if !self.transparent && self.code.is_none() {
            return Err(Error::new_spanned(
                ident,
                "missing `transparent` or `code` in #[http(...)]",
            ));
        }

        if self.transparent && self.code.is_some() {
            return Err(Error::new_spanned(
                ident,
                "cannot use both `transparent` and `code`",
            ));
        }

        if self.transparent
            && (self.message.is_some()
                || self.error_field.is_some()
                || self.errors_field.is_some()
                || self.tracing_level.is_some())
        {
            return Err(Error::new_spanned(
                ident,
                "`message`, `error`, `errors`, and `tracing` are not valid with `transparent`",
            ));
        }

        Ok(())
    }

    pub fn message(&self) -> Option<MessageValue> {
        self.message.clone()
    }

    pub fn default_message(&self) -> String {
        self.code
            .as_ref()
            .map(|c| c.canonical_reason().unwrap_or("Unknown Error"))
            .unwrap_or("Unknown Error")
            .to_string()
    }

    fn parse_message_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<MessageValue> {
        if !meta.input.peek(Token![=]) {
            return Err(Error::new(ident.span(), "expected '=' after 'message'"));
        }

        meta.input.parse::<Token![=]>()?;

        if let Ok(lit) = meta.input.parse::<LitStr>() {
            return Ok(MessageValue::Static(lit.value()));
        }

        if let Ok(field) = meta.input.parse::<Ident>() {
            return Ok(MessageValue::Field(field.to_string()));
        }

        Err(Error::new(
            ident.span(),
            "expected string literal or identifier",
        ))
    }

    fn parse_status_code_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<StatusCode> {
        if !meta.input.peek(Token![=]) {
            return Err(Error::new(ident.span(), "expected '=' after 'code'"));
        }

        meta.input.parse::<Token![=]>()?;

        let lit = meta.input.parse::<LitInt>()?;
        let code = lit.base10_parse::<u16>()?;

        let status = StatusCode::from_u16(code)
            .map_err(|_| Error::new(lit.span(), "invalid HTTP status code"))?;

        Ok(status)
    }
}
