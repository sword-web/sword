use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{ItemImpl, ItemTrait, Path, Type, TypeParamBound};

pub fn expand(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item: TokenStream2 = item.into();

    if let Ok(mut item_trait) = syn::parse2::<ItemTrait>(item.clone()) {
        expand_trait(&mut item_trait).into()
    } else if let Ok(item_impl) = syn::parse2::<ItemImpl>(item) {
        expand_impl(item_impl).into()
    } else {
        syn::Error::new(
            Span::call_site(),
            "`#[contract]` only supports trait definitions and trait implementations.",
        )
        .to_compile_error()
        .into()
    }
}

fn expand_trait(item_trait: &mut ItemTrait) -> TokenStream2 {
    add_di_bounds(item_trait);
    let has_async = item_trait
        .items
        .iter()
        .any(|item| matches!(item, syn::TraitItem::Fn(m) if m.sig.asyncness.is_some()));

    if has_async {
        quote! {
            #[::sword::internal::async_trait::async_trait]
            #item_trait
        }
    } else {
        quote! { #item_trait }
    }
}

fn expand_impl(item_impl: ItemImpl) -> TokenStream2 {
    let Some((_, trait_path, _)) = &item_impl.trait_ else {
        return syn::Error::new(
            Span::call_site(),
            "`#[contract]` requires a trait — inherent impl blocks are not supported.",
        )
        .to_compile_error();
    };

    let has_async = item_impl
        .items
        .iter()
        .any(|item| matches!(item, syn::ImplItem::Fn(m) if m.sig.asyncness.is_some()));

    let binding = gen_trait_binding(trait_path, &item_impl.self_ty);

    if has_async {
        quote! {
            #[::sword::internal::async_trait::async_trait]
            #item_impl
            #binding
        }
    } else {
        quote! {
            #item_impl
            #binding
        }
    }
}

fn gen_trait_binding(trait_path: &Path, struct_type: &Type) -> TokenStream2 {
    quote! {
        ::sword::internal::inventory::submit! {
            ::sword::internal::core::TraitBindingRegistrar {
                register: |container: &::sword::internal::core::DependencyContainer| {
                    container.register_trait_binding(
                        ::std::any::TypeId::of::<
                            ::sword::internal::core::InjectableTrait::<dyn #trait_path>
                        >(),
                        ::std::any::TypeId::of::<#struct_type>(),
                        Box::new(|state: &::sword::internal::core::State| {
                            let concrete: ::std::sync::Arc::<#struct_type> =
                                state.borrow::<#struct_type>()?;
                            let trait_obj: ::std::sync::Arc<dyn #trait_path> = concrete;
                            Ok(::std::sync::Arc::new(
                                ::sword::internal::core::InjectableTrait(trait_obj)
                            ) as ::std::sync::Arc<
                                dyn ::std::any::Any + ::std::marker::Send + ::std::marker::Sync
                            >)
                        }),
                    );
                },
            }
        }
    }
}

fn add_di_bounds(item_trait: &mut ItemTrait) {
    let supertraits = &mut item_trait.supertraits;

    let has_send = supertraits
        .iter()
        .any(|b| matches!(b, TypeParamBound::Trait(t) if t.path.is_ident("Send")));
    let has_sync = supertraits
        .iter()
        .any(|b| matches!(b, TypeParamBound::Trait(t) if t.path.is_ident("Sync")));
    let has_static = supertraits
        .iter()
        .any(|b| matches!(b, TypeParamBound::Lifetime(l) if l.ident == "static"));

    let was_empty = supertraits.is_empty();

    if !has_send {
        supertraits.push(syn::parse_quote!(Send));
    }
    if !has_sync {
        supertraits.push(syn::parse_quote!(Sync));
    }
    if !has_static {
        supertraits.push(syn::parse_quote!('static));
    }

    if was_empty && !supertraits.is_empty() {
        item_trait.colon_token = Some(Default::default());
    }
}
