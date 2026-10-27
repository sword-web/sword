pub mod schedule;

use crate::core::{ControllerKind, ControllerStruct};
use crate::shared::{
    Result,
    cmeta::CMetaStack,
    location,
    paths::*,
    traits::{generate_build_impl, generate_clone_impl},
};

use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_controller(controller: &ControllerStruct) -> Result<TokenStream> {
    let ControllerKind::Jobs = &controller.kind else {
        return Err(Error::custom("Expected a job controller").with_span(&controller.span()));
    };

    let self_ty = &controller.ty();
    let location = location::tokens(self_ty.span(), &self_ty.to_string());
    let fields = controller.fields();

    CMetaStack::push("job_controller", "controller_ty", &controller.name());

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::Jobs
            }
        }

        #inventory::submit! {
            #JobControllerRegistrar {
                type_id: #TypeId::of::<#self_ty>(),
                build_fn: |state: &#State| -> ::std::result::Result<(), #DiagnosticError> {
                    let controller = #self_ty::build(state).map_err(|e| {
                        #sword_error! {
                            code: "sword.build.jobs",
                            phase: #DiagnosticPhase::Build,
                            target: "job controller",
                            title: "Failed to build controller",
                            reason: format!("Failed to build job controller dependency graph: {e}"),
                            source_location: {
                                file: (#location).file(),
                                line: (#location).line(),
                                column: (#location).column(),
                                length: (#location).length(),
                                message: format!("`{}` could not be built", stringify!(#self_ty)),
                            },
                            context: {
                                "controller_name" => stringify!(#self_ty),
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                            cause: e,
                        }
                    })?;

                    state.insert::<#self_ty>(controller);

                    Ok(())
                }
            }
        }
    };

    Ok(expanded)
}
