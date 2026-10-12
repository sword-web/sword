use darling::{FromDeriveInput, ast, util};
use syn::{Ident, Type};

#[derive(FromDeriveInput)]
#[darling(supports(struct_named, struct_unit))]
pub struct Struct {
    pub ident: Ident,
    pub data: ast::Data<util::Ignored, syn::Field>,
}

impl Struct {
    pub fn fields(&self) -> Vec<(Ident, Type)> {
        self.data
            .as_ref()
            .take_struct()
            .map(|s| {
                s.fields
                    .into_iter()
                    .map(|f| (f.ident.clone().expect("named field"), f.ty.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }
}
