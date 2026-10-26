mod errors;
mod interceptors;
mod routes;

pub use errors::derive_http_error;
pub use routes::expand_route;

use crate::{
    core::{ControllerKind, ControllerStruct},
    shared::{
        Result,
        cmeta::CMetaStack,
        location,
        paths::*,
        traits::{generate_build_impl, generate_clone_impl},
    },
};

use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_controller(controller: &ControllerStruct) -> Result<TokenStream> {
    let ControllerKind::Web { path } = &controller.kind else {
        return Err(Error::custom("Expected a web controller").with_span(&controller.span()));
    };

    let self_ty = controller.ty();
    let location = location::tokens(self_ty.span(), &self_ty.to_string());
    let interceptors = controller.interceptors.clone();
    let fields = controller.fields();

    let serialized_controller_interceptors: Vec<_> = interceptors
        .into_iter()
        .map(|interceptor| TokenStream::from(interceptor).to_string())
        .collect();

    CMetaStack::push("web", "controller_path", path);
    CMetaStack::push("web", "controller_name", &controller.name());
    CMetaStack::push_list(
        "web",
        "controller_interceptors",
        serialized_controller_interceptors,
    );

    let build_impl = generate_build_impl(self_ty, &fields);
    let clone_impl = generate_clone_impl(self_ty, &fields);

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #WebController for #self_ty {
            fn base_path() -> &'static str {
                #path
            }
        }

        impl #ControllerSpec for #self_ty {
            fn kind() -> #Controller {
                #Controller::Web
            }
        }

        #inventory::submit! {
            #WebControllerRegistrar {
                controller_id: #TypeId::of::<#self_ty>(),
                controller_path: #path,
                build: |state: &#State| -> ::std::result::Result<(), #DiagnosticError> {
                    let controller = #self_ty::build(state).map_err(|e| {
                        #sword_error! {
                            code: "sword.build.web",
                            phase: #DiagnosticPhase::Build,
                            target: "web controller",
                            title: "Failed to build controller",
                            reason: format!("Failed to build web controller dependency graph: {e}"),
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
