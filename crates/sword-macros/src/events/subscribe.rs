use crate::shared::{Result, cmeta::CMetaStack, paths::*};

use darling::Error;
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, LitStr, Type};

pub fn expand_subscribe(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Handshake: leave the method untouched outside an event handler controller.
    let Some(controller_ty_str) = CMetaStack::get("event_controller", "controller_ty") else {
        return Ok(item);
    };

    let event_lit = syn::parse::<LitStr>(attr)?;
    let event_key = event_lit.value();

    if event_key.is_empty() {
        return Err(Error::custom(
            "event key must not be empty; specify the full key, e.g. #[handle(\"user.created\")]",
        )
        .with_span(&event_lit));
    }

    let item_fn = syn::parse::<ItemFn>(item)?;
    let event_type = extract_event_type(&item_fn)?;

    let controller_ty: syn::Ident = syn::parse_str(&controller_ty_str)?;
    let fn_name = &item_fn.sig.ident;

    let registration_name = format_ident!(
        "__SWORD_EVENT_HANDLER_{}_{}",
        controller_ty_str.replace("::", "_"),
        fn_name
    );
    let build_fn_name = format_ident!(
        "__sword_event_build_{}_{}",
        controller_ty.to_string().to_snake_case(),
        fn_name
    );

    let expanded = quote! {
        #item_fn

        #[allow(non_upper_case_globals)]
        #[doc(hidden)]
        const #registration_name: () = {
            fn #build_fn_name(state: &#State) -> #EventHandlerFn {
                let controller: #Arc<#controller_ty> =
                    state.borrow::<#controller_ty>().unwrap_or_else(|_| {
                        #sword_error! {
                            title: "Failed to retrieve event handler controller from state",
                            reason: #format!(
                                "Controller {} was not built for event handler",
                                #stringify!(#controller_ty)
                            ),
                            context: {
                                "controller" => #stringify!(#controller_ty),
                                "event_key" => #event_key,
                            },
                            hints: [
                                "Ensure the controller is registered via Module::register_controllers",
                            ],
                        }
                    });

                #Arc::new(
                    move |event: #Arc<dyn #Event>| -> #EventsBoxedFuture {
                        let controller = #Arc::clone(&controller);

                        #Box::pin(async move {
                            let typed = event.downcast_ref::<#event_type>().ok_or_else(|| {
                                #format!(
                                    "Failed to downcast event to {} for handler {} (key: {})",
                                    #type_name::<#event_type>(),
                                    #stringify!(#fn_name),
                                    #event_key,
                                )
                            })?;

                            controller.#fn_name(#Clone::clone(typed)).await
                        })
                    },
                )
            }

            #inventory::submit! {
                #EventHandlerRegistrar {
                    event_key: #event_key,
                    handler_type_id: #TypeId::of::<#controller_ty>(),
                    build_and_handle: #build_fn_name,
                }
            }
        };
    };

    Ok(expanded.into())
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

    let typed_params: Vec<&FnArg> = input_fn
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
