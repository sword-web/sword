mod expand;
mod parsing;

use proc_macro::TokenStream;

use crate::shared::{Result, cmeta::CMetaStack};

/// Entry point for the `#[on("...")]` handler attribute.
pub fn expand_on_handler(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Handshake: leave the method untouched outside a Socket.IO controller.
    if CMetaStack::get("socketio", "controller_ty").is_none() {
        return Ok(item);
    }

    let parsed = parsing::ParsedOnHandler::parse(attr, item)?;
    Ok(expand::expand_on_attribute(parsed).into())
}
