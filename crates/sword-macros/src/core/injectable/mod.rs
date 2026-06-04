mod parse;

use crate::shared::*;
use parse::*;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::ItemStruct;

pub fn expand_injectable(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<ItemStruct>(item.clone())?;
    let parsed = parse_injectable_input(attr, item)?;

    let injectable_impl = match parsed.kind {
        InjectableKind::Provider => generate_provider_trait(&parsed),
        InjectableKind::Component => generate_component_trait(&parsed),
    };

    let clone_impl = gen_clone(&parsed.struct_name, &parsed.fields);

    let trait_binding = parsed.trait_as.as_ref().map(|trait_type| {
        let struct_name = &parsed.struct_name;
        quote! {
            ::sword::internal::inventory::submit! {
                ::sword::internal::core::TraitBindingRegistrar {
                    register: |container: &::sword::internal::core::DependencyContainer| {
                        container.register_trait_binding(
                            ::std::any::TypeId::of::<::sword::internal::core::InjectableTrait::<#trait_type>>(),
                            ::std::any::TypeId::of::<#struct_name>(),
                            Box::new(|state: &::sword::internal::core::State| {
                                let concrete: ::std::sync::Arc<#struct_name> = state.borrow::<#struct_name>()?;
                                let trait_obj: ::std::sync::Arc<#trait_type> = concrete;
                                Ok(::std::sync::Arc::new(
                                    ::sword::internal::core::InjectableTrait(trait_obj)
                                ) as ::std::sync::Arc<dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync>)
                            }),
                        );
                    },
                }
            }
        }
    });

    let mut expanded = quote! {
        #input
        #injectable_impl
    };

    if parsed.derive_clone {
        expanded.extend(quote! {
            #clone_impl
        });
    }

    if let Some(trait_binding) = trait_binding {
        expanded.extend(trait_binding);
    }

    Ok(expanded.into())
}

pub fn generate_component_trait(input: &InjectableInput) -> TokenStream2 {
    let struct_name = &input.struct_name;

    let deps_impl = gen_deps(struct_name, &input.fields);
    let build_impl = gen_build(struct_name, &input.fields);

    quote! {
        #build_impl
        #deps_impl
        impl ::sword::internal::core::Component for #struct_name {}
    }
}

pub fn generate_provider_trait(parsed: &InjectableInput) -> TokenStream2 {
    let struct_name = &parsed.struct_name;

    quote! {
        impl ::sword::internal::core::Provider for #struct_name {}
    }
}
