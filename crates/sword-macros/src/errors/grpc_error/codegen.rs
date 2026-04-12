use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Fields, Ident};

use super::parse::GrpcErrorConfig;
use crate::errors::MessageValue;

pub struct GrpcErrorCodegen;

impl GrpcErrorCodegen {
    pub fn validate_variant_config(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
        config: &GrpcErrorConfig,
    ) -> syn::Result<()> {
        if config.transparent {
            let is_single_unnamed =
                matches!(fields, Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1);

            if !is_single_unnamed {
                return Err(Error::new_spanned(
                    variant_name,
                    "transparent variants must have exactly one unnamed field",
                ));
            }

            return Ok(());
        }

        match fields {
            Fields::Named(named) => {
                if let Some(MessageValue::Field(field_name)) = &config.message {
                    let exists = named
                        .named
                        .iter()
                        .filter_map(|field| field.ident.as_ref())
                        .any(|ident| ident == field_name);

                    if !exists {
                        return Err(Error::new_spanned(
                            variant_name,
                            format!(
                                "`message = {field_name}` references a missing field on {enum_name}::{variant_name}`"
                            ),
                        ));
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() != 1 {
                    return Err(Error::new_spanned(
                        variant_name,
                        "non-transparent tuple variants are only supported with exactly one field",
                    ));
                }

                if matches!(config.message, Some(MessageValue::Field(_))) {
                    return Err(Error::new_spanned(
                        variant_name,
                        "tuple variants do not support `message = field`; use a named-field variant or build the final client message before creating the error",
                    ));
                }
            }
            Fields::Unit => {
                if matches!(config.message, Some(MessageValue::Field(_))) {
                    return Err(Error::new_spanned(
                        variant_name,
                        "unit variants do not support field-based `message`",
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn generate_from_arm(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
        config: &GrpcErrorConfig,
    ) -> syn::Result<TokenStream> {
        if config.transparent {
            return Ok(quote! {
                #enum_name::#variant_name(inner) => ::sword::grpc::Status::from(inner),
            });
        }

        let code_variant = Self::parse_code_to_tonic_variant(config.default_code(), variant_name)?;
        let pattern = Self::generate_pattern(enum_name, variant_name, fields);
        let message_expr = Self::generate_message_expr(config, fields);
        let tracing_stmt = Self::generate_tracing_stmt(variant_name, config, fields, &code_variant);

        Ok(quote! {
            #pattern => {
                #tracing_stmt
                ::sword::grpc::Status::new(::sword::grpc::Code::#code_variant, #message_expr)
            },
        })
    }

    pub fn parse_code_to_tonic_variant(code: &str, variant_name: &Ident) -> syn::Result<Ident> {
        let variant = match code {
            "ok" => "Ok",
            "cancelled" => "Cancelled",
            "unknown" => "Unknown",
            "invalid_argument" => "InvalidArgument",
            "deadline_exceeded" => "DeadlineExceeded",
            "not_found" => "NotFound",
            "already_exists" => "AlreadyExists",
            "permission_denied" => "PermissionDenied",
            "resource_exhausted" => "ResourceExhausted",
            "failed_precondition" => "FailedPrecondition",
            "aborted" => "Aborted",
            "out_of_range" => "OutOfRange",
            "unimplemented" => "Unimplemented",
            "internal" => "Internal",
            "unavailable" => "Unavailable",
            "data_loss" => "DataLoss",
            "unauthenticated" => "Unauthenticated",
            _ => {
                return Err(Error::new_spanned(
                    variant_name,
                    "invalid gRPC code; use one of: ok, cancelled, unknown, invalid_argument, deadline_exceeded, not_found, already_exists, permission_denied, resource_exhausted, failed_precondition, aborted, out_of_range, unimplemented, internal, unavailable, data_loss, unauthenticated",
                ));
            }
        };

        Ok(Ident::new(variant, Span::call_site()))
    }

    pub fn generate_pattern(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
    ) -> TokenStream {
        match fields {
            Fields::Named(named) => {
                let field_names: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
                quote! { #enum_name::#variant_name { #(#field_names),* } }
            }
            Fields::Unnamed(_) => quote! { #enum_name::#variant_name(_inner) },
            Fields::Unit => quote! { #enum_name::#variant_name },
        }
    }

    pub fn generate_message_expr(config: &GrpcErrorConfig, fields: &Fields) -> TokenStream {
        match &config.message {
            Some(MessageValue::Static(message)) => quote! { #message },
            Some(MessageValue::Field(field_name)) => {
                let field_ident = Ident::new(field_name, Span::call_site());
                quote! { format!("{}", #field_ident) }
            }
            None => match fields {
                Fields::Unnamed(_) => quote! { format!("{}", _inner) },
                _ => quote! { __sword_internal_error.clone() },
            },
        }
    }

    pub fn generate_tracing_stmt(
        variant_name: &Ident,
        config: &GrpcErrorConfig,
        fields: &Fields,
        code_variant: &Ident,
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
        let code_str = code_variant.to_string();

        match fields {
            Fields::Unit => {
                quote! {
                    #tracing_macro!(
                        error = %__sword_internal_error,
                        error_type = #variant_str,
                        grpc_code = #code_str,
                        "gRPC error response"
                    );
                }
            }
            Fields::Unnamed(_) => {
                quote! {
                    #tracing_macro!(
                        error = %__sword_internal_error,
                        inner = ?_inner,
                        error_type = #variant_str,
                        grpc_code = #code_str,
                        "gRPC error response"
                    );
                }
            }
            Fields::Named(named) => {
                let field_logs = named.named.iter().map(|field| {
                    let field_name = field.ident.as_ref().unwrap();
                    quote! { #field_name = ?#field_name, }
                });

                quote! {
                    #tracing_macro!(
                        error = %__sword_internal_error,
                        #(#field_logs)*
                        error_type = #variant_str,
                        grpc_code = #code_str,
                        "gRPC error response"
                    );
                }
            }
        }
    }
}
