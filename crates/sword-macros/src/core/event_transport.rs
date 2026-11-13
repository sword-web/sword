use proc_macro2::TokenStream;
use quote::quote;

use crate::shared::{Error, Result, paths::*};

/// Parsed value of the `transport = EventTransport::...` controller argument.
///
/// This lives in `core` (not in the feature-gated `events` module) because
/// `ControllerKind::Events` references it and the controller macro must always
/// be able to report a clear error when the `events` feature is disabled.
pub enum EventTransportKind {
    Memory,
    Nats,
}

impl EventTransportKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Memory => "Memory",
            Self::Nats => "Nats",
        }
    }

    pub fn from_name(name: &str) -> Result<Self> {
        match name {
            "Memory" => Ok(Self::Memory),
            "Nats" => Ok(Self::Nats),
            _ => Err(Error::custom(format!(
                "Invalid event transport `{name}`. Expected 'Memory' or 'Nats'."
            ))),
        }
    }

    pub fn as_tokens(&self) -> TokenStream {
        match self {
            Self::Memory => quote! { #EventTransport::Memory },
            Self::Nats => quote! { #EventTransport::Nats },
        }
    }

    pub fn from_path(path: &syn::Path) -> Result<Self> {
        let name = path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();

        match name.as_str() {
            "Memory" => Ok(Self::Memory),
            "Nats" => Ok(Self::Nats),
            _ => Err(Error::custom(format!(
                "Invalid event transport type: {name}. Expected 'Memory' or 'Nats'."
            ))),
        }
    }
}
