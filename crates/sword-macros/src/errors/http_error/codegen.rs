use proc_macro2::TokenStream;
use quote::quote;
use syn::{Fields, Ident, Type};

use super::parse::HttpErrorConfig;
use crate::errors::MessageValue;

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

        let pattern = Self::generate_pattern(enum_name, variant_name, fields);
        let tracing_stmt = Self::generate_tracing_stmt(variant_name, config, fields);
        let builder = Self::generate_json_builder(config);

        quote! {
            #pattern => {
                #tracing_stmt
                #builder
            },
        }
    }

    pub fn generate_pattern(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
    ) -> TokenStream {
        match fields {
            Fields::Named(named) => {
                let field_names: Vec<_> = named
                    .named
                    .iter()
                    .map(|field| field.ident.as_ref())
                    .collect();
                quote! { #enum_name::#variant_name { #(#field_names),* } }
            }
            Fields::Unnamed(_) => {
                quote! { #enum_name::#variant_name(_inner) }
            }
            Fields::Unit => {
                quote! { #enum_name::#variant_name }
            }
        }
    }

    pub fn generate_json_builder(config: &HttpErrorConfig) -> TokenStream {
        let status_code = config.code.as_ref().unwrap().as_u16();

        let message_expr = match &config.message {
            Some(MessageValue::Static(message)) => quote! { #message },
            Some(MessageValue::Field(field_name)) => {
                let field_ident = Ident::new(field_name, proc_macro2::Span::call_site());
                quote! { format!("{}", #field_ident) }
            }
            None => {
                let default_message = config.default_message();
                quote! { #default_message }
            }
        };

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
        let Some(level) = &config.tracing_level else {
            return quote! {};
        };

        let tracing_macro = match level.as_str() {
            "trace" => quote! { ::sword::internal::tracing::trace },
            "debug" => quote! { ::sword::internal::tracing::debug },
            "info" => quote! { ::sword::internal::tracing::info },
            "warn" => quote! { ::sword::internal::tracing::warn },
            "error" => quote! { ::sword::internal::tracing::error },
            _ => return quote! {},
        };

        let variant_str = variant_name.to_string();
        let status_code = config.code.as_ref().unwrap().as_u16();

        match fields {
            Fields::Named(named) => {
                let field_logs = named.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    quote! { #field_name = ?#field_name, }
                });

                quote! {
                    #tracing_macro!(
                        error = %__sword_internal_error,
                        error_type = #variant_str,
                        status_code = #status_code,
                        #(#field_logs)*
                        "HTTP error response"
                    );
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    #tracing_macro!(
                        error = %__sword_internal_error,
                        inner = ?_inner,
                        error_type = #variant_str,
                        status_code = #status_code,
                        "HTTP error response"
                    );
                }
            }
            Fields::Unit => {
                quote! {
                    #tracing_macro!(
                        error = %__sword_internal_error,
                        error_type = #variant_str,
                        status_code = #status_code,
                        "HTTP error response"
                    );
                }
            }
        }
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
        if let Type::Path(type_path) = ty {
            if let Some(segment) = type_path.path.segments.last() {
                return segment.ident == "String";
            }
        }
        false
    }
}
