mod codegen;
mod parse;
mod validate;

pub use parse::HttpErrorConfig;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error};

pub fn derive_http_error(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "HttpError can only be derived for enums",
        ));
    };

    let defaults = HttpErrorConfig::from_attrs(&input.attrs)?;
    let mut from_arms = Vec::new();
    let mut variant_fns = Vec::new();

    for variant in &data.variants {
        let variant_config =
            HttpErrorConfig::parse_enum_variant_config(&variant.ident, &variant.attrs)?;

        let merged = variant_config.merged(&defaults);

        validate::HttpErrorValidator::validate_variant_config(
            enum_name,
            &variant.ident,
            &variant.fields,
            &merged,
        )?;

        variant_fns.push(codegen::HttpErrorCodegen::generate_variant_fn(
            &variant.ident,
            &variant.fields,
        ));
        from_arms.push(codegen::HttpErrorCodegen::generate_from_arm(
            enum_name,
            &variant.ident,
            &variant.fields,
            &merged,
        ));
    }

    Ok(quote! {
        impl From<#enum_name> for ::sword::web::JsonResponse {
            fn from(err: #enum_name) -> Self {
                let __sword_internal_error = err.to_string();

                match err {
                    #(#from_arms)*
                }
            }
        }

        impl ::sword::internal::web::IntoResponse for #enum_name {
            fn into_response(self) -> ::sword::internal::web::AxumResponse {
                ::sword::web::JsonResponse::from(self).into_response()
            }
        }

        #[allow(non_snake_case)]
        impl #enum_name {
            #(#variant_fns)*
        }
    })
}
