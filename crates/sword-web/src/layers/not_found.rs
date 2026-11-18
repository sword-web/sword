//! Standard responses for routes that do not match any path or method.
//!
//! These are registered as axum fallbacks, so they only run for requests that
//! reach the router itself. Mounted services such as static file serving keep
//! their own responses.

use axum::response::{IntoResponse, Response};

use crate::JsonResponse;

/// Response for a path that does not match any registered route.
pub async fn not_found() -> Response {
    JsonResponse::NotFound()
        .message("The requested resource was not found.")
        .into_response()
}

/// Response for a path that exists but does not support the request method.
pub async fn method_not_allowed() -> Response {
    JsonResponse::MethodNotAllowed()
        .message("The method is not allowed for the requested resource.")
        .into_response()
}
