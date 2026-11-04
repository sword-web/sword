use super::{
    JsonResponse, Request, RequestError, StreamRequest,
    parts::{AxumRequestPreparationExt, PreparedRequestParts},
};
use bytes::Bytes;
use sword_core::State;

use axum::{
    body::to_bytes,
    extract::{FromRef, FromRequest as AxumFromRequest, Request as AxumReq},
    http::{Method, request::Parts},
    response::IntoResponse,
};

/// Fixed-state version of `axum::extract::FromRequest` using sword's State.
///
/// This allows extractors to avoid the generic state parameter while still
/// leveraging Axum's extractor ecosystem.
pub trait FromRequest: Sized {
    type Rejection: IntoResponse;

    fn from_request(
        req: AxumReq,
        state: &State,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}

/// Fixed-state version of `axum::extract::FromRequestParts` using sword's State.
pub trait FromRequestParts: Sized {
    type Rejection: IntoResponse;

    fn from_request_parts(
        parts: &mut Parts,
        state: &State,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}

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

        let body_bytes = match parts.method {
            Method::POST | Method::PUT | Method::PATCH => {
                to_bytes(body, body_limit).await.map_err(|err| {
                    let inner = err.into_inner();
                    RequestError::from_body_read_error(inner.as_ref())
                })?
            }
            _ => Bytes::default(),
        };

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
        let mut out = AxumReq::new(axum::body::Body::from(req.body_bytes));

        *out.method_mut() = req.method;
        *out.uri_mut() = req.uri;
        *out.headers_mut() = req.headers;
        *out.extensions_mut() = req.extensions;

        Ok(out)
    }
}

impl TryFrom<StreamRequest> for AxumReq {
    type Error = RequestError;

    fn try_from(req: StreamRequest) -> Result<Self, Self::Error> {
        let mut out = AxumReq::new(req.body);

        *out.method_mut() = req.method;
        *out.uri_mut() = req.uri;
        *out.headers_mut() = req.headers;
        *out.extensions_mut() = req.extensions;

        Ok(out)
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
