use std::collections::HashMap;

use axum::{
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response as AxumResponse},
};

/// A builder for creating HTTP redirect responses.
///
/// # Example
///
/// ```rust
/// use sword::web::*;
///
/// async fn login_redirect() -> Redirect {
///     Redirect::found("/login")
/// }
///
/// async fn with_headers() -> Redirect {
///     Redirect::temporary("/temp")
///         .header("X-Redirect-Reason", "maintenance")
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Redirect {
    code: StatusCode,
    location: String,
    headers: Option<HashMap<HeaderName, HeaderValue>>,
}

impl Redirect {
    /// Creates a redirect with a custom status code.
    pub fn status(code: impl TryInto<StatusCode>, location: impl Into<String>) -> Self {
        Self {
            code: code.try_into().unwrap_or(StatusCode::TEMPORARY_REDIRECT),
            location: location.into(),
            headers: None,
        }
    }

    /// 301 Moved Permanently - Resource has permanently moved.
    pub fn permanent(location: impl Into<String>) -> Self {
        Self::status(StatusCode::MOVED_PERMANENTLY, location)
    }

    /// 302 Found - Temporary redirect (commonly used).
    pub fn found(location: impl Into<String>) -> Self {
        Self::status(StatusCode::FOUND, location)
    }

    /// 303 See Other - Redirect after POST (forces GET).
    pub fn see_other(location: impl Into<String>) -> Self {
        Self::status(StatusCode::SEE_OTHER, location)
    }

    /// 307 Temporary Redirect - Temporary, preserves method.
    pub fn temporary(location: impl Into<String>) -> Self {
        Self::status(StatusCode::TEMPORARY_REDIRECT, location)
    }

    /// 308 Permanent Redirect - Permanent, preserves method.
    pub fn permanent_redirect(location: impl Into<String>) -> Self {
        Self::status(StatusCode::PERMANENT_REDIRECT, location)
    }

    /// Adds a custom header to the redirect response.
    pub fn header(mut self, key: &str, value: &str) -> Self {
        if let (Ok(header_name), Ok(header_value)) =
            (HeaderName::try_from(key), HeaderValue::try_from(value))
        {
            (*self.headers.get_or_insert_with(HashMap::new)).insert(header_name, header_value);
        }
        self
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> AxumResponse {
        let mut response = (self.code, [("Location", self.location)]).into_response();

        if let Some(headers) = self.headers {
            for (key, value) in headers.iter() {
                response.headers_mut().insert(key, value.clone());
            }
        }

        response
    }
}
