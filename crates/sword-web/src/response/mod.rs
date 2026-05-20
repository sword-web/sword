mod file;
mod json;
mod redirect;

use crate::request::RequestError;

pub use file::*;
pub use json::{JsonResponse, JsonResponseBody};
pub use redirect::Redirect;
pub use sword_macros::HttpError;

#[allow(unused)]
#[cfg(feature = "multipart")]
pub use multipart::*;

pub type WebResult<T = JsonResponse, E = JsonResponse> = Result<T, E>;

impl From<RequestError> for JsonResponse {
    fn from(error: RequestError) -> JsonResponse {
        match error {
            RequestError::ParseError { message, details } => {
                tracing::error!(details = ?details,  "Request parse error: {message}");
                JsonResponse::BadRequest().message(message).error(details)
            }
            #[cfg(feature = "validation-validator")]
            RequestError::ValidatorError { message, details } => {
                tracing::error!(details = ?details,  "Request validation error: {message}");
                JsonResponse::BadRequest().message(message).errors(details)
            }

            RequestError::BodyIsEmpty => {
                JsonResponse::BadRequest().message("Request body is empty")
            }
            RequestError::BodyTooLarge => JsonResponse::PayloadTooLarge()
                .message("The request body exceeds the maximum allowed size by the server"),
            RequestError::UnsupportedMediaType { message } => {
                JsonResponse::UnsupportedMediaType().message(message)
            }
            RequestError::DeserializationError {
                message,
                error,
                source,
            } => {
                tracing::error!(source = %source, "Request deserialization error: {message}");
                JsonResponse::BadRequest().message(message).error(error)
            }
            RequestError::InvalidHeaderName(name) => {
                tracing::error!(header = %name, "Invalid header name");
                JsonResponse::BadRequest()
                    .message("Invalid header name")
                    .error(format!("Header '{name}' contains invalid characters"))
            }
            RequestError::InvalidHeaderValue(name) => {
                tracing::error!(header = %name, "Invalid header value");
                JsonResponse::BadRequest()
                    .message("Invalid header value")
                    .error(format!("Header '{name}' contains an invalid value",))
            }
            #[cfg(feature = "multipart")]
            RequestError::MultipartError(err) => {
                tracing::error!(error = %err, "Multipart error");
                JsonResponse::status(err.status()).message("Multipart error")
            }
            #[cfg(feature = "multipart")]
            RequestError::MultipartRejection(err) => {
                tracing::error!(error = %err, "Multipart rejection");
                JsonResponse::status(err.status()).message("Multipart rejection")
            }
        }
    }
}

#[cfg(feature = "validation-validator")]
/// Structured JSON output for validation errors  from the `validator` crate.
///
/// # Example
///
/// ```json
/// {
///   "email": [
///     {
///       "code": "invalid",
///       "message": "Must be a valid email address"
///     }
///   ],
///   "password": [
///     {
///       "code": "length",
///       "message": "Must be at least 8 characters long"
///     },
///     {
///       "code": "strength",
///       "message": "Must contain a number"
///     }
///   ]
/// }
/// ```
pub(crate) fn format_validator_errors(e: validator::ValidationErrors) -> serde_json::Value {
    let mut formatted_errors = serde_json::Map::new();

    for (field, field_errors) in e.field_errors() {
        let mut formatted_field_errors = vec![];

        for error in field_errors {
            formatted_field_errors.push(serde_json::json!({
                "code": error.code,
                "message": error.message,
            }));
        }

        formatted_errors.insert(
            field.to_string(),
            serde_json::Value::Array(formatted_field_errors),
        );
    }

    serde_json::Value::Object(formatted_errors)
}

#[cfg(feature = "multipart")]
mod multipart {
    use super::JsonResponse;
    use axum::extract::multipart::MultipartError;
    use axum::extract::multipart::MultipartRejection;

    impl From<MultipartRejection> for JsonResponse {
        fn from(err: MultipartRejection) -> Self {
            tracing::error!("MultipartRejection: {err:?}");
            JsonResponse::status(err.status())
        }
    }

    impl From<MultipartError> for JsonResponse {
        fn from(err: MultipartError) -> Self {
            tracing::error!("MultipartError: {err:?}");
            JsonResponse::status(err.status())
        }
    }
}
