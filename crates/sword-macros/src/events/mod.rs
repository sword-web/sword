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
    let ControllerKind::EventHandler { source } = &controller.kind else {
        return Err(
            Error::custom("Expected an event handler controller").with_span(controller.ty())
        );
    };

    let self_ty = controller.ty();
    let fields = controller.fields();

    CMetaStack::push("event_controller", "controller_ty", &self_ty.to_string());

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    let source_tokens = source.as_tokens();

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #EventHandler for #self_ty {}

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::EventHandler
            }
        }

        #inventory::submit! {
            #EventControllerRegistrar {
                handler_type_id: #TypeId::of::<#self_ty>(),
                source: #source_tokens,
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
