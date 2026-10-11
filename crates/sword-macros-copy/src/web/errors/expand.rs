use super::parse::HttpErrorArgs;
use crate::shared::errors::{
    default_tracing_level, generate_message_expr, generate_pattern, generate_tracing_stmt,
};

use crate::shared::paths::*;

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Fields, Ident, Type};

pub fn generate_from_arm(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &HttpErrorArgs,
) -> TokenStream {
    if config.transparent.is_present() {
        return quote! {
            #enum_name::#variant_name(inner) => #JsonResponse::from(inner),
        };
    }

    let pattern = generate_pattern(enum_name, variant_name, fields);
    let tracing_stmt = generate_variant_tracing_stmt(variant_name, config, fields);
    let builder = generate_json_builder(config);

    quote! {
        #pattern => {
            #tracing_stmt
            #builder
        },
    }
}

fn generate_json_builder(config: &HttpErrorArgs) -> TokenStream {
    let status_code = config.code.expect("code validated before codegen");

    let message_expr = generate_message_expr(&config.message, || {
        let default_message = config.default_message();
        quote! { #default_message }
    });

    let mut builder = quote! {
        #JsonResponse::status(#status_code).message(#message_expr)
    };

    if let Some(field_name) = &config.error {
        let field_ident = Ident::new(field_name.as_str(), Span::call_site());
        builder = quote! { #builder.error(&#field_ident) };
    }

    if let Some(field_name) = &config.errors {
        let field_ident = Ident::new(field_name.as_str(), Span::call_site());
        builder = quote! { #builder.errors(&#field_ident) };
    }

    builder
}

fn generate_variant_tracing_stmt(
    variant_name: &Ident,
    config: &HttpErrorArgs,
    fields: &Fields,
) -> TokenStream {
    let status_code = config.code.expect("code validated before codegen");
    let level = config.tracing.or(Some(default_tracing_level(status_code)));

    generate_tracing_stmt(
        variant_name,
        level,
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

        if is_string_type(ty) {
            quote! { #name: impl Into<String> }
        } else {
            quote! { #name: #ty }
        }
    });

    let field_assignments = named.named.iter().map(|field| {
        let name = &field.ident;
        let ty = &field.ty;

        if is_string_type(ty) {
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
