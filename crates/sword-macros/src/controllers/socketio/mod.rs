mod generation;
mod on_handler;

use super::shared::{CMetaStack, ControllerStruct, ParsedControllerKind};
use generation::generate_socketio_controller_builder;
use proc_macro::TokenStream;
use syn::{Error, Path};

pub use on_handler::expand_on_handler;

pub fn expand_socketio_controller(input: &ControllerStruct) -> syn::Result<TokenStream> {
    let ParsedControllerKind::SocketIo { namespace } = &input.kind else {
        return Err(Error::new(
            input.name.span(),
            "Invalid Socket.IO controller",
        ));
    };

    let controller_name = input.name.to_string();

    CMetaStack::push("controller_kind", "socketio");
    CMetaStack::push("controller_path", namespace);
    CMetaStack::push("controller_name", &controller_name);
    CMetaStack::push("socketio_controller_name", &controller_name);
    CMetaStack::push("socketio_namespace", namespace);

    let interceptors: Vec<Path> = input
        .interceptors
        .iter()
        .filter_map(|interceptor| interceptor.sword_path().cloned())
        .collect();

    Ok(TokenStream::from(generate_socketio_controller_builder(
        input,
        &interceptors,
    )?))
}
