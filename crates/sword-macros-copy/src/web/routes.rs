mod context;
mod expand;
mod parsing;

pub use context::WebRouteContext;
pub use parsing::{ParsedRouteAttribute, RequestMode};

use crate::shared::Result;
use proc_macro::TokenStream;

/// Entry point for `#[get]`, `#[post]`, ... route attributes.
pub fn expand_route(method: &str, attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let parsed = ParsedRouteAttribute::parse(method, attr, item)?;
    Ok(expand::expand_web_attribute(parsed).into())
}
