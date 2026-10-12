use darling::{FromDeriveInput, FromVariant, util::Flag};
use syn::{DeriveInput, Variant};

use crate::shared::Result;
use crate::shared::errors::{
    MessageValue, TracingLevel, read_tracing_attr, validate_transparent_container,
    validate_transparent_variant,
};

/// Parsed `#[grpc_error(...)]` (container defaults) or `#[grpc(...)]` (variant).
#[derive(FromDeriveInput, FromVariant)]
#[darling(attributes(grpc_error, grpc))]
pub struct GrpcErrorArgs {
    pub transparent: Flag,
    pub code: Option<String>,
    pub message: Option<MessageValue>,
    pub tracing: Option<TracingLevel>,
}

impl GrpcErrorArgs {
    pub fn container(input: &DeriveInput) -> Result<Self> {
        let args = Self::from_derive_input(input)?;
        validate_transparent_container(args.transparent.is_present(), "grpc")?;
        Ok(args)
    }

    pub fn variant(variant: &Variant) -> Result<Self> {
        let mut args = Self::from_variant(variant)?;

        if args.tracing.is_none() {
            args.tracing = read_tracing_attr(&variant.attrs)?;
        }

        let has_conflict = args.code.is_some() || args.message.is_some() || args.tracing.is_some();

        validate_transparent_variant(
            args.transparent.is_present(),
            has_conflict,
            &variant.ident,
            "`code`, `message`, or `tracing`",
        )?;

        Ok(args)
    }

    pub fn merged(self, defaults: &Self) -> Self {
        Self {
            transparent: self.transparent,
            code: self.code.or_else(|| defaults.code.clone()),
            message: self.message.or_else(|| defaults.message.clone()),
            tracing: self.tracing.or(defaults.tracing),
        }
    }

    pub fn default_code(&self) -> &str {
        self.code.as_deref().unwrap_or("unknown")
    }
}
