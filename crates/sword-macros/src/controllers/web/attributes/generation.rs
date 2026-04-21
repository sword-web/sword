use super::parsing::ParsedRouteAttribute;
use crate::controllers::web::expand_web_interceptor_args;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub struct WebRouteGenerator {
    route: ParsedRouteAttribute,
}

impl WebRouteGenerator {
    pub fn new(route: ParsedRouteAttribute) -> Self {
        Self { route }
    }

    pub fn expand(self) -> TokenStream1 {
        let route_fn_name = self.route_fn_name();
        let handler_with_interceptors = self.apply_route_interceptors(self.build_handler_router());
        let inventory_registration = self.build_route_registration(&route_fn_name);
        let input_fn = &self.route.function;

        TokenStream1::from(quote! {
            #input_fn

            pub fn #route_fn_name(
                controller: std::sync::Arc<Self>,
                state: ::sword::internal::core::State,
            ) -> ::sword::internal::web::MethodRouter<::sword::internal::core::State> {
                #handler_with_interceptors
            }

            #inventory_registration
        })
    }

    fn route_fn_name(&self) -> syn::Ident {
        let fn_name = &self.route.function.sig.ident;
        format_ident!("__sword_route_{}", fn_name)
    }

    fn controller_ident(&self) -> syn::Ident {
        format_ident!("{}", self.route.context.controller_name)
    }

    fn build_handler_router(&self) -> TokenStream {
        let routing_fn = self.route.method.routing_fn_tokens();
        let call_parts = HandlerCallParts::from_function(&self.route.function);
        let fn_name = &call_parts.fn_name;

        if call_parts.has_params() {
            let closure_params = &call_parts.closure_params;
            let call_args = &call_parts.call_args;

            quote! {
                ::sword::internal::web::routing::#routing_fn({
                    let ctrl = std::sync::Arc::clone(&controller);
                    move |#(#closure_params),*| async move {
                        use ::sword::internal::web::IntoResponse;
                        ctrl.#fn_name(#(#call_args),*).await.into_response()
                    }
                })
            }
        } else {
            quote! {
                ::sword::internal::web::routing::#routing_fn({
                    let ctrl = std::sync::Arc::clone(&controller);
                    move || async move {
                        use ::sword::internal::web::IntoResponse;
                        ctrl.#fn_name().await.into_response()
                    }
                })
            }
        }
    }

    fn apply_route_interceptors(&self, mut handler: TokenStream) -> TokenStream {
        for interceptor in self.route.interceptors.iter().rev() {
            let generated_interceptor =
                expand_web_interceptor_args(interceptor, self.route.request_mode);
            handler = quote! {
                #handler.layer(#generated_interceptor)
            };
        }

        for interceptor in self.route.context.controller_interceptors.iter().rev() {
            let generated_interceptor =
                expand_web_interceptor_args(interceptor, self.route.request_mode);
            handler = quote! {
                #handler.layer(#generated_interceptor)
            };
        }

        handler
    }

    fn build_route_registration(&self, route_fn_name: &syn::Ident) -> TokenStream {
        let fn_name = &self.route.function.sig.ident;
        let controller_ident = self.controller_ident();
        let registration_name = format_ident!(
            "__SWORD_ROUTE_REGISTRAR_{}_{}",
            self.route.context.controller_name.replace("::", "_"),
            fn_name
        );

        let controller_name = &self.route.context.controller_name;
        let route_path = &self.route.path;
        let method = self.route.method.as_str();

        quote! {
            #[allow(non_upper_case_globals)]
            #[doc(hidden)]
            const #registration_name: () = {
                ::sword::internal::inventory::submit! {
                    ::sword::internal::web::RouteRegistrar {
                        controller_id: ::std::any::TypeId::of::<#controller_ident>(),
                        path: #route_path,
                        handler: |state: ::sword::internal::core::State| -> ::sword::internal::web::MethodRouter<::sword::internal::core::State> {
                            let controller =
                                state.borrow::<#controller_ident>().unwrap_or_else(|err| {
                                    ::sword::internal::core::sword_error!(
                                        title: "Failed to build HTTP controller",
                                        reason: err,
                                        context: {
                                            "controller" => #controller_name,
                                            "route" => format!("{} {}", #method, #route_path),
                                        },
                                        hints: ["Ensure all controller dependencies are registered in the DI container"],
                                    )
                                });

                            #controller_ident::#route_fn_name(controller, state)
                        },
                    }
                }
            };
        }
    }
}

struct HandlerCallParts {
    fn_name: syn::Ident,
    closure_params: Vec<TokenStream>,
    call_args: Vec<TokenStream>,
}

impl HandlerCallParts {
    fn from_function(function: &syn::ItemFn) -> Self {
        let fn_name = function.sig.ident.clone();

        let params: Vec<_> = function
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let syn::FnArg::Typed(pat_type) = arg {
                    Some(pat_type.ty.clone())
                } else {
                    None
                }
            })
            .collect();

        let closure_params = params
            .iter()
            .enumerate()
            .map(|(i, ty)| {
                let param_name = format_ident!("p{}", i);
                quote! { #param_name: #ty }
            })
            .collect();

        let call_args = (0..params.len())
            .map(|i| {
                let param_name = format_ident!("p{}", i);
                quote! { #param_name }
            })
            .collect();

        Self {
            fn_name,
            closure_params,
            call_args,
        }
    }

    fn has_params(&self) -> bool {
        !self.closure_params.is_empty()
    }
}
