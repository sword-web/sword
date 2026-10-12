mod handlers;

pub use handlers::expand_on_handler;

use crate::{
    core::{ControllerKind, ControllerStruct},
    shared::{
        Result,
        cmeta::CMetaStack,
        paths::*,
        traits::{generate_build_impl, generate_clone_impl},
    },
};

use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Path;

pub fn expand_controller(controller: &ControllerStruct) -> Result<TokenStream> {
    let ControllerKind::SocketIo { namespace } = &controller.kind else {
        return Err(Error::custom("Expected a Socket.IO controller").with_span(&controller.span()));
    };

    let self_ty = controller.ty();
    let fields = controller.fields();

    let interceptors: Vec<Path> = controller
        .interceptors
        .iter()
        .filter_map(|interceptor| interceptor.path().cloned())
        .collect();

    CMetaStack::push("socketio", "controller_ty", &controller.name());
    CMetaStack::push("socketio", "namespace", namespace);

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    let interceptor_applications = interceptors.iter().rev().map(|interceptor_path| {
        quote! {
            let interceptor = state.borrow::<#interceptor_path>()
                .unwrap_or_else(|err| {
                    #sword_error! {
                        title: "Failed to retrieve Socket.IO interceptor from State",
                        reason: err,
                        context: {
                            "interceptor" => #stringify!(#interceptor_path),
                        },
                        hints: ["Ensure the interceptor is registered and built before controller setup"],
                    }
                });

            let handler = handler.with(move |ctx: #SocketContext| {
                let interceptor = #Arc::clone(&interceptor);

                async move {
                    <#interceptor_path as #OnConnect>::on_connect(&*interceptor, ctx)
                        .await
                        .map_err(|e| #Box::new(e) as #Box<dyn #Display + Send>)
                }
            });
        }
    });

    let setup_impl = quote! {
        #[doc(hidden)]
        pub fn __socketio_setup(state: &#State) {
            use #ConnectHandler;

            let controller = #Arc::new(
                <#self_ty as #Build>::build(state).unwrap_or_else(|err| {
                    #sword_error! {
                        title: "Failed to build Socket.IO controller",
                        reason: err,
                        context: {
                            "controller" => #stringify!(#self_ty),
                        },
                        hints: ["Ensure all controller dependencies are registered as providers or components"],
                    }
                })
            );

            let io = state.get::<#SocketIo>()
                .unwrap_or_else(|err| {
                    #sword_error! {
                        title: "Socket.IO component not found in application state",
                        reason: err,
                        context: {
                            "controller" => #stringify!(#self_ty),
                        },
                        hints: [
                            "Enable the `socketio` feature in Cargo.toml",
                            "Configure the socketio server section in your configuration file",
                        ],
                    }
                });

            let controller_type_id = #TypeId::of::<#self_ty>();
            let mut connection_handler: Option<#HandlerRegistrar> = None;
            let mut message_handlers = Vec::new();

            for handler_meta in #inventory::iter::<#HandlerRegistrar>() {
                if handler_meta.controller_type_id != controller_type_id {
                    continue;
                }

                match handler_meta.event_kind {
                    #SocketEventKind::Connection => {
                        if connection_handler.is_some() {
                            #sword_error! {
                                title: "Multiple connection handlers found in Socket.IO controller",
                                reason: "Only one #[on(\"connection\")] handler is allowed per controller",
                                context: {
                                    "controller" => #stringify!(#self_ty),
                                },
                            }
                        }

                        connection_handler = Some(handler_meta.clone());
                    },
                    #SocketEventKind::Message(_)
                        | #SocketEventKind::Disconnection
                        | #SocketEventKind::Fallback => {
                        message_handlers.push(handler_meta.clone());
                    }
                }
            }

            let message_handlers: #Arc<[#HandlerRegistrar]> =
                #Arc::from(message_handlers.into_boxed_slice());

            let base_handler = move |ctx: #SocketContext| -> #SocketBoxedFuture {
                let socket = ctx.socket_ref().clone();

                #Box::pin(async move {
                    if let Some(handler) = connection_handler {
                        (handler.call_fn)(controller.clone(), ctx).await;
                    }

                    for handler in message_handlers.iter() {
                        (handler.register_fn)(controller.clone(), socket.clone());
                    }
                })
            };

            let handler = base_handler;
            #(#interceptor_applications)*

            io.ns(#namespace, handler);
        }
    };

    let setup_registration = quote! {
        #inventory::submit! {
            #SocketIoHandlerRegistrar {
                handler_type_id: #TypeId::of::<#self_ty>(),
                handler_type_name: #stringify!(#self_ty),
                setup_fn: #self_ty::__socketio_setup,
            }
        }
    };

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #SocketIoController for #self_ty {
            fn namespace() -> &'static str {
                #namespace
            }
        }

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::SocketIo
            }
        }

        impl #self_ty {
            #setup_impl
        }

        #setup_registration
    };

    Ok(expanded)
}
