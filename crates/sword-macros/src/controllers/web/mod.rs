pub mod attributes;
mod interceptor;

use super::shared::{CMetaStack, ControllerStruct};
use crate::controllers::shared::ParsedControllerKind;
use crate::shared::{gen_build, gen_clone, gen_deps};

use proc_macro::TokenStream;
use quote::quote;
use syn::Error;

pub use interceptor::*;

pub fn expand_web_controller(input: &ControllerStruct) -> syn::Result<TokenStream> {
    let ParsedControllerKind::Web { path } = &input.kind else {
        return Err(Error::new(input.name.span(), "Expected a web controller"));
    };

    let ControllerStruct {
        name: self_name,
        fields: self_fields,
        interceptors: controller_interceptors,
        ..
    } = input;

    let serialized_controller_interceptors: Vec<_> = controller_interceptors
        .iter()
        .map(|interceptor| interceptor.to_token_stream().to_string())
        .collect();

    CMetaStack::push("controller_kind", "web");
    CMetaStack::push("controller_path", path);
    CMetaStack::push("controller_name", &self_name.to_string());
    CMetaStack::push_list(
        "controller_interceptors",
        serialized_controller_interceptors,
    );

    let deps_impl = gen_deps(self_name, self_fields);
    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    let builder = quote! {
        #build_impl
        #deps_impl
        #clone_impl

        ::sword::internal::inventory::submit! {
            ::sword::internal::web::WebControllerRegistrar {
                controller_id: ::std::any::TypeId::of::<#self_name>(),
                controller_path: #path,
                build: |state: &::sword::internal::core::State| {
                    state.insert::<#self_name>(#self_name::build(state).unwrap_or_else(|e| {
                        ::sword::internal::core::sword_error! {
                            title: "Failed to build controller",
                            reason: "An error occurred while building the web controller",
                            context: {
                                "controller_name" => stringify!(#self_name),
                                "error" => format!("{e:?}"),
                                "source" => "WebControllerRegistrar::build",
                            },
                            hints: ["Check the error message for details on what went wrong during construction"],
                        }
                    }));
                },
            }
        }

        impl ::sword::internal::web::WebController for #self_name {
            fn base_path() -> &'static str {
                #path
            }
        }

        impl ::sword::internal::core::ControllerSpec for #self_name {
            fn kind() -> ::sword::internal::core::Controller {
                ::sword::internal::core::Controller::Web
            }

            fn type_id() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<#self_name>()
            }
        }
    };

    let expanded = quote! {
        #builder
    };

    Ok(TokenStream::from(expanded))
}
