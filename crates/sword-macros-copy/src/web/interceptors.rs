use crate::{core::InterceptorArgs, shared::paths::*, web::routes::RequestMode};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, Path};

pub struct RequestModeTokens {
    request_ty: TokenStream,
    traditional_trait: TokenStream,
    with_config_trait: TokenStream,
}

impl RequestModeTokens {
    fn from_request_mode(request_mode: RequestMode) -> Self {
        match request_mode {
            RequestMode::Streaming => Self {
                request_ty: quote! { #StreamRequest },
                traditional_trait: quote! { #OnRequestStream },
                with_config_trait: quote! { #OnRequestStreamWithConfig },
            },
            _ => Self {
                request_ty: quote! { #Request },
                traditional_trait: quote! { #OnRequest },
                with_config_trait: quote! { #OnRequestWithConfig },
            },
        }
    }
}

pub fn expand_sword_interceptor(args: &InterceptorArgs, req_mode: RequestMode) -> TokenStream {
    let mode = RequestModeTokens::from_request_mode(req_mode);

    match args {
        InterceptorArgs::Expression(expr) => quote! { #expr },
        InterceptorArgs::Traditional(path) => generate_sword_traditional(path, &mode),
        InterceptorArgs::WithConfig {
            interceptor,
            config,
        } => generate_sword_with_config(interceptor, &mode, config),
    }
}

fn generate_sword_traditional(path: &Path, mode: &RequestModeTokens) -> TokenStream {
    let request_type = mode.request_ty.clone();
    let simple_trait = mode.traditional_trait.clone();

    quote! {
        {
            fn __check_on_request__<M: #simple_trait>(mw: &M) -> &M { mw }

            let middleware = state.borrow::<#path>()
                .unwrap_or_else(|err| {
                    #sword_error!(
                        title: "Failed to retrieve HTTP interceptor from State",
                        reason: err,
                        context: {
                            "interceptor" => stringify!(#path),
                        },
                        hints: ["Ensure the interceptor is registered and built before controller initialization"],
                    )
                });

            let _ = __check_on_request__(&*middleware);

            #mw_with_state(state.clone(), move |mut req: #request_type, next: #Next| {
                req.set_next(next);

                let mw = #Arc::clone(&middleware);

                async move {
                    <#path as #simple_trait>::on_request(&*mw, req).await
                }
            })
        }
    }
}

fn generate_sword_with_config(
    interceptor_ty: &Path,
    req_mode: &RequestModeTokens,
    config: &Expr,
) -> TokenStream {
    let request_ty = &req_mode.request_ty;
    let with_config_trait = &req_mode.with_config_trait;

    quote! {
        {
            fn __check_on_request_configured__<M, C>(mw: &M) -> &M
            where
                M: #with_config_trait<C>
            {
                mw
            }

            let interceptor = state.borrow::<#interceptor_ty>()
                .unwrap_or_else(|err| {
                    #sword_error! {
                        title: "Failed to retrieve HTTP interceptor from State",
                        reason: err,
                        context: {
                            "interceptor" => stringify!(#interceptor_ty),
                        },
                        hints: ["Ensure the interceptor is registered and built before controller initialization"],
                    }
                });

            let _ = __check_on_request_configured__::<#interceptor_ty, _>(&*middleware);

            #mw_with_state(state.clone(), move |mut req: #request_ty, next: #Next| {
                req.set_next(next);

                let mw = #Arc::clone(&middleware);

                async move {
                    <#interceptor_ty as #with_config_trait<_>>::on_request(&*mw, #config, req).await
                }
            })
        }
    }
}
