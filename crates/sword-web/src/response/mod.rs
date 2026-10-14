mod file;
mod json;
mod redirect;
mod sse;

use crate::request::RequestError;

pub use file::*;
pub use json::{JsonResponse, JsonResponseBody};
pub use redirect::Redirect;
pub use sse::*;
pub use sword_macros::HttpError;

pub type WebResult<T = JsonResponse, E = JsonResponse> = Result<T, E>;

impl From<RequestError> for JsonResponse {
    fn from(error: RequestError) -> JsonResponse {
        tracing::trace!(
            target: "sword.web.rejection",
            rejection = ?error,
            "rejecting request"
        );

        match error {
            RequestError::ParseError { message, details } => {
                JsonResponse::BadRequest().message(message).error(details)
            }
            #[cfg(feature = "validation-validator")]
            RequestError::ValidatorError { message, details } => {
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
            RequestError::DeserializationError { message, error, .. } => {
                JsonResponse::BadRequest().message(message).error(error)
            }
            RequestError::InvalidHeaderName(name) => JsonResponse::BadRequest()
                .message("Invalid header name")
                .error(format!("Header '{name}' contains invalid characters")),

            RequestError::InvalidHeaderValue(name) => JsonResponse::BadRequest()
                .message("Invalid header value")
                .error(format!("Header '{name}' contains an invalid value",)),

            #[cfg(feature = "multipart")]
            RequestError::MultipartError(err) => JsonResponse::status(err.status())
                .message("Multipart error")
                .error(err.body_text()),

            #[cfg(feature = "multipart")]
            RequestError::MultipartRejection(err) => JsonResponse::status(err.status())
                .message("Multipart rejection")
                .error(err.body_text()),
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
    use crate::request::RequestError;

    use axum::extract::multipart::MultipartError;
    use axum::extract::multipart::MultipartRejection;

    impl From<MultipartRejection> for JsonResponse {
        fn from(err: MultipartRejection) -> Self {
            RequestError::from(err).into()
        }
    }

    impl From<MultipartError> for JsonResponse {
        fn from(err: MultipartError) -> Self {
            RequestError::from(err).into()
        }
    }
}
