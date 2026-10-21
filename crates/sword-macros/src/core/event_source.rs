use crate::shared::{Error, Result, paths::*};

use proc_macro2::TokenStream;
use quote::quote;

/// Parsed value of the `source = EventSource::...` controller argument.
///
/// This lives in `core` (not in the feature-gated `events` module) because
/// `ControllerKind::Events` references it and the controller macro must always
/// be able to report a clear error when the `events` feature is disabled.
pub enum EventSourceKind {
    Memory,
    Nats,
}

impl EventSourceKind {
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
                "Invalid event source `{name}`. Expected 'Memory' or 'Nats'."
            ))),
        }
    }

    pub fn as_tokens(&self) -> TokenStream {
        match self {
            Self::Memory => quote! { #EventSource::Memory },
            Self::Nats => quote! { #EventSource::Nats },
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
                "Invalid event source type: {name}. Expected 'Memory' or 'Nats'."
            ))),
        }
    }
}
