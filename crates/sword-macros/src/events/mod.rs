mod event;
mod subscribe;

pub use event::expand_event_struct;
pub use subscribe::expand_subscribe;

use crate::core::{ControllerKind, ControllerStruct, EventSourceKind};
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
                build: |state: &#State| -> ::std::result::Result<(), #DiagnosticError> {
                    let controller = #self_ty::build(state).map_err(|e| {
                        #sword_diagnostic! {
                            code: "sword.build.events",
                            phase: #DiagnosticPhase::Build,
                            target: "event controller",
                            title: "Failed to build controller",
                            reason: format!("Failed to build event controller dependency graph: {e}"),
                            context: {
                                "controller_name" => #stringify!(#self_ty),
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                            cause: e,
                        }
                    })?;

                    state.insert::<#self_ty>(controller);

                    Ok(())
                },
            }
        }
    };

    Ok(expanded)
}
