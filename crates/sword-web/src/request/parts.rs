use super::RequestError;
use axum::{
    RequestPartsExt,
    body::Body,
    extract::{Path, Request as AxumReq, rejection::PathRejection},
    http::request::Parts,
};
use axum_responses::JsonResponse;
use std::collections::HashMap;
use sword_layers::body_limit::BodyLimitValue;

pub(super) struct PreparedRequestParts {
    pub params: HashMap<String, String>,
    pub parts: Parts,
    pub body: Body,
    pub body_limit: usize,
}

#[allow(async_fn_in_trait)]
pub(super) trait PartsExtractionExt {
    fn body_limit(&self) -> usize;
    fn validate_content_length(&self, body_limit: usize) -> Result<(), RequestError>;
    async fn extract_path_params(&mut self) -> Result<HashMap<String, String>, JsonResponse>;
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

    async fn extract_path_params(&mut self) -> Result<HashMap<String, String>, JsonResponse> {
        let path_params = self
            .extract::<Path<HashMap<String, String>>>()
            .await
            .map_err(|e| {
                let message = match e {
                    PathRejection::FailedToDeserializePathParams(_) => {
                        "Failed to deserialize path parameters".to_string()
                    }
                    PathRejection::MissingPathParams(m) => m.body_text().to_string(),
                    _ => "Failed to extract path parameters".to_string(),
                };

                JsonResponse::BadRequest().message(message)
            })?;

        Ok(path_params.0)
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
