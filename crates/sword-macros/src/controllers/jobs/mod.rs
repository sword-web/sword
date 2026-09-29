mod handle;

pub use handle::{ScheduleKind, expand_handle};

use crate::controllers::shared::{CMetaStack, ControllerStruct, ParsedControllerKind};
use proc_macro::TokenStream;
use quote::quote;
use syn::Error;

pub fn expand_jobs_controller(input: &ControllerStruct) -> syn::Result<TokenStream> {
    let ParsedControllerKind::JobHandler = &input.kind else {
        return Err(Error::new_spanned(
            &input.name,
            "Expected Job Controller struct",
        ));
    };

    let self_name = &input.name;
    let self_fields = &input.fields;
    let controller_name_str = input.name.to_string();

    let build_impl = crate::shared::gen_build(self_name, self_fields);
    let clone_impl = crate::shared::gen_clone(self_name, self_fields);

    CMetaStack::push("job_controller", "controller_name", &controller_name_str);

    let expanded = quote! {
        #build_impl
        #clone_impl

        impl ::sword::internal::core::ControllerSpec for #self_name {
            fn kind() -> ::sword::internal::core::Controller {
                ::sword::internal::core::Controller::JobHandler
            }
        }

        ::sword::internal::inventory::submit! {
            ::sword::internal::jobs::JobControllerRegistrar {
                type_id: ::std::any::TypeId::of::<#self_name>(),
                build_fn: |state: &::sword::internal::core::State| {
                    let controller = <#self_name as ::sword::internal::core::Build>::build(state)
                        .unwrap_or_else(|e| {
                            ::sword::internal::core::sword_error! {
                                title: "Failed to build controller",
                                reason: format!("Failed to build job controller dependency graph: {e}"),
                                context: {
                                    "controller_name" => stringify!(#self_name),
                                    "error" => format!("{e:?}"),
                                    "source" => "JobControllerRegistrar::build",
                                },
                                hints: ["Check the error message for details on what went wrong during construction"],
                            }
                        });

                    state.insert::<#self_name>(controller);
                },
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
