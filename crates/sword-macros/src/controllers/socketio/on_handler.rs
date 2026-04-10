use crate::controllers::shared::CMetaStack;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{ItemFn, LitStr};

pub fn expand_on_handler(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let event_lit = syn::parse::<LitStr>(attr)?;
    let event_name = event_lit.value();
    let input_fn = syn::parse::<ItemFn>(item)?;

    let controller_name = CMetaStack::get("socketio_controller_name")
        .ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "#[on] must be used inside an impl block for a struct with #[controller(kind = Controller::SocketIo, ...)]",
            )
        })?;

    let namespace = CMetaStack::get("socketio_namespace").ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "socketio_namespace not found in CMetaStack",
        )
    })?;

    let fn_name = &input_fn.sig.ident;
    let controller_ident: syn::Ident = syn::parse_str(&controller_name)?;

    let event_kind = match event_name.as_str() {
        "connection" => {
            quote! { ::sword::internal::socketio::SocketEventKind::Connection }
        }
        "disconnection" => {
            quote! { ::sword::internal::socketio::SocketEventKind::Disconnection }
        }
        "fallback" => {
            quote! { ::sword::internal::socketio::SocketEventKind::Fallback }
        }
        custom => {
            quote! { ::sword::internal::socketio::SocketEventKind::Message(#custom) }
        }
    };

    let registration_name = format_ident!(
        "__SWORD_SOCKETIO_HANDLER_{}_{}",
        controller_name.replace("::", "_"),
        fn_name
    );

    let fn_name_str = fn_name.to_string();
    let fn_name_pascal = fn_name_str
        .split('_')
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<String>();

    let handler_struct_name = format_ident!(
        "SwordSocketIoHandler{}_{}",
        controller_name.replace("::", ""),
        fn_name_pascal
    );

    let handler_impl = quote! {
        pub fn register_handler(
            controller_any: ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>,
            socket: ::sword::socketio::SocketRef,
        ) {
            let controller = controller_any
                .downcast::<#controller_ident>()
                .unwrap_or_else(|_| {
                    ::sword::internal::core::sword_error!(
                        title: "Failed to downcast Socket.IO controller type",
                        reason: format!(
                            "Expected controller type {} during handler registration",
                            stringify!(#controller_ident)
                        ),
                        context: {
                            "controller" => stringify!(#controller_ident),
                        },
                        hints: ["This indicates an internal macro invariant violation"],
                    )
                });

            socket.on(#event_name, move |ctx: ::sword::socketio::SocketContext| {
                let controller = ::std::sync::Arc::clone(&controller);
                async move {
                    controller.#fn_name(ctx).await;
                }
            });
        }

        pub fn call_handler(
            controller_any: ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>,
            ctx: ::sword::socketio::SocketContext,
        ) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ()> + Send>> {
            let controller = controller_any
                .downcast::<#controller_ident>()
                .unwrap_or_else(|_| {
                    ::sword::internal::core::sword_error!(
                        title: "Failed to downcast Socket.IO controller type",
                        reason: format!(
                            "Type mismatch while executing handler for {}",
                            stringify!(#controller_ident)
                        ),
                        context: {
                            "controller" => stringify!(#controller_ident),
                        },
                        hints: ["This indicates an internal macro invariant violation"],
                    )
                });

            ::std::boxed::Box::pin(async move {
                controller.#fn_name(ctx).await;
            })
        }
    };

    let inventory_registration = quote! {
        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            #[doc(hidden)]
            pub struct #handler_struct_name;

            impl #handler_struct_name {
                #handler_impl
            }

                ::sword::internal::inventory::submit! {
                ::sword::internal::socketio::HandlerRegistrar {
                    controller_type_id: ::std::any::TypeId::of::<#controller_ident>(),
                    namespace: #namespace,
                    event_kind: #event_kind,
                    method_name: stringify!(#fn_name),
                    register_fn: #handler_struct_name::register_handler,
                    call_fn: #handler_struct_name::call_handler,
                }
            }
        };
    };

    let expanded = quote! {
        #input_fn
        #inventory_registration
    };

    Ok(TokenStream::from(expanded))
}
