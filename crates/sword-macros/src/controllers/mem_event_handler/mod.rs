mod handle;

use super::shared::{CMetaStack, ControllerStruct, ParsedControllerKind};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::Error;

pub use handle::expand_handle;

pub fn expand_mem_event_handler(input: &ControllerStruct) -> syn::Result<TokenStream> {
    let ParsedControllerKind::MemEventHandler { namespace } = &input.kind else {
        return Err(Error::new_spanned(
            &input.name,
            "Expected a MemEventHandler controller struct",
        ));
    };

    let self_name = &input.name;
    let self_fields = &input.fields;
    let controller_name_str = self_name.to_string();

    let deps_impl = crate::shared::gen_deps(self_name, self_fields);
    let build_impl = crate::shared::gen_build(self_name, self_fields);
    let clone_impl = crate::shared::gen_clone(self_name, self_fields);

    CMetaStack::push("controller_kind", "mem_event_handler");
    CMetaStack::push("controller_name", &controller_name_str);
    CMetaStack::push("mem_event_namespace", namespace);

    let expanded: TokenStream2 = quote! {
        #build_impl
        #deps_impl
        #clone_impl

        impl ::sword::internal::events::MemEventHandler for #self_name {
            fn namespace() -> &'static str {
                #namespace
            }
        }

        impl ::sword::internal::core::ControllerSpec for #self_name {
            fn kind() -> ::sword::internal::core::Controller {
                ::sword::internal::core::Controller::MemEventHandler
            }

            fn type_id() -> ::std::any::TypeId {
                ::std::any::TypeId::of::<Self>()
            }
        }

        const _: () = {
            ::sword::internal::inventory::submit! {
                ::sword::internal::events::MemEventControllerRegistrar {
                    handler_type_id: ::std::any::TypeId::of::<#self_name>(),
                    build: |state: &::sword::internal::core::State| {
                        let controller = <#self_name as ::sword::internal::core::Build>::build(state)
                            .unwrap_or_else(|err| {
                                ::sword::internal::core::sword_error! {
                                    title: "Failed to build MemEventHandler controller",
                                    reason: err,
                                    context: {
                                        "controller" => #controller_name_str,
                                    },
                                    hints: [
                                        "Ensure all controller dependencies are registered as providers or components",
                                    ],
                                }
                            });

                        state.insert::<#self_name>(controller);
                    },
                }
            }
        };
    };

    Ok(TokenStream::from(expanded))
}
