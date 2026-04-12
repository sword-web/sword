//! Not found response mapping middleware.
//!
//! This module provides a layer that transforms plain `404 Not Found`
//! responses into Sword's standardized JSON response format.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use axum_responses::JsonResponse;
use serde_json::Value;
use tower::{Layer, Service};

#[derive(Clone)]
pub struct NotFoundLayer;

impl<S> Layer<S> for NotFoundLayer {
    type Service = NotFoundService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        NotFoundService { inner }
    }
}

#[derive(Clone)]
pub struct NotFoundService<S> {
    inner: S,
}

impl<S> Service<Request<Body>> for NotFoundService<S>
where
    S: Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
    S::Error: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let response = inner.call(req).await?;

            if response.status() != StatusCode::NOT_FOUND {
                return Ok(response);
            }

            let (parts, body) = response.into_parts();

            let body_bytes = match to_bytes(body, usize::MAX).await {
                Ok(bytes) => bytes,
                Err(_) => {
                    return Ok(Response::from_parts(parts, Body::from("body read error")));
                }
            };

            let already_has_message = match serde_json::from_slice::<Value>(&body_bytes) {
                Ok(json) => json.get("message").is_some(),
                Err(_) => false,
            };

            if already_has_message {
                return Ok(Response::from_parts(parts, Body::from(body_bytes)));
            }

            Ok(JsonResponse::NotFound()
                .message("The requested resource was not found.")
                .into_response())
        })
    }
}
