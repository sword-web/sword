pub mod shared;

#[cfg(feature = "grpc-controllers")]
pub mod grpc;

#[cfg(feature = "socketio-controllers")]
pub mod socketio;

#[cfg(feature = "socketio-controllers")]
pub use socketio::expand_on_handler;

#[cfg(feature = "web-controllers")]
pub mod web;

use proc_macro::TokenStream;
use quote::quote;
use shared::{ControllerStruct, ParsedControllerKind};
use syn::ItemStruct;

pub fn expand_controller(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed_input = ControllerStruct::parse(attr, item)?;

    let controller_kind = &parsed_input.kind;

    let builder: proc_macro2::TokenStream = match controller_kind {
        #[cfg(feature = "web-controllers")]
        ParsedControllerKind::Web { .. } => web::expand_web_controller(&parsed_input)?,

        #[cfg(feature = "socketio-controllers")]
        ParsedControllerKind::SocketIo { .. } => {
            socketio::expand_socketio_controller(&parsed_input)?
        }

        #[cfg(feature = "grpc-controllers")]
        ParsedControllerKind::Grpc { .. } => grpc::expand_grpc_controller(&parsed_input)?,
    }
    .into();

    let expanded = quote! {
        #input
        #builder
    };

    Ok(TokenStream::from(expanded))
}
