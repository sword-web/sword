use proc_macro::TokenStream;
use syn::{Ident, ItemStruct, Type};

use super::{ControllerArgs, ParsedControllerKind};
use crate::{interceptor::InterceptorArgs, shared::StructFields};

pub struct ControllerStruct {
    pub name: Ident,
    pub kind: ParsedControllerKind,
    pub fields: Vec<(Ident, Type)>,
    pub interceptors: Vec<InterceptorArgs>,
}

impl ControllerStruct {
    pub fn parse(attr: TokenStream, item: TokenStream) -> syn::Result<Self> {
        let input = syn::parse::<ItemStruct>(item)?;
        let fields = StructFields::parse(&input)?;

        let controller_args = syn::parse::<ControllerArgs>(attr)?;
        let kind = ParsedControllerKind::try_from(controller_args)?;

        let mut interceptors = Vec::new();

        for attr in &input.attrs {
            if attr.path().is_ident("interceptor") {
                interceptors.push(attr.parse_args::<InterceptorArgs>()?);
            }
        }

        Ok(Self {
            name: input.ident,
            kind,
            fields,
            interceptors,
        })
    }
}
