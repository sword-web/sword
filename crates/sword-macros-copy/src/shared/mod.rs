pub mod paths;
pub mod structs;
pub mod traits;

pub use darling::Error;

pub type Result<T> = std::result::Result<T, darling::Error>;

pub fn emit(result: Result<proc_macro::TokenStream>) -> proc_macro::TokenStream {
    match result {
        Ok(ts) => ts,
        Err(err) => err.write_errors().into(),
    }
}
