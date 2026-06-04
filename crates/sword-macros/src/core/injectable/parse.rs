use crate::shared::StructFields;
use proc_macro::TokenStream;
use syn::parse::{ParseStream, Result as ParseResult};
use syn::{Ident, ItemStruct, Token, Type, parse::Parse};

pub enum InjectableKind {
    Provider,
    Component,
}

pub struct InjectableInput {
    pub struct_name: Ident,
    pub fields: Vec<(Ident, Type)>,
    pub derive_clone: bool,
    pub kind: InjectableKind,
    pub trait_as: Option<Type>,
}

struct InjectableArgs {
    kind: InjectableKind,
    derive_clone: bool,
    trait_as: Option<Type>,
}

impl Parse for InjectableArgs {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut kind = InjectableKind::Component;
        let mut derive_clone = true;
        let mut trait_as = None;

        if input.is_empty() {
            return Ok(Self {
                kind,
                derive_clone,
                trait_as,
            });
        }

        while !input.is_empty() {
            if input.peek(Token![as]) {
                input.parse::<Token![as]>()?;
                input.parse::<Token![=]>()?;
                trait_as = Some(input.parse()?);
            } else {
                let arg: Ident = input.parse()?;

                match arg.to_string().as_str() {
                    "provider" => kind = InjectableKind::Provider,
                    "component" => kind = InjectableKind::Component,
                    "no_derive_clone" => derive_clone = false,
                    _ => {
                        return Err(syn::Error::new_spanned(
                            arg,
                            "Unknown attribute. Use 'provider', 'component', 'no_derive_clone', or 'as = dyn Trait'",
                        ));
                    }
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            kind,
            derive_clone,
            trait_as,
        })
    }
}

pub fn parse_injectable_input(
    attr: TokenStream,
    item: TokenStream,
) -> Result<InjectableInput, syn::Error> {
    let input = syn::parse::<ItemStruct>(item)?;
    let args = syn::parse::<InjectableArgs>(attr)?;

    Ok(InjectableInput {
        struct_name: input.clone().ident,
        fields: StructFields::parse(&input)?,
        derive_clone: args.derive_clone,
        kind: args.kind,
        trait_as: args.trait_as,
    })
}
