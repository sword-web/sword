use crate::shared::{Result, paths::*};
use darling::{FromMeta, ast::NestedMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

// `#[event(key = "literal_str")]` macro Args
#[derive(FromMeta)]
struct Args {
    key: String,
}

pub fn expand_event_struct(args: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(item)?;
    let args = Args::from_list(&NestedMeta::parse_meta_list(args.into())?)?;

    let event_key = args.key;
    let struct_name = &input.ident;

    let expanded = quote! {
        #input

        impl #Event for #struct_name {
            fn key(&self) -> &'static str {
                #event_key
            }

            fn clone_event(&self) -> #Box<dyn #Event> {
                #Box::new(self.clone())
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
