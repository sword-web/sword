use super::JsonResponse;
use super::RequestError;

use axum::{
    RequestPartsExt,
    body::Body,
    extract::{RawPathParams, Request as AxumReq},
    http::request::Parts,
};

use sword_layers::body_limit::BodyLimitValue;

pub(super) struct PreparedRequestParts {
    pub params: RawPathParams,
    pub parts: Parts,
    pub body: Body,
    pub body_limit: usize,
}

#[allow(async_fn_in_trait)]
pub(super) trait PartsExtractionExt {
    fn body_limit(&self) -> usize;
    fn validate_content_length(&self, body_limit: usize) -> Result<(), RequestError>;
    async fn extract_path_params(&mut self) -> Result<RawPathParams, JsonResponse>;
}

impl PartsExtractionExt for Parts {
    fn body_limit(&self) -> usize {
        self.extensions
            .get::<BodyLimitValue>()
            .cloned()
            .unwrap_or_default()
            .0
    }

    fn validate_content_length(&self, body_limit: usize) -> Result<(), RequestError> {
        let Some(content_length) = self.headers.get("content-length") else {
            return Ok(());
        };

        let cl_str = content_length.to_str().map_err(|_| {
            RequestError::parse_error(
                "Invalid Content-Length header",
                "Header contains invalid format",
            )
        })?;

        let size = cl_str.parse::<usize>().map_err(|_| {
            RequestError::parse_error(
                "Invalid Content-Length header",
                "Header value must be a valid number",
            )
        })?;

        if size > body_limit {
            return Err(RequestError::BodyTooLarge);
        }

        Ok(())
    }

    async fn extract_path_params(&mut self) -> Result<RawPathParams, JsonResponse> {
        self.extract::<RawPathParams>()
            .await
            .map_err(|e| JsonResponse::status(e.status()).message(e.body_text()))
    }
}

#[allow(async_fn_in_trait)]
pub(super) trait AxumRequestPreparationExt {
    async fn prepare(self) -> Result<PreparedRequestParts, JsonResponse>;
}

impl AxumRequestPreparationExt for AxumReq {
    async fn prepare(self) -> Result<PreparedRequestParts, JsonResponse> {
        let (mut parts, body) = self.into_parts();

        let params = parts.extract_path_params().await?;
        let body_limit = parts.body_limit();
        parts
            .validate_content_length(body_limit)
            .map_err(JsonResponse::from)?;

        Ok(PreparedRequestParts {
            params,
            parts,
            body,
            body_limit,
        })
    }
}
