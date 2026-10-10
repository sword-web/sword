use crate::{core::ControllerStruct, shared::Result};
use proc_macro2::TokenStream;

pub fn expand_controller(_controller: &ControllerStruct) -> Result<TokenStream> {
    todo!("implement the socketio controller expansion")
}
