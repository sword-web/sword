use darling::{FromDeriveInput, FromVariant, util::Flag};
use syn::{DeriveInput, Variant};

use crate::shared::Result;
use crate::shared::errors::{
    FieldRef, MessageValue, TracingLevel, read_tracing_attr, validate_transparent_container,
    validate_transparent_variant,
};

/// Parsed `#[http_error(...)]` (container defaults) or `#[http(...)]` (variant).
#[derive(FromDeriveInput, FromVariant)]
#[darling(attributes(http_error, http))]
pub struct HttpErrorArgs {
    pub transparent: Flag,
    pub code: Option<u16>,
    pub message: Option<MessageValue>,
    pub error: Option<FieldRef>,
    pub errors: Option<FieldRef>,
    pub tracing: Option<TracingLevel>,
}

impl HttpErrorArgs {
    pub fn container(input: &DeriveInput) -> Result<Self> {
        let args = Self::from_derive_input(input)?;
        validate_transparent_container(args.transparent.is_present(), "http")?;
        Ok(args)
    }

    pub fn variant(variant: &Variant) -> Result<Self> {
        let mut args = Self::from_variant(variant)?;

        if args.tracing.is_none() {
            args.tracing = read_tracing_attr(&variant.attrs)?;
        }

        let has_conflict = args.code.is_some()
            || args.message.is_some()
            || args.error.is_some()
            || args.errors.is_some()
            || args.tracing.is_some();

        validate_transparent_variant(
            args.transparent.is_present(),
            has_conflict,
            &variant.ident,
            "`code`, `message`, `error`, `errors`, or `tracing`",
        )?;

        Ok(args)
    }

    pub fn merged(self, defaults: &Self) -> Self {
        Self {
            transparent: self.transparent,
            code: self.code.or(defaults.code),
            message: self.message.or_else(|| defaults.message.clone()),
            error: self.error.or_else(|| defaults.error.clone()),
            errors: self.errors.or_else(|| defaults.errors.clone()),
            tracing: self.tracing.or(defaults.tracing),
        }
    }

    pub fn default_message(&self) -> String {
        self.code
            .and_then(default_reason)
            .unwrap_or("Unknown Error")
            .to_string()
    }
}

fn default_reason(code: u16) -> Option<&'static str> {
    Some(match code {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        405 => "Method Not Allowed",
        409 => "Conflict",
        422 => "Unprocessable Entity",
        500 => "Internal Server Error",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => return None,
    })
}
