use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::discouraged::Speculative;
use syn::{
    Expr, Path, Token,
    parse::{Parse, ParseStream},
};

pub enum InterceptorArgs {
    Traditional(Path),
    WithConfig { interceptor: Path, config: Expr },
    Expression(Expr),
}

enum ParsedTraditionalInterceptor {
    Traditional(Path),
    WithConfig { interceptor: Path, config: Expr },
}

impl InterceptorArgs {
    pub fn is_sword(&self) -> bool {
        matches!(self, Self::Traditional(_) | Self::WithConfig { .. })
    }

    pub fn sword_path(&self) -> Option<&Path> {
        match self {
            Self::Traditional(path) => Some(path),
            Self::WithConfig { interceptor, .. } => Some(interceptor),
            Self::Expression(_) => None,
        }
    }

    pub fn to_token_stream(&self) -> TokenStream {
        match self {
            Self::Traditional(interceptor) => quote! { #interceptor },
            Self::WithConfig {
                interceptor,
                config,
            } => {
                quote! { #interceptor, config = #config }
            }
            Self::Expression(expr) => quote! { #expr },
        }
    }
}

impl From<InterceptorArgs> for TokenStream {
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

impl Parse for InterceptorArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Try the Sword-specific syntax first (`Auth` or `Auth, config = ...`).
        // We parse it on a fork so we can fall back to a generic expression
        // without consuming the real input when that specialized form doesn't match.
        let fork = input.fork();

        if let Ok(parsed) = fork.parse::<ParsedTraditionalInterceptor>() {
            input.advance_to(&fork);

            return Ok(match parsed {
                ParsedTraditionalInterceptor::Traditional(interceptor) => {
                    Self::Traditional(interceptor)
                }
                ParsedTraditionalInterceptor::WithConfig {
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

impl Parse for ParsedTraditionalInterceptor {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let interceptor: Path = input.parse()?;

        if input.is_empty() {
            return Ok(Self::Traditional(interceptor));
        }

        input.parse::<Token![,]>()?;

        // `Path, config = Expr` is the only keyed Sword form we accept here.
        // Any other tail should fail so the outer parser can treat it as a
        // generic expression instead.
        let key: syn::Ident = input.parse()?;
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
