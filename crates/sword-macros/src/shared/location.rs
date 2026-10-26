use proc_macro2::{Span, TokenStream};
use quote::quote_spanned;

/// Emits an expression that builds a [`SourceLocation`] for a user item.
///
/// `file!`, `line!` and `column!` resolve against the span of the tokens they
/// are written in. Expanding them with the item's own span makes them point at
/// the user's declaration rather than at the attribute that triggered the macro.
///
/// [`SourceLocation`]: sword_core::diagnostic::SourceLocation
pub fn tokens(span: Span, name: &str) -> TokenStream {
    let length = name.chars().count();

    quote_spanned! { span =>
        ::sword::internal::core::diagnostic::SourceLocation::new(
            file!(),
            line!() as usize,
            column!() as usize,
            #length,
        )
    }
}
