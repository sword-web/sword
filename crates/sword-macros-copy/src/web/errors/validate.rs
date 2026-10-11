use syn::{Error, Fields, Ident};

use super::parse::HttpErrorArgs;
use crate::shared::errors::{
    MessageValue, validate_message_references, validate_named_field_refs, validate_single_unnamed,
    validate_transparent_single_unnamed,
};

pub fn validate_variant(
    enum_name: &Ident,
    variant_name: &Ident,
    fields: &Fields,
    config: &HttpErrorArgs,
) -> syn::Result<()> {
    if config.transparent.is_present() {
        validate_transparent_single_unnamed(variant_name, fields)?;
        return Ok(());
    }

    if config.code.is_none() {
        return Err(Error::new_spanned(
            variant_name,
            "missing `code` after merging #[http_error(...)] and #[http(...)]",
        ));
    }

    match fields {
        Fields::Named(named) => {
            validate_message_references(enum_name, variant_name, named, &config.message)?;

            let extra_refs: Vec<(&str, &str)> = config
                .error
                .as_ref()
                .map(|field| ("error", field.as_str()))
                .into_iter()
                .chain(
                    config
                        .errors
                        .as_ref()
                        .map(|field| ("errors", field.as_str())),
                )
                .collect();

            validate_named_field_refs(enum_name, variant_name, named, &extra_refs)?;
        }
        Fields::Unnamed(unnamed) => {
            validate_single_unnamed(variant_name, unnamed)?;

            if has_field_based_refs(config) {
                return Err(Error::new_spanned(
                    variant_name,
                    "tuple variants do not support `message = field`, interpolated messages, `error`, or `errors`; use named fields or construct the client message before creating the error",
                ));
            }
        }
        Fields::Unit => {
            if has_field_based_refs(config) {
                return Err(Error::new_spanned(
                    variant_name,
                    "unit variants do not support field-based or interpolated `message`, `error`, or `errors`",
                ));
            }
        }
    }

    Ok(())
}

fn has_field_based_refs(config: &HttpErrorArgs) -> bool {
    matches!(
        config.message,
        Some(MessageValue::Field(_)) | Some(MessageValue::Interpolated(_))
    ) || config.error.is_some()
        || config.errors.is_some()
}
