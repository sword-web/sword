use crate::controllers::shared::CMetaStack;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, LitStr, Type};

pub fn expand_handle(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let event_lit = syn::parse::<LitStr>(attr)?;
    let relative_key = event_lit.value();
    let input_fn = syn::parse::<ItemFn>(item)?;

    let controller_name = CMetaStack::get("controller_name").ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "#[handle] must be used inside an impl block for a struct with \
             #[controller(kind = Controller::MemEventHandler, ...)]",
        )
    })?;

    let namespace = CMetaStack::get("mem_event_namespace").ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "mem_event_namespace not found in CMetaStack",
        )
    })?;

    let full_key = if relative_key.is_empty() {
        namespace.clone()
    } else {
        format!("{}.{}", namespace, relative_key)
    };

    let fn_name = &input_fn.sig.ident;
    let controller_ident: syn::Ident = syn::parse_str(&controller_name)?;

    let event_type = extract_event_type(&input_fn)?;

    let registration_name = format_ident!(
        "__SWORD_EVENT_HANDLER_{}_{}",
        controller_name.replace("::", "_"),
        fn_name
    );

    let controller_snake = controller_name
        .split("::")
        .map(|s| {
            let mut result = String::new();
            for (i, c) in s.chars().enumerate() {
                if c.is_uppercase() && i > 0 {
                    result.push('_');
                }
                result.push(c.to_ascii_lowercase());
            }
            result
        })
        .collect::<Vec<_>>()
        .join("__");

    let build_fn_name = format_ident!("__sword_event_build_{}_{}", controller_snake, fn_name);

    let expansion = quote! {
        #input_fn

        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            fn #build_fn_name(
                state: &::sword::internal::core::State,
            ) -> ::sword::internal::events::EventHandlerFn {
                let controller: ::std::sync::Arc<#controller_ident> =
                    state.borrow::<#controller_ident>().unwrap_or_else(|_| {
                        ::sword::internal::core::sword_error!(
                            title: "Failed to retrieve event handler controller from state",
                            reason: ::std::format!(
                                "Controller {} was not built for event handler",
                                ::std::stringify!(#controller_ident)
                            ),
                            context: {
                                "controller" => ::std::stringify!(#controller_ident),
                                "event_key" => #full_key,
                            },
                            hints: [
                                "Ensure the controller is registered via Module::register_controllers",
                            ],
                        )
                    });

                ::std::sync::Arc::new(
                    move |event: ::std::sync::Arc<dyn ::sword::internal::events::Event>| {
                        let controller = controller.clone();
                        let cloned = event.clone_event();

                        ::std::boxed::Box::pin(async move {
                            let typed = cloned.downcast_ref::<#event_type>().ok_or_else(|| {
                                ::std::format!(
                                    "Failed to downcast event to {} for handler {} (key: {})",
                                    ::std::any::type_name::<#event_type>(),
                                    ::std::stringify!(#fn_name),
                                    #full_key,
                                )
                            })?;

                            (controller.#fn_name(
                                ::std::clone::Clone::clone(typed),
                            )
                            .await)
                        })
                            as ::std::pin::Pin<
                                ::std::boxed::Box<
                                    dyn ::std::future::Future<
                                        Output = ::sword::internal::events::EventHandlerResult<
                                            (),
                                        >,
                                    > + ::std::marker::Send,
                                >,
                            >
                    },
                )
            }

            ::sword::internal::inventory::submit! {
                ::sword::internal::events::MemEventRouteRegistrar {
                    event_key: #full_key,
                    handler_type_id: ::std::any::TypeId::of::<#controller_ident>(),
                    build_and_handle: #build_fn_name,
                }
            }
        };
    };

    Ok(TokenStream::from(expansion))
}

fn extract_event_type(input_fn: &ItemFn) -> syn::Result<Type> {
    let has_receiver = input_fn
        .sig
        .inputs
        .first()
        .is_some_and(|arg| matches!(arg, FnArg::Receiver(_)));

    if !has_receiver {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "Event handler must have `&self` as the first parameter, \
             e.g. `async fn handle(&self, event: MyEvent) -> Result<()>`",
        ));
    }

    let typed_params: Vec<&syn::FnArg> = input_fn
        .sig
        .inputs
        .iter()
        .filter(|arg| matches!(arg, FnArg::Typed(_)))
        .collect();

    let event_arg = typed_params.first().ok_or_else(|| {
        syn::Error::new_spanned(
            &input_fn.sig,
            "Event handler must have a typed parameter for the event, \
             e.g. `async fn handle(&self, event: MyEvent) -> Result<()>`",
        )
    })?;

    match event_arg {
        FnArg::Typed(pat_type) => Ok(*pat_type.ty.clone()),
        _ => unreachable!(),
    }
}
