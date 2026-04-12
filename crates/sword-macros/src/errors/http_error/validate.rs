use syn::{Error, Fields, Ident};

use super::parse::HttpErrorConfig;
use crate::errors::MessageValue;

pub struct HttpErrorValidator;

impl HttpErrorValidator {
    pub fn validate_variant_config(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &Fields,
        config: &HttpErrorConfig,
    ) -> syn::Result<()> {
        if config.transparent {
            let is_single_unnamed =
                matches!(fields, Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1);

            if !is_single_unnamed {
                return Err(Error::new_spanned(
                    variant_name,
                    "transparent variants must have exactly one unnamed field",
                ));
            }

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
                if let Some(field) = Self::message_field_name(config) {
                    Self::ensure_named_field_exists(
                        enum_name,
                        variant_name,
                        named,
                        field,
                        "message",
                    )?;
                }

                if let Some(field) = &config.error_field {
                    Self::ensure_named_field_exists(
                        enum_name,
                        variant_name,
                        named,
                        field,
                        "error",
                    )?;
                }

                if let Some(field) = &config.errors_field {
                    Self::ensure_named_field_exists(
                        enum_name,
                        variant_name,
                        named,
                        field,
                        "errors",
                    )?;
                }
            }
            Fields::Unnamed(unnamed) => {
                if unnamed.unnamed.len() != 1 {
                    return Err(Error::new_spanned(
                        variant_name,
                        "non-transparent tuple variants are only supported with exactly one field",
                    ));
                }

                if Self::message_field_name(config).is_some()
                    || config.error_field.is_some()
                    || config.errors_field.is_some()
                {
                    return Err(Error::new_spanned(
                        variant_name,
                        "tuple variants do not support `message = field`, `error`, or `errors`; use named fields or construct the client message before creating the error",
                    ));
                }
            }
            Fields::Unit => {
                if Self::message_field_name(config).is_some()
                    || config.error_field.is_some()
                    || config.errors_field.is_some()
                {
                    return Err(Error::new_spanned(
                        variant_name,
                        "unit variants do not support field-based `message`, `error`, or `errors`",
                    ));
                }
            }
        }

        Ok(())
    }

    fn ensure_named_field_exists(
        enum_name: &Ident,
        variant_name: &Ident,
        fields: &syn::FieldsNamed,
        field_name: &str,
        attr_name: &str,
    ) -> syn::Result<()> {
        let exists = fields
            .named
            .iter()
            .filter_map(|field| field.ident.as_ref())
            .any(|ident| ident == field_name);

        if exists {
            return Ok(());
        }

        Err(Error::new_spanned(
            variant_name,
            format!(
                "`{attr_name} = {field_name}` references a missing field on {enum_name}::{variant_name}`"
            ),
        ))
    }

    pub fn message_field_name(config: &HttpErrorConfig) -> Option<&str> {
        match &config.message {
            Some(MessageValue::Field(field)) => Some(field.as_str()),
            _ => None,
        }
    }
}
