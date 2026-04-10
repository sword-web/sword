mod parse;

use parse::{HttpErrorConfig, MessageValue};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Ident, Type};

pub fn derive_http_error(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_name = &input.ident;

    let Data::Enum(data) = &input.data else {
        return Err(Error::new_spanned(
            input,
            "HttpError can only be derived for enums",
        ));
    };

    let variants = &data.variants;
    let mut from_arms = Vec::new();
    let mut variant_fns = Vec::new();

    for variant in variants {
        let config = HttpErrorConfig::from_attrs(&variant.ident, &variant.attrs)?;

        if config.transparent {
            let is_single_unnamed_field = matches!(
                &variant.fields,
                Fields::Unnamed(f) if f.unnamed.len() == 1
            );

            if !is_single_unnamed_field {
                return Err(Error::new_spanned(
                    &variant.ident,
                    "transparent variants must have exactly one unnamed field",
                ));
            }
        }

        if (config.error_field.is_some() || config.errors_field.is_some())
            && !matches!(&variant.fields, Fields::Named(_))
        {
            return Err(Error::new_spanned(
                &variant.ident,
                "`error` and `errors` can only be used with named fields",
            ));
        }

        variant_fns.push(generate_variant_fn(&variant.ident, &variant.fields));

        from_arms.push(generate_from_arm(
            enum_name,
            &variant.ident,
            &variant.fields,
            &config,
        ));
    }

    Ok(quote! {
        impl From<#enum_name> for ::sword::web::JsonResponse {
            fn from(err: #enum_name) -> Self {
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

fn generate_variant_fn(variant_name: &Ident, fields: &Fields) -> TokenStream {
    let Fields::Named(named) = fields else {
        return quote! {};
    };

    let fn_name = variant_name;

    let params = named.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

        if is_string_type(ty) {
            quote! { #name: impl Into<String> }
        } else {
            quote! { #name: #ty }
        }
    });

    let field_assignments = named.named.iter().map(|f| {
        let name = &f.ident;
        let ty = &f.ty;

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

fn is_string_type(ty: &syn::Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "String";
        }
    }

    false
}

fn generate_from_arm(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &HttpErrorConfig,
) -> TokenStream {
    if !config.transparent {
        let pattern = generate_pattern(enum_name, variant_name, fields);
        let tracing_stmt = generate_tracing_stmt(variant_name, config, fields);
        let builder = generate_json_builder(fields, config);

        return quote! {
            #pattern => {
                #tracing_stmt
                #builder
            },
        };
    }

    quote! {
        #enum_name::#variant_name(inner) => ::sword::web::JsonResponse::from(inner),
    }
}

fn generate_pattern(enum_name: &Ident, variant_name: &Ident, fields: &Fields) -> TokenStream {
    match fields {
        Fields::Named(named) => {
            let field_names: Vec<_> = named.named.iter().map(|f| &f.ident).collect();
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

fn generate_json_builder(fields: &Fields, config: &HttpErrorConfig) -> TokenStream {
    let code = config.code.as_ref().unwrap().as_u16();

    let message_expr = match config.message() {
        Some(MessageValue::Static(msg)) => {
            quote! { format!(#msg) }
        }
        Some(MessageValue::Field(field_name)) => {
            let field_ident = Ident::new(&field_name, Span::call_site());
            quote! { format!("{}", #field_ident) }
        }
        None => {
            let default_msg = config.default_message();
            quote! { #default_msg }
        }
    };

    let base = quote! {
        ::sword::web::JsonResponse::status(#code).message(#message_expr)
    };

    match fields {
        Fields::Named(_) => {
            if let Some(field) = &config.error_field {
                let field_ident = Ident::new(field, Span::call_site());
                quote! { #base.error(#field_ident) }
            } else if let Some(field) = &config.errors_field {
                let field_ident = Ident::new(field, Span::call_site());
                quote! { #base.errors(#field_ident) }
            } else {
                base
            }
        }
        _ => base,
    }
}

fn generate_tracing_stmt(
    variant_name: &Ident,
    config: &HttpErrorConfig,
    fields: &Fields,
) -> TokenStream {
    if config.transparent {
        return quote! {};
    }

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
        Fields::Unit => {
            quote! {
                #tracing_macro!(
                    error_type = #variant_str,
                    status_code = #status_code,
                    "HTTP error response"
                );
            }
        }

        Fields::Unnamed(f) if f.unnamed.len() == 1 => {
            quote! {
                #tracing_macro!(
                    error = ?_inner,
                    error_type = #variant_str,
                    status_code = #status_code,
                    "HTTP error response"
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
                    #(#field_logs)*
                    error_type = #variant_str,
                    status_code = #status_code,
                    "HTTP error response"
                );
            }
        }

        _ => quote! {},
    }
}
