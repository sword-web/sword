use crate::shared::{Error, Result, paths::*};
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

    if !matches!(input.data, syn::Data::Struct(_)) {
        return Err(
            Error::custom("#[event] can only be applied to structs").with_span(&input.ident)
        );
    }

    let event_key = args.key;
    let struct_ty = &input.ident;

    let expanded = quote! {
        #input

        impl #Event for #struct_ty {
            fn key(&self) -> &'static str {
                #event_key
            }

            fn to_json(&self) -> Result<Vec<u8>, #EventCodecError> {
                #events_serde_json::to_vec(self).map_err(Into::into)
            }
        }

        #inventory::submit! {
            #EventRegistrar {
                key: #event_key,
                from_json: |bytes: &[u8]| -> Result<Box<dyn #Event>, #EventCodecError> {
                    #events_serde_json::from_slice::<#struct_ty>(bytes)
                        .map(|event| Box::new(event) as Box<dyn #Event>)
                        .map_err(Into::into)
                },
            }
        }

    };

    Ok(TokenStream::from(expanded))
}
