use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Fields, Ident, Type};

use super::parse::HttpErrorConfig;
use crate::errors;

pub struct HttpErrorCodegen;

impl HttpErrorCodegen {
    pub fn generate_from_arm(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
        config: &HttpErrorConfig,
    ) -> TokenStream {
        if config.transparent {
            return quote! {
                #enum_name::#variant_name(inner) => ::sword::web::JsonResponse::from(inner),
            };
        }

        let pattern = errors::generate_pattern(enum_name, variant_name, fields);
        let tracing_stmt = Self::generate_tracing_stmt(variant_name, config, fields);
        let builder = Self::generate_json_builder(config);

        quote! {
            #pattern => {
                #tracing_stmt
                #builder
            },
        }
    }

    pub fn generate_json_builder(config: &HttpErrorConfig) -> TokenStream {
        let status_code = config.code.as_ref().unwrap().as_u16();

        let message_expr = errors::generate_message_expr(&config.message, || {
            let default_message = config.default_message();
            quote! { #default_message }
        });

        let mut builder = quote! {
            ::sword::web::JsonResponse::status(#status_code).message(#message_expr)
        };

        if let Some(field_name) = &config.error_field {
            let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
            builder = quote! { #builder.error(&#field_ident) };
        }

        if let Some(field_name) = &config.errors_field {
            let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
            builder = quote! { #builder.errors(&#field_ident) };
        }

        builder
    }

    pub fn generate_tracing_stmt(
        variant_name: &Ident,
        config: &HttpErrorConfig,
        fields: &Fields,
    ) -> TokenStream {
        let status_code = config.code.as_ref().unwrap().as_u16();

        errors::generate_tracing_stmt(
            variant_name,
            &config.tracing_level,
            fields,
            &Ident::new("status_code", Span::call_site()),
            quote! { #status_code },
            "HTTP error response",
        )
    }

    pub fn generate_variant_fn(variant_name: &Ident, fields: &Fields) -> TokenStream {
        let Fields::Named(named) = fields else {
            return quote! {};
        };

        let fn_name = variant_name;

        let params = named.named.iter().map(|field| {
            let name = &field.ident;
            let ty = &field.ty;

            if Self::is_string_type(ty) {
                quote! { #name: impl Into<String> }
            } else {
                quote! { #name: #ty }
            }
        });

        let field_assignments = named.named.iter().map(|field| {
            let name = &field.ident;
            let ty = &field.ty;

            if Self::is_string_type(ty) {
                quote! { #name: #name.into() }
            } else {
                quote! { #name }
            }
        });

        quote! {
            #[allow(non_snake_case)]
            pub fn #fn_name(#(#params),*) -> Self {
                Self::#variant_name {
                    #(#field_assignments),*
                }
            }
        }
    }

    fn is_string_type(ty: &Type) -> bool {
        if let Type::Path(type_path) = ty
            && let Some(segment) = type_path.path.segments.last()
        {
            return segment.ident == "String";
        }
        false
    }
}
