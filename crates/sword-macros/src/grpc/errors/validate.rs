use syn::{Error, Fields, Ident};

use super::parse::GrpcErrorArgs;
use crate::shared::errors::{
    MessageValue, validate_message_references, validate_single_unnamed,
    validate_transparent_single_unnamed,
};

pub fn validate_variant(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &GrpcErrorArgs,
) -> syn::Result<()> {
    if config.transparent.is_present() {
        validate_transparent_single_unnamed(variant_name, fields)?;
        return Ok(());
    }

    match fields {
        Fields::Named(named) => {
            validate_message_references(enum_name, variant_name, named, &config.message)?;
        }
        Fields::Unnamed(unnamed) => {
            validate_single_unnamed(variant_name, unnamed)?;

            if has_field_based_message(config) {
                return Err(Error::new_spanned(
                    variant_name,
                    "tuple variants do not support `message = field` or interpolated messages; use a named-field variant or build the final client message before creating the error",
                ));
            }
        }
        Fields::Unit => {
            if has_field_based_message(config) {
                return Err(Error::new_spanned(
                    variant_name,
                    "unit variants do not support field-based or interpolated `message`",
                ));
            }
        }
    }

    Ok(())
}

const fn has_field_based_message(config: &GrpcErrorArgs) -> bool {
    matches!(
        config.message,
        Some(MessageValue::Field(_)) | Some(MessageValue::Interpolated(_))
    )
}
