mod expand;
mod parse;
mod validate;

use crate::shared::Result;
use crate::shared::paths::*;

use parse::GrpcErrorArgs;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput};

pub fn derive_grpc_error(input: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = syn::parse(input)?;
    let enum_ty = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(
            darling::Error::custom("GrpcError can only be derived for enums").with_span(enum_ty),
        );
    };

    let defaults = GrpcErrorArgs::container(&input)?;

    let mut from_arms = Vec::new();

    for variant in &data.variants {
        let variant_args = GrpcErrorArgs::variant(variant)?;
        let merged = variant_args.merged(&defaults);

        validate::validate_variant(enum_ty, &variant.ident, &variant.fields, &merged)?;

        from_arms.push(expand::generate_from_arm(
            enum_ty,
            &variant.ident,
            &variant.fields,
            &merged,
        )?);
    }

    let expanded = quote! {
        impl From<#enum_ty> for #GrpcStatus {
            fn from(err: #enum_ty) -> Self {
                let __sword_internal_error = err.to_string();

                match err {
                    #(#from_arms)*
                }
            }
        }
    };

    Ok(expanded.into())
}
