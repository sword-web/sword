use syn::{Attribute, Error, Ident, LitStr, Token, meta::ParseNestedMeta, spanned::Spanned};

#[derive(Debug, Clone)]
pub enum MessageValue {
    Static(String),
    Field(String),
}

#[derive(Default)]
pub struct GrpcErrorConfig {
    pub transparent: bool,
    pub code: Option<String>,
    pub message: Option<MessageValue>,
    pub tracing_level: Option<String>,
}

impl GrpcErrorConfig {
    pub fn from_attrs(ident: &Ident, attrs: &[Attribute]) -> syn::Result<Self> {
        let mut config = Self::default();

        for attr in attrs.iter().filter(|a| a.path().is_ident("grpc")) {
            config.parse_grpc_attr(attr)?;
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
                            "invalid tracing level, expected one of: debug, info, warn, error, trace",
                        ));
                    }
                }

                Ok(())
            })?;
        }

        config.validate(ident)?;

        Ok(config)
    }

    fn parse_grpc_attr(&mut self, attr: &Attribute) -> syn::Result<()> {
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
                    self.code = Some(Self::parse_code_value(ident, &meta)?);
                }
                "message" => {
                    self.message = Some(Self::parse_message_value(ident, &meta)?);
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

    fn parse_code_value(ident: &Ident, meta: &ParseNestedMeta) -> syn::Result<String> {
        if !meta.input.peek(Token![=]) {
            return Err(Error::new(ident.span(), "expected '=' after 'code'"));
        }

        meta.input.parse::<Token![=]>()?;
        let lit = meta.input.parse::<LitStr>()?;
        Ok(lit.value())
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

    fn validate(&self, ident: &Ident) -> syn::Result<()> {
        if !self.transparent && self.code.is_none() {
            return Err(Error::new_spanned(
                ident,
                "missing `transparent` or `code` in #[grpc(...)]",
            ));
        }

        if self.transparent && self.code.is_some() {
            return Err(Error::new_spanned(
                ident,
                "cannot use both `transparent` and `code`",
            ));
        }

        if self.transparent && (self.message.is_some() || self.tracing_level.is_some()) {
            return Err(Error::new_spanned(
                ident,
                "`message` and `tracing` are not valid with `transparent`",
            ));
        }

        Ok(())
    }

    pub fn message(&self) -> Option<MessageValue> {
        self.message.clone()
    }

    pub fn code(&self) -> Option<&str> {
        self.code.as_deref()
    }
}
