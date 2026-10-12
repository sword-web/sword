mod expand;
mod parse;
mod validate;

use crate::shared::Result;
use crate::shared::paths::*;

use parse::HttpErrorArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_http_error(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse(input)?;
    let enum_ty = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(
            darling::Error::custom("HttpError can only be derived for enums").with_span(enum_ty),
        );
    };

    let defaults = HttpErrorArgs::container(&input)?;

    let mut from_arms = Vec::new();
    let mut variant_fns = Vec::new();

    for variant in &data.variants {
        let variant_args = HttpErrorArgs::variant(variant)?;
        let merged = variant_args.merged(&defaults);

        validate::validate_variant(enum_ty, &variant.ident, &variant.fields, &merged)?;

        from_arms.push(expand::generate_from_arm(
            enum_ty,
            &variant.ident,
            &variant.fields,
            &merged,
        ));

        variant_fns.push(expand::generate_variant_fn(&variant.ident, &variant.fields));
    }

    let expanded = quote! {
        impl From<#enum_ty> for #JsonResponse {
            fn from(err: #enum_ty) -> Self {
                let __sword_internal_error = err.to_string();

                match err {
                    #(#from_arms)*
                }
            }
        }

        impl #IntoResponse for #enum_ty {
            fn into_response(self) -> #AxumResponse {
                #JsonResponse::from(self).into_response()
            }
        }

        #[allow(non_snake_case)]
        impl #enum_ty {
            #(#variant_fns)*
        }
    };

    Ok(expanded.into())
}
