use crate::{
    controllers::shared::{ControllerStruct, ParsedControllerKind},
    shared::{gen_build, gen_clone, gen_deps},
};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, Path};

pub fn generate_socketio_controller_builder(
    input: &ControllerStruct,
    interceptors: &[Path],
) -> syn::Result<TokenStream> {
    let ParsedControllerKind::SocketIo { namespace } = &input.kind else {
        return Err(Error::new_spanned(
            &input.name,
            "Expected a Socket.IO controller struct",
        ));
    };

    let self_name = &input.name;
    let self_fields = &input.fields;
    let controller_name_str = self_name.to_string();

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    let interceptor_applications = interceptors.iter().rev().map(|interceptor_path| {
        quote! {
            let interceptor = state.borrow::<#interceptor_path>()
                .unwrap_or_else(|err| {
                    ::sword::internal::core::sword_error!(
                        title: "Failed to retrieve Socket.IO interceptor from State",
                        reason: err,
                        context: {
                            "interceptor" => stringify!(#interceptor_path),
                        },
                        hints: ["Ensure the interceptor is registered and built before controller setup"],
                    )
                });

            let handler = handler.with(move |ctx: ::sword::socketio::SocketContext| {
                let interceptor = ::std::sync::Arc::clone(&interceptor);
                async move {
                    <#interceptor_path as ::sword::socketio::OnConnect>::on_connect(&*interceptor, ctx)
                        .await
                        .map_err(|e| ::std::boxed::Box::new(e) as ::std::boxed::Box<dyn ::std::fmt::Display + Send>)
                }
            });
        }
    });

    let setup_impl = quote! {
        #[doc(hidden)]
        pub fn __socketio_setup(state: &::sword::internal::core::State) {
            use ::sword::internal::socketio::ConnectHandler;

            let controller = ::std::sync::Arc::new(
                <#self_name as ::sword::internal::core::Build>::build(state).unwrap_or_else(|err| {
                    ::sword::internal::core::sword_error! {
                        title: "Failed to build Socket.IO controller",
                        reason: err,
                        context: {
                            "controller" => #controller_name_str,
                        },
                        hints: ["Ensure all controller dependencies are registered as providers or components"],
                    }
                })
            );

            let io = <::sword::socketio::SocketIo as ::sword::internal::core::FromState>::from_state(state)
                .unwrap_or_else(|err| {
                    ::sword::internal::core::sword_error! {
                        title: "Socket.IO component not found in application state",
                        reason: err,
                        context: {
                            "controller" => #controller_name_str,
                        },
                        hints: [
                            "Enable the `socketio-controllers` feature in Cargo.toml",
                            "Configure the socketio server section in your configuration file",
                        ],
                    }
                });

            let controller_type_id = ::std::any::TypeId::of::<#self_name>();
            let mut connection_handler: ::std::option::Option<::sword::internal::socketio::HandlerRegistrar> = None;
            let mut message_handlers = ::std::vec::Vec::new();

            for handler_meta in ::sword::internal::inventory::iter::<::sword::internal::socketio::HandlerRegistrar>() {
                if handler_meta.controller_type_id != controller_type_id {
                    continue;
                }

                match handler_meta.event_kind {
                    ::sword::internal::socketio::SocketEventKind::Connection => {
                        if connection_handler.is_some() {
                            ::sword::internal::core::sword_error!(
                                title: "Multiple connection handlers found in Socket.IO controller",
                                reason: "Only one #[on(\"connection\")] handler is allowed per controller",
                                context: {
                                    "controller" => #controller_name_str,
                                },
                            );
                        }
                        connection_handler = Some(handler_meta.clone());
                    },
                    ::sword::internal::socketio::SocketEventKind::Message(_)
                        | ::sword::internal::socketio::SocketEventKind::Disconnection
                        | ::sword::internal::socketio::SocketEventKind::Fallback => {
                        message_handlers.push(handler_meta.clone());
                    }
                }
            }

            let message_handlers: ::std::sync::Arc<[::sword::internal::socketio::HandlerRegistrar]> =
                ::std::sync::Arc::from(message_handlers.into_boxed_slice());

            let base_handler = move |ctx: ::sword::socketio::SocketContext| -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + ::std::marker::Send>> {
                let controller = controller.clone();
                let socket = ctx.socket.clone();
                let connection_handler = connection_handler.clone();
                let message_handlers = message_handlers.clone();

                ::std::boxed::Box::pin(async move {
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
        const _: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::socketio::SocketIoHandlerRegistrar {
                    handler_type_id: ::std::any::TypeId::of::<#self_name>(),
                    handler_type_name: stringify!(#self_name),
                    setup_fn: #self_name::__socketio_setup,
                }
            }
        };
    };

    let expanded = quote! {
        #build_impl
        #deps_impl
        #clone_impl

        impl ::sword::internal::socketio::SocketIoController for #self_name {
            fn namespace() -> &'static str {
                #namespace
            }
        }

        impl ::sword::internal::core::ControllerSpec for #self_name {
            fn kind() -> ::sword::internal::core::Controller {
                ::sword::internal::core::Controller::SocketIo
            }

            fn type_id() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<Self>()
            }
        }

        impl #self_name {
            #setup_impl
        }

        #setup_registration
    };

    Ok(expanded)
}
