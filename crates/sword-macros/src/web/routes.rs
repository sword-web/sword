mod context;
mod expand;
mod parsing;

use proc_macro::TokenStream;

use crate::shared::Result;

pub use self::context::WebRouteContext;
pub use self::parsing::{ParsedRouteAttribute, RequestMode};

/// Entry point for `#[get]`, `#[post]`, ... route attributes.
pub fn expand_route(method: &str, attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let parsed = ParsedRouteAttribute::parse(method, attr, item)?;
    Ok(expand::expand_web_attribute(&parsed).into())
}
