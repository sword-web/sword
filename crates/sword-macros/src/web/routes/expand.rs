use crate::shared::{location, paths::*};
use crate::web::interceptors::expand_sword_interceptor;
use crate::web::routes::ParsedRouteAttribute;

use super::parsing::ReturnKind;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn expand_web_attribute(route: &ParsedRouteAttribute) -> TokenStream {
    let original_function = &route.function;
    let route_fn_name = route.route_fn_name();
    let controller_ty = format_ident!("{}", route.context.controller_name);

    let path = &route.path;
    let method = route.method.as_str();

    let route_ident = &route.function.sig.ident;
    let location = location::tokens(route_ident.span(), &route_ident.to_string());

    let registration_name = format_ident!(
        "__SWORD_ROUTE_REGISTRAR_{}_{}",
        &route.context.controller_name.replace("::", "_"),
        route_fn_name
    );

    let handler_router = build_handler_router(route);
    let route_fn_body = apply_route_interceptors(route, handler_router);

    let expanded = quote! {
        #original_function

        pub fn #route_fn_name(
            controller: #Arc<Self>,
            state: #State,
        ) -> ::std::result::Result<#MethodRouter<#State>, #DiagnosticError> {
            Ok(#route_fn_body)
        }

        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            #inventory::submit! {
                #RouteRegistrar {
                    controller_id: #TypeId::of::<#controller_ty>(),
                    path: #path,
                    handler: |state: #State| -> ::std::result::Result<
                        #MethodRouter<#State>,
                        #DiagnosticError,
                    > {
                        let controller = state.borrow::<#controller_ty>().map_err(|err| {
                            #sword_error! {
                                code: "sword.build.web",
                                phase: #DiagnosticPhase::Build,
                                target: "web route",
                                title: "Failed to build HTTP controller",
                                reason: err,
                                source_location: {
                                    file: (#location).file(),
                                    line: (#location).line(),
                                    column: (#location).column(),
                                    length: (#location).length(),
                                    message: format!("`{}`", stringify!(#controller_ty)),
                                },
                                context: {
                                    "controller" => #stringify!(#controller_ty),
                                    "route" => format!("{} {}", #method, #path),
                                },
                                hints: ["Ensure all controller dependencies are registered in the DI container"],
                                cause: err,
                            }
                        })?;

                        #controller_ty::#route_fn_name(controller, state)
                    }
                }
            }
        };
    };

    expanded
}

// ---- handler router --------------------------------------------------------

fn build_handler_router(route: &ParsedRouteAttribute) -> TokenStream {
    let routing_fn = route.method.routing_fn_tokens();
    let body = build_handler_body(route);

    match route.request_mode.to_tokens() {
        Some(req_ty) => quote! {
            #routing::#routing_fn({
                let controller = #Arc::clone(&controller);
                move |req: #req_ty| async move { #body }
            })
        },
        None => quote! {
            #routing::#routing_fn({
                let controller = #Arc::clone(&controller);
                move || async move { #body }
            })
        },
    }
}

// ---- interceptors ----------------------------------------------------------

fn apply_route_interceptors(route: &ParsedRouteAttribute, mut handler: TokenStream) -> TokenStream {
    for interceptor in route.interceptors.iter().rev() {
        let generated = expand_sword_interceptor(interceptor, route.request_mode);
        handler = quote! { #handler.layer(#generated) };
    }

    for interceptor in route.context.controller_interceptors.iter().rev() {
        let generated = expand_sword_interceptor(interceptor, route.request_mode);
        handler = quote! { #handler.layer(#generated) };
    }

    handler
}

// ---- handler bodies --------------------------------------------------------

fn build_handler_body(route: &ParsedRouteAttribute) -> TokenStream {
    if route.method.is_sse() {
        build_sse_handler_body(route)
    } else if route.return_info.is_result {
        build_result_handler_body(route)
    } else {
        build_plain_handler_body(route)
    }
}

fn method_call(route: &ParsedRouteAttribute) -> TokenStream {
    let method = &route.function.sig.ident;

    if route.request_mode.to_tokens().is_some() {
        quote! { controller.#method(req).await }
    } else {
        quote! { controller.#method().await }
    }
}

fn build_plain_handler_body(route: &ParsedRouteAttribute) -> TokenStream {
    let call = method_call(route);

    quote! {
        #IntoResponse::into_response(#call)
    }
}

fn build_result_handler_body(route: &ParsedRouteAttribute) -> TokenStream {
    let call = method_call(route);
    let status = route.return_info.status_code;

    let ok_arm = match route.return_info.kind {
        ReturnKind::Passthrough => quote! {
            Ok(__data) => {
                #IntoResponse::into_response(__data)
            }
        },
        ReturnKind::Serialize => quote! {
            Ok(__data) => {
                #IntoResponse::into_response(#JsonResponse::status(#status).data(__data))
            }
        },
        ReturnKind::Empty => quote! {
            Ok(_) => {
                #IntoResponse::into_response(#JsonResponse::status(#status))
            }
        },
    };

    quote! {
        match #call {
            #ok_arm,
            Err(__err) => {
               #IntoResponse::into_response(__err)
            },
        }
    }
}

fn build_sse_handler_body(route: &ParsedRouteAttribute) -> TokenStream {
    let call = method_call(route);

    if route.return_info.is_result {
        quote! {
            match #call {
                Ok(__data) => {
                    #IntoResponse::into_response(__data)
                }
                Err(__err) => {
                    #IntoResponse::into_response(__err)
                }
            }
        }
    } else {
        quote! {
            #IntoResponse::into_response(#call)
        }
    }
}
