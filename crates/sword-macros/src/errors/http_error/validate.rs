use syn::{Error, Fields, Ident};

use super::parse::HttpErrorConfig;
use crate::errors::{MessageValue, validate_message_references, validate_named_field_refs};

pub struct HttpErrorValidator;

impl HttpErrorValidator {
    pub fn validate_variant_config(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
        config: &HttpErrorConfig,
    ) -> syn::Result<()> {
        if config.transparent {
            crate::errors::validate_transparent_single_unnamed(variant_name, fields)?;
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
                    .error_field
                    .as_deref()
                    .map(|field| ("error", field))
                    .into_iter()
                    .chain(
                        config
                            .errors_field
                            .as_deref()
                            .map(|field| ("errors", field)),
                    )
                    .collect();

                validate_named_field_refs(enum_name, variant_name, named, &extra_refs)?;
            }
            Fields::Unnamed(unnamed) => {
                crate::errors::validate_single_unnamed(variant_name, unnamed)?;

                if Self::message_field_name(config).is_some()
                    || matches!(config.message, Some(MessageValue::Interpolated(_)))
                    || config.error_field.is_some()
                    || config.errors_field.is_some()
                {
                    return Err(Error::new_spanned(
                        variant_name,
                        "tuple variants do not support `message = field`, interpolated messages, `error`, or `errors`; use named fields or construct the client message before creating the error",
                    ));
                }
            }
            Fields::Unit => {
                if Self::message_field_name(config).is_some()
                    || matches!(config.message, Some(MessageValue::Interpolated(_)))
                    || config.error_field.is_some()
                    || config.errors_field.is_some()
                {
                    return Err(Error::new_spanned(
                        variant_name,
                        "unit variants do not support field-based or interpolated `message`, `error`, or `errors`",
                    ));
                }
            }
        }

        Ok(())
    }

    pub fn message_field_name(config: &HttpErrorConfig) -> Option<&str> {
        match &config.message {
            Some(MessageValue::Field(field)) => Some(field.as_str()),
            _ => None,
        }
    }
}
