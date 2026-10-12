use super::parsing::ParsedOnHandler;
use crate::shared::paths::*;

use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_on_attribute(handler: ParsedOnHandler) -> TokenStream {
    let ParsedOnHandler {
        function,
        fn_name,
        controller_ty,
        namespace,
        event_name,
        event_kind,
        registration_name,
        handler_struct_name,
    } = handler;

    quote! {
        #function

        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            #[doc(hidden)]
            pub struct #handler_struct_name;

            impl #handler_struct_name {
                pub fn register_handler(
                    controller_any: #Arc<dyn #Any + Send + Sync>,
                    socket: #SocketRef,
                ) {
                    let controller = controller_any
                        .downcast::<#controller_ty>()
                        .unwrap_or_else(|_| {
                            #sword_error! {
                                title: "Failed to downcast Socket.IO controller type",
                                reason: #format!(
                                    "Expected controller type {} during handler registration",
                                    #stringify!(#controller_ty)
                                ),
                                context: {
                                    "controller" => #stringify!(#controller_ty),
                                },
                                hints: ["This indicates an internal macro invariant violation"],
                            }
                        });

                    socket.on(#event_name, move |ctx: #SocketContext| {
                        let controller = #Arc::clone(&controller);

                        async move {
                            controller.#fn_name(ctx).await;
                        }
                    });
                }

                pub fn call_handler(
                    controller_any: #Arc<dyn #Any + Send + Sync>,
                    ctx: #SocketContext,
                ) -> #SocketBoxedFuture {
                    let controller = controller_any
                        .downcast::<#controller_ty>()
                        .unwrap_or_else(|_| {
                            #sword_error! {
                                title: "Failed to downcast Socket.IO controller type",
                                reason: #format!(
                                    "Type mismatch while executing handler for {}",
                                    #stringify!(#controller_ty)
                                ),
                                context: {
                                    "controller" => #stringify!(#controller_ty),
                                },
                                hints: ["This indicates an internal macro invariant violation"],
                            }
                        });

                    #Box::pin(async move {
                        controller.#fn_name(ctx).await;
                    })
                }
            }

            #inventory::submit! {
                #HandlerRegistrar {
                    controller_type_id: #TypeId::of::<#controller_ty>(),
                    namespace: #namespace,
                    event_kind: #event_kind,
                    method_name: #stringify!(#fn_name),
                    register_fn: #handler_struct_name::register_handler,
                    call_fn: #handler_struct_name::call_handler,
                }
            }
        };
    }
}
