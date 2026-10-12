use crate::shared::{
    Result,
    paths::*,
    structs::Struct,
    traits::{generate_build_impl, generate_clone_impl, generate_has_deps_impl},
};

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    Attribute, DeriveInput, Expr, Ident, Path, Token,
    parse::{Parse, ParseStream, discouraged::Speculative},
};

// #[derive(Interceptor)] macro expansion ===========================================

pub fn expand_interceptor_derive(item: TokenStream) -> Result<TokenStream> {
    let derive_input = syn::parse::<DeriveInput>(item)?;
    let parsed_struct = Struct::from_derive_input(&derive_input)?;

    let self_name = &parsed_struct.ident;
    let self_fields = &parsed_struct.fields();

    let build_impl = generate_build_impl(self_name, self_fields);
    let clone_impl = generate_clone_impl(self_name, self_fields);
    let deps_impl = generate_has_deps_impl(self_name, self_fields);

    let expanded = quote! {
        #build_impl
        #clone_impl
        #deps_impl

        impl #Interceptor for #self_name {}

        #inventory::submit! {
            #InterceptorRegistrar {
                register: #self_name::register
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

// #[interceptor()] macro parsing ===================================================

#[derive(Clone)]
pub enum InterceptorArgs {
    /// This variant represents -> `#[interceptor(Auth)]`
    Traditional(Path),

    /// This variant represents -> `#[interceptor(Auth, config = expr)]`
    WithConfig { interceptor: Path, config: Expr },

    /// This variant represents -> `#[interceptor(<expr>)]` (escape hatch)
    Expression(Expr),
}

pub enum SwordInterceptor {
    Traditional(Path),
    WithConfig { interceptor: Path, config: Expr },
}

impl InterceptorArgs {
    /// Returns `Some(&Path)` if the provided interceptor is a sword's built in trait impl
    /// like `OnConnect`, `OnRequest`, `OnRequestWithConfig` and others.
    pub const fn path(&self) -> Option<&Path> {
        match self {
            InterceptorArgs::Traditional(path) => Some(path),
            InterceptorArgs::WithConfig { interceptor, .. } => Some(interceptor),
            Self::Expression(_) => None,
        }
    }

    /// Returns `true` if the provided interceptor is a sword's built in trait impl
    /// like `OnConnect`, `OnRequest`, `OnRequestWithConfig` and others.
    pub const fn is_sword_interceptor(&self) -> bool {
        matches!(self, Self::Traditional(_) | Self::WithConfig { .. })
    }
}

impl From<InterceptorArgs> for TokenStream2 {
    fn from(args: InterceptorArgs) -> Self {
        match args {
            InterceptorArgs::Traditional(interceptor) => quote! { #interceptor },
            InterceptorArgs::WithConfig {
                interceptor,
                config,
            } => {
                quote! { #interceptor, config = #config }
            }
            InterceptorArgs::Expression(expr) => quote! { #expr },
        }
    }
}

impl Parse for SwordInterceptor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // a sword interceptor always start with a path
        let interceptor = input.parse::<Path>()?;

        // if after the path there's nothing:
        if input.is_empty() {
            return Ok(Self::Traditional(interceptor));
        }

        // if there's something else must be `, config = <expr>`
        input.parse::<Token![,]>()?;

        let key = input.parse::<Ident>()?;

        if key != "config" {
            return Err(syn::Error::new(key.span(), "expected `config`"));
        }

        input.parse::<Token![=]>()?;

        let config: Expr = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("unexpected tokens after interceptor config"));
        }

        Ok(Self::WithConfig {
            interceptor,
            config,
        })
    }
}

impl Parse for InterceptorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // first of all try to parse the input as a sword interceptor
        let fork = input.fork();

        if let Ok(parsed) = fork.parse::<SwordInterceptor>() {
            input.advance_to(&fork);

            return Ok(match parsed {
                SwordInterceptor::Traditional(path) => Self::Traditional(path),
                SwordInterceptor::WithConfig {
                    interceptor,
                    config,
                } => Self::WithConfig {
                    interceptor,
                    config,
                },
            });
        }

        Ok(Self::Expression(input.parse()?))
    }
}

pub fn take_interceptors(attrs: &[Attribute]) -> Result<Vec<InterceptorArgs>> {
    let mut interceptors = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("interceptor") {
            interceptors.push(attr.parse_args::<InterceptorArgs>()?);
        }
    }

    Ok(interceptors)
}
