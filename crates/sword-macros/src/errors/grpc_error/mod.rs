mod codegen;
mod parse;

pub use parse::{parse_enum_grpc_error_config, parse_variant_grpc_error_config};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error};

pub fn derive_grpc_error(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "GrpcError can only be derived for enums",
        ));
    };

    let defaults = parse_enum_grpc_error_config(&input.attrs)?;
    let mut from_arms = Vec::new();

    for variant in &data.variants {
        let variant_config = parse_variant_grpc_error_config(&variant.ident, &variant.attrs)?;
        let merged = variant_config.merged(&defaults);

        codegen::GrpcErrorCodegen::validate_variant_config(
            enum_name,
            &variant.ident,
            &variant.fields,
            &merged,
        )?;

        from_arms.push(codegen::GrpcErrorCodegen::generate_from_arm(
            enum_name,
            &variant.ident,
            &variant.fields,
            &merged,
        )?);
    }

    Ok(quote! {
        impl From<#enum_name> for ::sword::grpc::Status {
            fn from(err: #enum_name) -> Self {
                let __sword_internal_error = err.to_string();

                match err {
                    #(#from_arms)*
                }
            }
        }
    })
}
