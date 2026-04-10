mod generation;
pub(crate) mod parsing;

use proc_macro::TokenStream;

use generation::WebRouteGenerator;
pub use parsing::{ParsedRouteAttribute, RequestMode};

pub fn attribute(attribute_str: &str, attr: TokenStream, item: TokenStream) -> TokenStream {
    let parsed = match ParsedRouteAttribute::parse(attribute_str, attr, item) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error().into(),
    };

    WebRouteGenerator::new(parsed).expand()
}
