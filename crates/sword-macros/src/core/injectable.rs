use crate::shared::{
    Result, location,
    paths::*,
    structs::Struct,
    traits::{generate_build_impl, generate_clone_impl, generate_has_deps_impl},
};

use darling::{Error, FromDeriveInput, FromMeta, ast::NestedMeta, util::Flag};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::DeriveInput;

// `#[injectable] args section ================================================================

#[derive(Debug, FromMeta)]
#[darling(rename_all = "snake_case")]
pub enum InjectableKind {
    Provider,
    Component,
}

#[derive(FromMeta)]
pub struct Args {
    pub no_derive_clone: Flag,
    pub component: Flag,
    pub provider: Flag,
}

impl Args {
    pub fn kind(&self) -> InjectableKind {
        if self.provider.is_present() {
            InjectableKind::Provider
        } else {
            InjectableKind::Component
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.provider.is_present() && self.component.is_present() {
            return Err(Error::custom("cannot use both `provider` and `component`")
                .with_span(&self.component.span()));
        }

        Ok(())
    }

    pub fn derive_clone(&self) -> bool {
        !self.no_derive_clone.is_present()
    }
}

// `#[injectable] expand section ===============================================================

pub fn expand_injectable(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let derive_input = syn::parse::<DeriveInput>(item.clone())?;
    let parsed_struct = Struct::from_derive_input(&derive_input)?;

    let args = Args::from_list(&NestedMeta::parse_meta_list(attr.into())?)?;

    args.validate()?;

    let injectable_impl = match args.kind() {
        InjectableKind::Provider => generate_provider_trait(&parsed_struct),
        InjectableKind::Component => generate_component_trait(&parsed_struct),
    };

    let clone_impl = if args.derive_clone() {
        generate_clone_impl(&parsed_struct.ident, &parsed_struct.fields())
    } else {
        quote! {}
    };

    let struct_tokens: TokenStream2 = item.into();

    let expanded = quote! {
        #struct_tokens
        #injectable_impl
        #clone_impl
    };

    Ok(expanded.into())
}

fn generate_component_trait(item: &Struct) -> TokenStream2 {
    let struct_name = &item.ident;
    let location = location::tokens(struct_name.span(), &struct_name.to_string());

    let deps_impl = generate_has_deps_impl(struct_name, &item.fields());
    let build_impl = generate_build_impl(struct_name, &item.fields());

    quote! {
        #build_impl
        #deps_impl

        impl #Component for #struct_name {
            fn source_location() -> ::std::option::Option<#DiagnosticSourceLocation> {
                ::std::option::Option::Some(#location)
            }
        }
    }
}

fn generate_provider_trait(item: &Struct) -> TokenStream2 {
    let struct_name = &item.ident;

    quote! {
        impl #Provider for #struct_name {}
    }
}
