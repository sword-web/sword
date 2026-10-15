use super::JsonResponse;

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

        Ok(PreparedRequestParts {
            params,
            parts,
            body,
            body_limit,
        })
    }
}
