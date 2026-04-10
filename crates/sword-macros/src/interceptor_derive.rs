use crate::shared::{StructFields, gen_build, gen_clone};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{Ident, ItemStruct, Type};

pub struct InterceptorInput {
    pub struct_name: Ident,
    pub fields: Vec<(Ident, Type)>,
}

pub fn parse_interceptor_input(item: &ItemStruct) -> syn::Result<InterceptorInput> {
    let fields = StructFields::parse(item)?;

    Ok(InterceptorInput {
        struct_name: item.ident.clone(),
        fields,
    })
}

pub fn derive_interceptor(input: TokenStream) -> syn::Result<TokenStream> {
    let input_struct = syn::parse::<ItemStruct>(input)?;
    let parsed_input = parse_interceptor_input(&input_struct)?;
    let builder = generate_interceptor_builder(&parsed_input);

    let expanded = quote! {
        #builder
    };

    Ok(TokenStream::from(expanded))
}

pub fn generate_interceptor_builder(input: &InterceptorInput) -> TokenStream2 {
    let self_name = &input.struct_name;
    let self_fields = &input.fields;

    let build_impl = gen_build(self_name, self_fields);
    let clone_impl = gen_clone(self_name, self_fields);

    quote! {
        #build_impl
        #clone_impl

        impl ::sword::internal::core::Interceptor for #self_name {}

        ::sword::internal::inventory::submit! {
            ::sword::internal::core::InterceptorRegistrar {
                register: #self_name::register,
            }
        }
    }
}
