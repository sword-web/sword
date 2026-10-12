use crate::shared::paths::*;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Ident, Type};

/// Generates the implementation of the `Build` trait for a component.
///
/// This generator is used by (#[interceptor], #[injectable], etc.) macros
/// to create the `build()` method that constructs an instance from the `State`.
///
/// The generated code extracts each field dependency from the `State` using
/// `State::get` (by-value fields) or `State::borrow` (`Arc<T>` fields) and
/// assembles them into a new component instance.
pub fn generate_build_impl(name: &Ident, fields: &[(Ident, Type)]) -> TokenStream {
    let extracts = generate_field_extractions(fields);
    let assigns = generate_field_assignments(fields);

    quote! {
        impl #Build for #name {
            fn build(state: &#State) -> Result<Self, #DependencyInjectionError>
            where
                Self: Sized,
            {
                #extracts

                Ok(Self { #assigns })
            }
        }
    }
}

/// Generates the implementation of the Clone trait for a component.
///
/// This generator creates an explicit Clone implementation that clones each
/// field individually, useful for components with Arc-wrapped dependencies.
pub fn generate_clone_impl(name: &Ident, fields: &[(Ident, Type)]) -> TokenStream {
    let clones = fields.iter().map(|(field_name, _)| {
        quote! { #field_name: self.#field_name.clone() }
    });

    quote! {
        impl Clone for #name {
            fn clone(&self) -> Self {
                Self {
                    #(#clones),*
                }
            }
        }
    }
}

/// Generates the implementation of the `HasDeps` trait for a component.
///
/// This generator creates the `deps()` method that returns the `TypeId` of allS
/// component dependencies, allowing the DI system to resolve the correct
/// construction order using topological sorting.
pub fn generate_has_deps_impl(name: &Ident, fields: &[(Ident, Type)]) -> TokenStream {
    let dep_types = fields.iter().map(|(_, field_type)| {
        if let Some(inner_type) = extract_arc_inner_type(field_type) {
            quote! { #TypeId::of::<#inner_type>() }
        } else {
            quote! { #TypeId::of::<#field_type>() }
        }
    });

    quote! {
        impl #HasDeps for #name {
            fn deps() -> Vec<#TypeId> {
                vec![#(#dep_types),*]
            }
        }
    }
}

fn extract_arc_inner_type(ty: &Type) -> Option<&Type> {
    let Type::Path(type_path) = ty else {
        return None;
    };

    let path = &type_path.path;
    let last_segment = path.segments.last()?;

    if last_segment.ident != "Arc" {
        return None;
    }

    let segments: Vec<_> = path.segments.iter().collect();

    let is_std_arc_path = match segments.len() {
        1 => true,
        2 => segments[0].ident == "sync",
        3 => {
            let root = &segments[0].ident;
            let mid = &segments[1].ident;
            (root == "std" || root == "alloc") && mid == "sync"
        }
        _ => false,
    };

    if !is_std_arc_path {
        return None;
    }

    match &last_segment.arguments {
        syn::PathArguments::AngleBracketed(args) => args.args.first().and_then(|arg| match arg {
            syn::GenericArgument::Type(inner) => Some(inner),
            _ => None,
        }),
        _ => None,
    }
}

fn generate_field_extractions(fields: &[(Ident, Type)]) -> TokenStream {
    let extractions =
        fields.iter().map(
            |(field_name, field_type)| match extract_arc_inner_type(field_type) {
                Some(inner_type) => {
                    quote! {
                        let #field_name = state.borrow::<#inner_type>()?;
                    }
                }
                None => {
                    quote! {
                        let #field_name = state.get::<#field_type>()?;
                    }
                }
            },
        );

    quote! {
        #(#extractions)*
    }
}

fn generate_field_assignments(fields: &[(Ident, Type)]) -> TokenStream {
    let assignments = fields.iter().map(|(name, _)| {
        quote! { #name }
    });

    quote! {
        #(#assignments),*
    }
}
