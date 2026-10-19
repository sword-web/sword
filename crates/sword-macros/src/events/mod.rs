mod event;
mod subscribe;

pub use event::expand_event_struct;
pub use subscribe::expand_subscribe;

use crate::core::{ControllerKind, ControllerStruct};
use crate::shared::{
    Result,
    cmeta::CMetaStack,
    paths::*,
    traits::{generate_build_impl, generate_clone_impl},
};

use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_controller(controller: &ControllerStruct) -> Result<TokenStream> {
    let ControllerKind::Events { source } = &controller.kind else {
        return Err(
            Error::custom("Expected an event handler controller").with_span(controller.ty())
        );
    };

    let self_ty = controller.ty();
    let fields = controller.fields();

    CMetaStack::push("event_controller", "controller_ty", &self_ty.to_string());
    CMetaStack::push(
        "event_controller",
        "source",
        source.as_ref().map(EventSourceKind::name).unwrap_or(""),
    );

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::Events
            }
        }

        #inventory::submit! {
            #EventControllerRegistrar {
                ty: #TypeId::of::<#self_ty>(),
                build: |state: &#State| {
                    let controller = #self_ty::build(state).unwrap_or_else(|e| {
                        #sword_error! {
                            title: "Failed to build controller",
                            reason: #format!("Failed to build event controller dependency graph: {e}"),
                            context: {
                                "controller_name" => #stringify!(#self_ty),
                                "error" => #format!("{e:?}"),
                                "source" => "EventControllerRegistrar::build",
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                        }
                    });

                    state.insert::<#self_ty>(controller);
                },
            }
        }
    };

    Ok(expanded)
}

pub enum EventSourceKind {
    Memory,
    Nats,
}

impl EventSourceKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Memory => "Memory",
            Self::Nats => "Nats",
        }
    }

    pub fn from_name(name: &str) -> Result<Self> {
        match name {
            "Memory" => Ok(Self::Memory),
            "Nats" => Ok(Self::Nats),
            _ => Err(Error::custom(format!(
                "Invalid event source `{name}`. Expected 'Memory' or 'Nats'."
            ))),
        }
    }

    pub fn as_tokens(&self) -> TokenStream {
        match self {
            Self::Memory => quote! { #EventSource::Memory },
            Self::Nats => quote! { #EventSource::Nats },
        }
    }

    pub fn from_path(path: &syn::Path) -> Result<Self> {
        let name = path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
            .unwrap_or_default();

        match name.as_str() {
            "Memory" => Ok(Self::Memory),
            "Nats" => Ok(Self::Nats),
            _ => Err(Error::custom(format!(
                "Invalid event source type: {name}. Expected 'Memory' or 'Nats'."
            ))),
        }
    }
}
