use crate::shared::{Error, Result, cmeta::CMetaStack, paths::*};

use heck::ToPascalCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Ident, ItemFn, LitStr};

/// Parsed data from an `#[on("...")]` handler attribute.
pub struct ParsedOnHandler {
    /// The original function, kept as-is.
    pub function: ItemFn,

    /// The handler method name.
    pub fn_name: Ident,

    /// The enclosing Socket.IO controller type.
    pub controller_ty: Ident,

    /// The controller namespace from `CMetaStack`.
    pub namespace: String,

    /// The raw event name literal.
    pub event_name: LitStr,

    /// The matching `SocketEventKind` tokens.
    pub event_kind: TokenStream2,

    /// Name of the generated `const` registrar.
    pub registration_name: Ident,

    /// Name of the generated helper struct holding the handler functions.
    pub handler_struct_name: Ident,
}

impl ParsedOnHandler {
    pub fn parse(attr: TokenStream, item: TokenStream) -> Result<Self> {
        let event_name = syn::parse::<LitStr>(attr)?;

        if event_name.value().is_empty() {
            return Err(Error::custom(
                "event name must not be empty; specify the event, e.g. #[on(\"message\")]",
            )
            .with_span(&event_name));
        }

        let function = syn::parse::<ItemFn>(item)?;
        let fn_name = function.sig.ident.clone();

        let controller_ty_str = CMetaStack::get("socketio", "controller_ty")
            .ok_or_else(|| Error::custom("socketio controller type not found in CMetaStack"))?;
        let namespace = CMetaStack::get("socketio", "namespace")
            .ok_or_else(|| Error::custom("socketio namespace not found in CMetaStack"))?;

        let controller_ty: Ident = syn::parse_str(&controller_ty_str)?;

        let event_kind = event_kind_tokens(&event_name.value());
        let controller_ty_name = controller_ty.to_string();

        let registration_name = format_ident!(
            "__SWORD_SOCKETIO_HANDLER_{}_{}",
            controller_ty_name.replace("::", "_"),
            fn_name
        );

        let handler_struct_name = format_ident!(
            "SwordSocketIoHandler{}_{}",
            controller_ty_name.replace("::", ""),
            fn_name.to_string().to_pascal_case()
        );

        Ok(Self {
            function,
            fn_name,
            controller_ty,
            namespace,
            event_name,
            event_kind,
            registration_name,
            handler_struct_name,
        })
    }
}

/// Maps a raw event name to its `SocketEventKind` tokens.
fn event_kind_tokens(event_name: &str) -> TokenStream2 {
    match event_name {
        "connection" => quote! { #SocketEventKind::Connection },
        "disconnection" => quote! { #SocketEventKind::Disconnection },
        "fallback" => quote! { #SocketEventKind::Fallback },
        custom => quote! { #SocketEventKind::Message(#custom) },
    }
}
