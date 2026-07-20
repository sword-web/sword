use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{DeriveInput, Error, Expr, Lit, Meta};

pub fn expand_event_struct(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse::<DeriveInput>(item)?;

    if !matches!(input.data, syn::Data::Struct(_)) {
        return Err(syn::Error::new_spanned(
            &input,
            "#[event] can only be applied to structs",
        ));
    }

    let struct_name = &input.ident;
    let meta = syn::parse::<Meta>(args)?;

    let nv = meta.require_name_value().map_err(|_| {
        Error::new_spanned(&meta, r#"expected format: #[event(key = "event.key"])"#)
    })?;

    if !nv.path.is_ident("key") {
        return Err(Error::new_spanned(&nv.path, "expected `key` attribute"));
    }

    let Expr::Lit(expr_lit) = &nv.value else {
        return Err(Error::new_spanned(
            &nv.value,
            "expected string literal for key",
        ));
    };

    let Lit::Str(lit_str) = &expr_lit.lit else {
        return Err(Error::new_spanned(
            &expr_lit.lit,
            "expected string literal for key",
        ));
    };

    let key = lit_str.value();

    let expanded: TokenStream2 = quote! {
        #[derive(::std::clone::Clone, ::std::fmt::Debug)]
        #input

        impl ::sword::internal::events::Event for #struct_name {
            fn key(&self) -> &'static str {
                #key
            }

            fn clone_event(&self) -> ::std::boxed::Box<dyn ::sword::internal::events::Event> {
                ::std::boxed::Box::new(self.clone())
            }
        }
    };

    Ok(TokenStream::from(expanded))
}
