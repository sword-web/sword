use super::parse::GrpcErrorArgs;
use crate::shared::errors::{
    TracingLevel, generate_message_expr, generate_pattern, generate_tracing_stmt,
};

use crate::shared::paths::*;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Error, Fields, Ident};

pub fn generate_from_arm(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &GrpcErrorArgs,
) -> syn::Result<TokenStream> {
    if config.transparent.is_present() {
        return Ok(quote! {
            #enum_name::#variant_name(inner) => #GrpcStatus::from(inner),
        });
    }

    let code_variant = parse_code_to_tonic_variant(config.default_code(), variant_name)?;
    let pattern = generate_pattern(enum_name, variant_name, fields);
    let message_expr = generate_message_expr(&config.message, || match fields {
        Fields::Unnamed(_) => quote! { format!("{}", _inner) },
        _ => quote! { __sword_internal_error.clone() },
    });
    let tracing_stmt = generate_variant_tracing_stmt(variant_name, config, fields, &code_variant);

    Ok(quote! {
        #pattern => {
            #tracing_stmt
            #GrpcStatus::new(#GrpcCode::#code_variant, #message_expr)
        },
    })
}

fn parse_code_to_tonic_variant(code: &str, variant_name: &Ident) -> syn::Result<Ident> {
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

fn generate_variant_tracing_stmt(
    variant_name: &Ident,
    config: &GrpcErrorArgs,
    fields: &Fields,
    code_variant: &Ident,
) -> TokenStream {
    let code_str = code_variant.to_string();
    let level = config.tracing.or(Some(default_tracing_level(&code_str)));

    generate_tracing_stmt(
        variant_name,
        level,
        fields,
        &Ident::new("grpc_code", Span::call_site()),
        quote! { #code_str },
        "gRPC error response",
    )
}

/// Default tracing level for a gRPC code, matching the access logger's `auto`
/// policy: `ok` → info, client errors → warn, server errors → error.
fn default_tracing_level(code_str: &str) -> TracingLevel {
    match code_str {
        "Ok" => TracingLevel::Info,
        "InvalidArgument" | "NotFound" | "AlreadyExists" | "PermissionDenied"
        | "FailedPrecondition" | "OutOfRange" | "Unauthenticated" | "Aborted" | "Cancelled" => {
            TracingLevel::Warn
        }
        _ => TracingLevel::Error,
    }
}
