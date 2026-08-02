use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Fields, Ident};

use super::parse::GrpcErrorConfig;
use crate::errors::{MessageValue, validate_message_references};

pub struct GrpcErrorCodegen;

impl GrpcErrorCodegen {
    pub fn validate_variant_config(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
        config: &GrpcErrorConfig,
    ) -> syn::Result<()> {
        if config.transparent {
            crate::errors::validate_transparent_single_unnamed(variant_name, fields)?;
            return Ok(());
        }

        match fields {
            Fields::Named(named) => {
                validate_message_references(enum_name, variant_name, named, &config.message)?;
            }
            Fields::Unnamed(unnamed) => {
                crate::errors::validate_single_unnamed(variant_name, unnamed)?;

                if matches!(
                    config.message,
                    Some(MessageValue::Field(_) | MessageValue::Interpolated(_))
                ) {
                    return Err(Error::new_spanned(
                        variant_name,
                        "tuple variants do not support `message = field` or interpolated messages; use a named-field variant or build the final client message before creating the error",
                    ));
                }
            }
            Fields::Unit => {
                if matches!(
                    config.message,
                    Some(MessageValue::Field(_) | MessageValue::Interpolated(_))
                ) {
                    return Err(Error::new_spanned(
                        variant_name,
                        "unit variants do not support field-based or interpolated `message`",
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
        let pattern = crate::errors::generate_pattern(enum_name, variant_name, fields);
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

    pub fn generate_message_expr(config: &GrpcErrorConfig, fields: &Fields) -> TokenStream {
        crate::errors::generate_message_expr(&config.message, || match fields {
            Fields::Unnamed(_) => quote! { format!("{}", _inner) },
            _ => quote! { __sword_internal_error.clone() },
        })
    }

    pub fn generate_tracing_stmt(
        variant_name: &Ident,
        config: &GrpcErrorConfig,
        fields: &Fields,
        code_variant: &Ident,
    ) -> TokenStream {
        let code_str = code_variant.to_string();

        crate::errors::generate_tracing_stmt(
            variant_name,
            &config.tracing_level,
            fields,
            &Ident::new("grpc_code", Span::call_site()),
            quote! { #code_str },
            "gRPC error response",
        )
    }
}
