use super::{
    Request, RequestError, StreamRequest,
    parts::{AxumRequestPreparationExt, PreparedRequestParts},
};
use sword_core::State;

use axum::{
    body::to_bytes,
    extract::{FromRef, FromRequest as AxumFromRequest, Request as AxumReq},
    http::request::Parts,
    response::IntoResponse,
};
use axum_responses::JsonResponse;

#[allow(async_fn_in_trait)]
/// Fixed-state version of `axum::extract::FromRequest` using sword's State.
///
/// This allows extractors to avoid the generic state parameter while still
/// leveraging Axum's extractor ecosystem.
pub trait FromRequest: Sized {
    type Rejection: IntoResponse;

    async fn from_request(req: AxumReq, state: &State) -> Result<Self, Self::Rejection>;
}

#[allow(async_fn_in_trait)]
/// Fixed-state version of `axum::extract::FromRequestParts` using sword's State.
pub trait FromRequestParts: Sized {
    type Rejection: IntoResponse;

    async fn from_request_parts(parts: &mut Parts, state: &State) -> Result<Self, Self::Rejection>;
}

/// Implementation of `FromRequest` for `Request`.
///
/// Allows `Request` to be automatically extracted from HTTP requests
/// in Axum handlers, providing easy access to parameters, headers, body, and state.
impl FromRequest for Request {
    type Rejection = JsonResponse;

    async fn from_request(req: AxumReq, _: &State) -> Result<Self, Self::Rejection> {
        let PreparedRequestParts {
            params,
            parts,
            body,
            body_limit,
        } = req.prepare().await?;

        let body_bytes = to_bytes(body, body_limit).await.map_err(|err| {
            let inner = err.into_inner();

            RequestError::from_body_read_error(inner.as_ref())
        })?;

        Ok(Self {
            params,
            body_bytes,
            method: parts.method,
            headers: parts.headers,
            uri: parts.uri,
            extensions: parts.extensions,
            next: None,
        })
    }
}

impl FromRequest for StreamRequest {
    type Rejection = JsonResponse;

    async fn from_request(req: AxumReq, _: &State) -> Result<Self, Self::Rejection> {
        let PreparedRequestParts {
            params,
            parts,
            body,
            body_limit,
        } = req.prepare().await?;

        Ok(Self {
            params,
            body,
            method: parts.method,
            headers: parts.headers,
            uri: parts.uri,
            extensions: parts.extensions,
            next: None,
            body_limit,
        })
    }
}

/// Implementation of conversion from `Request` to `axum::extract::Request`.
///
/// Allows converting a `sword::web::Request` back to an Axum request,
/// preserving headers, method, URI, body, and extensions.
impl TryFrom<Request> for AxumReq {
    type Error = RequestError;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        use axum::body::Body;

        let mut builder = AxumReq::builder().method(req.method).uri(req.uri);

        for (key, value) in &req.headers {
            builder = builder.header(key, value);
        }

        let body = Body::from(req.body_bytes);

        let mut request = builder.body(body).map_err(|_| {
            RequestError::parse_error(
                "Failed to build axum request",
                "Error building request".to_string(),
            )
        })?;

        *request.extensions_mut() = req.extensions;

        Ok(request)
    }
}

impl TryFrom<StreamRequest> for AxumReq {
    type Error = RequestError;

    fn try_from(req: StreamRequest) -> Result<Self, Self::Error> {
        let mut builder = AxumReq::builder().method(req.method).uri(req.uri);

        for (key, value) in &req.headers {
            builder = builder.header(key, value);
        }

        let mut request = builder.body(req.body).map_err(|_| {
            RequestError::parse_error(
                "Failed to build axum request",
                "Error building request".to_string(),
            )
        })?;

        *request.extensions_mut() = req.extensions;

        Ok(request)
    }
}

/// Implementation for compatibility with Axum's generic state system
impl<S> AxumFromRequest<S> for Request
where
    S: Send + Sync + 'static,
    State: FromRef<S>,
{
    type Rejection = JsonResponse;

    async fn from_request(req: AxumReq, state: &S) -> Result<Self, Self::Rejection> {
        let state = State::from_ref(state);
        <Self as FromRequest>::from_request(req, &state).await
    }
}

impl<S> AxumFromRequest<S> for StreamRequest
where
    S: Send + Sync + 'static,
    State: FromRef<S>,
{
    type Rejection = JsonResponse;

    async fn from_request(req: AxumReq, state: &S) -> Result<Self, Self::Rejection> {
        let state = State::from_ref(state);
        <Self as FromRequest>::from_request(req, &state).await
    }
}
