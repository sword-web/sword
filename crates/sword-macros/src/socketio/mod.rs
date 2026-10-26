mod handlers;

pub use handlers::expand_on_handler;

use crate::{
    core::{ControllerKind, ControllerStruct},
    shared::{
        Result,
        cmeta::CMetaStack,
        location,
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
    let location = location::tokens(self_ty.span(), &self_ty.to_string());
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
            let interceptor = state.borrow::<#interceptor_path>().map_err(|err| {
                #sword_error! {
                    code: "sword.build.socketio",
                    phase: #DiagnosticPhase::Build,
                    target: "Socket.IO interceptor",
                    title: "Failed to retrieve Socket.IO interceptor from State",
                    reason: err,
                    context: {
                        "interceptor" => #stringify!(#interceptor_path),
                    },
                    hints: ["Ensure the interceptor is registered and built before controller setup"],
                    cause: err,
                }
            })?;

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
        pub fn __socketio_setup(state: &#State) -> ::std::result::Result<(), #DiagnosticError> {
            use #ConnectHandler;

            let controller = #Arc::new(<#self_ty as #Build>::build(state).map_err(|err| {
                #sword_error! {
                    code: "sword.build.socketio",
                    phase: #DiagnosticPhase::Build,
                    target: "Socket.IO controller",
                    title: "Failed to build Socket.IO controller",
                    reason: err,
                    source_location: {
                        file: (#location).file(),
                        line: (#location).line(),
                        column: (#location).column(),
                        length: (#location).length(),
                        message: format!("`{}` could not be built", stringify!(#self_ty)),
                    },
                    context: {
                        "controller" => #stringify!(#self_ty),
                    },
                    hints: ["Ensure all controller dependencies are registered as providers or components"],
                    cause: err,
                }
            })?);

            let io = state.get::<#SocketIo>().map_err(|err| {
                #sword_error! {
                    code: "sword.build.socketio",
                    phase: #DiagnosticPhase::Build,
                    target: "Socket.IO controller",
                    title: "Socket.IO component not found in application state",
                    reason: err,
                    source_location: {
                        file: (#location).file(),
                        line: (#location).line(),
                        column: (#location).column(),
                        length: (#location).length(),
                        message: format!("`{}` was not set up", stringify!(#self_ty)),
                    },
                    context: {
                        "controller" => #stringify!(#self_ty),
                    },
                    hints: [
                        "Enable the `socketio` feature in Cargo.toml",
                        "Configure the socketio server section in your configuration file",
                    ],
                    cause: err,
                }
            })?;

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
                            return Err(#sword_error! {
                                code: "sword.build.socketio",
                                phase: #DiagnosticPhase::Build,
                                target: "Socket.IO controller",
                                title: "Multiple connection handlers found in Socket.IO controller",
                                reason: "Only one #[on(\"connection\")] handler is allowed per controller",
                                source_location: {
                                    file: (#location).file(),
                                    line: (#location).line(),
                                    column: (#location).column(),
                                    length: (#location).length(),
                                    message: format!("`{}` declares two connection handlers", stringify!(#self_ty)),
                                },
                                context: {
                                    "controller" => #stringify!(#self_ty),
                                },
                            });
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
                    if let Some(handler) = connection_handler
                        && let Err(err) = (handler.call_fn)(controller.clone(), ctx).await
                    {
                        err.report();
                    }

                    for handler in message_handlers.iter() {
                        if let Err(err) = (handler.register_fn)(controller.clone(), socket.clone()) {
                            err.report();
                        }
                    }
                })
            };

            let handler = base_handler;
            #(#interceptor_applications)*

            io.ns(#namespace, handler);

            Ok(())
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
