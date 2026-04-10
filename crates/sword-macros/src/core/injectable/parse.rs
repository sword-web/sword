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
}

struct InjectableArgs {
    kind: InjectableKind,
    derive_clone: bool,
}

impl Parse for InjectableArgs {
    fn parse(input: ParseStream) -> ParseResult<Self> {
        let mut kind = InjectableKind::Component;
        let mut derive_clone = true;

        if input.is_empty() {
            return Ok(Self { kind, derive_clone });
        }

        while !input.is_empty() {
            let arg: Ident = input.parse()?;

            match arg.to_string().as_str() {
                "provider" => kind = InjectableKind::Provider,
                "component" => kind = InjectableKind::Component,
                "no_derive_clone" => derive_clone = false,
                _ => {
                    return Err(syn::Error::new_spanned(
                        arg,
                        "Unknown attribute. Use 'provider', 'component', or 'no_derive_clone'",
                    ));
                }
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { kind, derive_clone })
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
    })
}
