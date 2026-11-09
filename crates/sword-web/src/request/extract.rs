use axum::{
    RequestPartsExt,
    body::{Body as AxumBody, to_bytes},
    extract::{FromRef, FromRequest as AxumFromRequest, RawPathParams, Request as AxumRequest},
    http::Method,
    response::IntoResponse,
};
use bytes::Bytes;

use sword_core::State;
use sword_layers::body_limit::BodyLimitValue;

use super::{JsonResponse, Request, RequestError, StreamRequest};

/// Fixed-state version of `axum::extract::FromRequest` using sword's State.
///
/// This allows extractors to avoid the generic state parameter while still
/// leveraging Axum's extractor ecosystem.
pub trait FromRequest: Sized {
    type Rejection: IntoResponse;

    fn from_request(
        req: AxumRequest,
        state: &State,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send;
}

/// Allows `Request` to be automatically extracted from HTTP requests
/// in Axum handlers, providing easy access to parameters, headers, body, and state.
impl FromRequest for Request {
    type Rejection = JsonResponse;

    async fn from_request(req: AxumRequest, state: &State) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let params = parts
            .extract::<RawPathParams>()
            .await
            .map_err(|e| JsonResponse::status(e.status()).message(e.body_text()))?;

        let body_bytes = match parts.method {
            Method::POST | Method::PUT | Method::PATCH => {
                let body_limit = state
                    .borrow::<BodyLimitValue>()
                    .map_or_else(|_| BodyLimitValue::default().0, |value| value.0);

                to_bytes(body, body_limit).await.map_err(|err| {
                    let inner = err.into_inner();
                    RequestError::from_body_read_error(inner.as_ref())
                })?
            }
            _ => Bytes::default(),
        };

        Ok(Self {
            parts,
            params,
            body_bytes,
            next: None,
        })
    }
}

impl FromRequest for StreamRequest {
    type Rejection = JsonResponse;

    async fn from_request(req: AxumRequest, state: &State) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();

        let params = parts
            .extract::<RawPathParams>()
            .await
            .map_err(|e| JsonResponse::status(e.status()).message(e.body_text()))?;

        let body_limit = state
            .borrow::<BodyLimitValue>()
            .map_or_else(|_| BodyLimitValue::default().0, |value| value.0);

        Ok(Self {
            parts,
            params,
            body,
            next: None,
            body_limit,
        })
    }
}

/// Implementation of conversion from `Request` to `axum::extract::Request`.
///
/// Allows converting a `sword::web::Request` back to an Axum request,
/// preserving headers, method, URI, body, and extensions.
impl TryFrom<Request> for AxumRequest {
    type Error = RequestError;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        Ok(AxumRequest::from_parts(
            req.parts,
            AxumBody::from(req.body_bytes),
        ))
    }
}

impl TryFrom<StreamRequest> for AxumRequest {
    type Error = RequestError;

    fn try_from(req: StreamRequest) -> Result<Self, Self::Error> {
        Ok(AxumRequest::from_parts(req.parts, req.body))
    }
}

/// Implementation for compatibility with Axum's generic state system
impl<S> AxumFromRequest<S> for Request
where
    S: Send + Sync + 'static,
    State: FromRef<S>,
{
    type Rejection = JsonResponse;

    async fn from_request(req: AxumRequest, state: &S) -> Result<Self, Self::Rejection> {
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

    async fn from_request(req: AxumRequest, state: &S) -> Result<Self, Self::Rejection> {
        let state = State::from_ref(state);
        <Self as FromRequest>::from_request(req, &state).await
    }
}
