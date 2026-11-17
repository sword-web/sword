//! Not found response mapping middleware.
//!
//! This middleware rewrites plain `404 Not Found` (and `405 Method Not Allowed`)
//! responses into Sword's standardized JSON shape.
//!
//! It is intentionally not a `Router::fallback`: services mounted with
//! `nest_service` (static files, Socket.IO, ...) answer with their own 404 and
//! would never reach a fallback. Looking at the final status is the only way to
//! normalize those responses too.

use axum::{
    body::{Body, to_bytes},
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use serde_json::Value;

use crate::JsonResponse;

pub async fn not_found_mw(req: Request, next: Next) -> Response {
    let response = next.run(req).await;

    let status = response.status();

    if status != StatusCode::NOT_FOUND && status != StatusCode::METHOD_NOT_ALLOWED {
        return response;
    }

    let (parts, body) = response.into_parts();

    let Ok(body_bytes) = to_bytes(body, usize::MAX).await else {
        return Response::from_parts(parts, Body::from("body read error"));
    };

    let already_has_message = serde_json::from_slice::<Value>(&body_bytes)
        .ok()
        .is_some_and(|json| json.get("message").is_some());

    if already_has_message {
        return Response::from_parts(parts, Body::from(body_bytes));
    }

    let mut replacement = if status == StatusCode::METHOD_NOT_ALLOWED {
        JsonResponse::MethodNotAllowed()
            .message("The method is not allowed for the requested resource.")
            .into_response()
    } else {
        JsonResponse::NotFound()
            .message("The requested resource was not found.")
            .into_response()
    };

    // Keep the headers produced by the inner services (request id, vary, allow, ...)
    // but not the ones that describe the body we just replaced.
    let mut headers = parts.headers;
    headers.remove(axum::http::header::CONTENT_TYPE);
    headers.remove(axum::http::header::CONTENT_LENGTH);

    for (name, value) in headers {
        if let Some(name) = name {
            replacement.headers_mut().append(name, value);
        }
    }

    replacement
}
