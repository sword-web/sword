pub mod schedule;

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
    let ControllerKind::JobHandler = &controller.kind else {
        return Err(
            Error::custom("Expected a web controller").with_span(&controller.base.ident.span())
        );
    };

    let self_ty = &controller.base.ident;
    let fields = controller.base.fields();

    CMetaStack::push("job_controller", "controller_ty", &self_ty.to_string());

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::JobHandler
            }
        }

        #inventory::submit! {
            #JobControllerRegistrar {
                type_id: #TypeId::of::<#self_ty>(),
                build_fn: |state: &#State| {
                    let controller = #self_ty::build(state).unwrap_or_else(|e| {
                        #sword_error! {
                            title: "Failed to build controller",
                            reason: format!("Failed to build job controller dependency graph: {e}"),
                            context: {
                                "controller_name" => stringify!(#self_ty),
                                "error" => format!("{e:?}"),
                                "source" => "JobControllerRegistrar::build",
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                        }
                    });

                    state.insert::<#self_ty>(controller);
                }
            }
        }
    };

    Ok(expanded)
}
