pub mod cmeta;
pub mod errors;
pub mod location;
pub mod paths;
pub mod structs;
pub mod traits;

pub use darling::Error;
use syn::{FnArg, Type};

pub type Result<T> = std::result::Result<T, darling::Error>;

pub fn emit(result: Result<proc_macro::TokenStream>) -> proc_macro::TokenStream {
    match result {
        Ok(ts) => ts,
        Err(err) => err.write_errors().into(),
    }
}

pub fn expect_type(arg: &FnArg, expected: &str) -> Result<()> {
    let FnArg::Typed(pat) = arg else {
        return Ok(());
    };

    let last = match pat.ty.as_ref() {
        Type::Path(type_path) => type_path
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string()),
        _ => None,
    };

    if last.as_deref() != Some(expected) {
        let message = format!("this parameter must be of type `{expected}`");
        Err(Error::custom(message).with_span(&pat.ty))?
    }

    Ok(())
}
