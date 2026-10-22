use crate::{
    core::EventSourceKind,
    shared::{Result, cmeta::CMetaStack, paths::*},
};

use darling::{Error, FromMeta, ast::NestedMeta};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{FnArg, ItemFn, Path, Type};

#[derive(FromMeta)]
struct Args {
    key: String,

    #[darling(default)]
    source: Option<Path>,

    #[darling(default)]
    group: Option<String>,
}

pub fn expand_subscribe(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Handshake: leave the method untouched outside an event handler controller.
    let Some(controller_ty_str) = CMetaStack::get("event_controller", "controller_ty") else {
        return Ok(item);
    };

    let args = Args::from_list(&NestedMeta::parse_meta_list(attr.into())?)?;
    let source = resolve_event_source(&args)?;

    let group_tokens = match &args.group {
        Some(group) => quote! { Some(#group) },
        None => quote! { None },
    };

    let source_tokens = source.as_tokens();
    let event_key = args.key;

    if event_key.is_empty() {
        return Err(Error::custom(
            "event key must not be empty; specify it like #[subscribe(key = \"user.created\")]",
        ));
    }

    let item_fn = syn::parse::<ItemFn>(item)?;
    let event_param = extract_event_param(&item_fn)?;

    let controller_ty: syn::Ident = syn::parse_str(&controller_ty_str)?;
    let fn_name = &item_fn.sig.ident;

    let body = match event_param {
        EventParam::Typed(boxed) => {
            let ty = boxed.as_ref().clone();
            quote! {
                let typed = event.cast::<#ty>().ok_or_else(|| {
                    #format!(
                        "Failed to downcast event to {} for handler {} (key: {})",
                        #type_name::<#ty>(),
                        #stringify!(#fn_name),
                        #event_key,
                    )
                })?;

                controller.#fn_name(typed).await
            }
        }
        EventParam::Dynamic => quote! {
            controller.#fn_name(&*event).await
        },
    };

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
                let controller = state.borrow::<#controller_ty>().unwrap_or_else(|_| {
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
                            #body
                        })
                    },
                )
            }

            #inventory::submit! {
                #EventHandlerRegistrar {
                    key: #event_key,
                    source: #source_tokens,
                    group: #group_tokens,
                    controller_ty: #TypeId::of::<#controller_ty>(),
                    handler_fn: #build_fn_name,
                }
            }
        };
    };

    Ok(expanded.into())
}

fn resolve_event_source(args: &Args) -> Result<EventSourceKind> {
    let controller_source = match CMetaStack::get("event_controller", "source").as_deref() {
        None | Some("") => None,
        Some(name) => Some(EventSourceKind::from_name(name)?),
    };

    let handler_source = args
        .source
        .as_ref()
        .map(EventSourceKind::from_path)
        .transpose()?;

    let source = match (handler_source, controller_source) {
        (Some(handler), None) => handler,
        (None, Some(controller)) => controller,
        (Some(_), Some(_)) => {
            return Err(Error::custom(
                "the controller already declares a `source`; don't declare it on the handler",
            ));
        }
        (None, None) => {
            return Err(Error::custom(
                "this handler must declare `source` because the controller does not",
            ));
        }
    };

    Ok(source)
}

enum EventParam {
    Typed(std::boxed::Box<Type>),
    Dynamic,
}

fn extract_event_param(input_fn: &ItemFn) -> syn::Result<EventParam> {
    let has_receiver = input_fn
        .sig
        .inputs
        .first()
        .is_some_and(|arg| matches!(arg, FnArg::Receiver(_)));

    if !has_receiver {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "Event handler must have `&self` as the first parameter, \
             e.g. `async fn handle(&self, event: &MyEvent) -> Result<()>`",
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
            "Event handler must have a parameter for the event, \
             e.g. `async fn handle(&self, event: &MyEvent) -> Result<()>`",
        )
    })?;

    match event_arg {
        FnArg::Typed(pat_type) => match &*pat_type.ty {
            // &mut T / &mut dyn Event
            Type::Reference(r) if r.mutability.is_some() => Err(syn::Error::new_spanned(
                &pat_type.ty,
                "event handler parameter must be `&T` or `&dyn Event`, not `&mut T`",
            )),
            // &dyn Event
            Type::Reference(r) if matches!(&*r.elem, Type::TraitObject(_)) => {
                Ok(EventParam::Dynamic)
            }
            // &T
            Type::Reference(r) => Ok(EventParam::Typed(std::boxed::Box::new((*r.elem).clone()))),
            // T por valor u otra forma
            _ => Err(syn::Error::new_spanned(
                &pat_type.ty,
                "event handler must take `&T` or `&dyn Event`, \
                 e.g. `async fn handle(&self, event: &MyEvent)`",
            )),
        },
        _ => unreachable!(),
    }
}
