use http_body_util::LengthLimitError;
use std::error::Error as StdError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RequestError {
    #[error("Failed to parse request: {message}")]
    ParseError { message: String, details: String },

    #[error("Deserialization error: {error}")]
    DeserializationError {
        message: &'static str,
        error: String,

        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[cfg(feature = "validation-validator")]
    #[error("Failed to validate request")]
    ValidatorError {
        message: &'static str,
        details: serde_json::Value,
    },

    #[error("Request body is empty")]
    BodyIsEmpty,

    #[error("Request body is too large")]
    BodyTooLarge,

    #[error("Unsupported media type: {message}")]
    UnsupportedMediaType { message: &'static str },

    #[error("Invalid header name: {0}")]
    InvalidHeaderName(String),

    #[error("Invalid header value for '{0}'")]
    InvalidHeaderValue(String),

    #[cfg(feature = "multipart")]
    #[error("Multipart error: {0}")]
    MultipartError(#[from] axum::extract::multipart::MultipartError),

    #[cfg(feature = "multipart")]
    #[error("Multipart Rejection: {0}")]
    MultipartRejection(#[from] axum::extract::multipart::MultipartRejection),
}

impl RequestError {
    pub fn parse_error(message: impl Into<String>, details: impl Into<String>) -> Self {
        RequestError::ParseError {
            message: message.into(),
            details: details.into(),
        }
    }

    pub(crate) fn from_body_read_error(err: &(dyn StdError + 'static)) -> Self {
        if err.is::<LengthLimitError>() {
            return Self::BodyTooLarge;
        }

        let mut source = err.source();

        while let Some(current) = source {
            if current.is::<LengthLimitError>() {
                return Self::BodyTooLarge;
            }

            source = current.source();
        }

        Self::parse_error("Failed to read request body", "Error reading body")
    }

    #[cfg(feature = "validation-validator")]
    pub fn validator_error(message: &'static str, details: serde_json::Value) -> Self {
        RequestError::ValidatorError { message, details }
    }

    pub fn unsupported_media_type(message: &'static str) -> Self {
        RequestError::UnsupportedMediaType { message }
    }

    pub fn deserialization_error(
        message: &'static str,
        error: String,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        RequestError::DeserializationError {
            message,
            error,
            source,
        }
    }

    pub fn invalid_header_value(name: impl Into<String>) -> Self {
        RequestError::InvalidHeaderValue(name.into())
    }
}
