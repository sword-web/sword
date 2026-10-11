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
        paths::*,
        traits::{generate_build_impl, generate_clone_impl},
    },
};

use darling::Error;
use proc_macro2::TokenStream;
use quote::quote;

pub fn expand_controller(controller: &ControllerStruct) -> Result<TokenStream> {
    let ControllerKind::Web { path } = &controller.kind else {
        return Err(
            Error::custom("Expected a web controller").with_span(&controller.base.ident.span())
        );
    };

    let self_name = &controller.base.ident;
    let interceptors = controller.interceptors.clone();
    let fields = controller.base.fields();

    let serialized_controller_interceptors: Vec<_> = interceptors
        .into_iter()
        .map(|interceptor| TokenStream::from(interceptor).to_string())
        .collect();

    CMetaStack::push("web", "controller_path", path);
    CMetaStack::push("web", "controller_name", &self_name.to_string());
    CMetaStack::push_list(
        "web",
        "controller_interceptors",
        serialized_controller_interceptors,
    );

    let build_impl = generate_build_impl(self_name, &fields);
    let clone_impl = generate_clone_impl(self_name, &fields);

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl #WebController for #self_name {
            fn base_path() -> &'static str {
                #path
            }
        }

        impl #ControllerSpec for #self_name {
            fn kind() -> #Controller {
                #Controller::Web
            }
        }

        #inventory::submit! {
            #WebControllerRegistrar {
                controller_id: #TypeId::of::<#self_name>(),
                controller_path: #path,
                build: |state: &#State| {
                    let controller = #self_name::build(state).unwrap_or_else(|e| {
                        #sword_error! {
                            title: "Failed to build controller",
                            reason: format!("Failed to build web controller dependency graph: {e}"),
                            context: {
                                "controller_name" => stringify!(#self_name),
                                "error" => format!("{e:?}"),
                                "source" => "WebControllerRegistrar::build",
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                        }
                    })

                    state.insert::<#self_name>(controller);
                }
            }
        }
    };

    Ok(expanded)
}
