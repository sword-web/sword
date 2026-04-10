//! Not found response mapping middleware.
//!
//! This module provides a layer that transforms plain `404 Not Found`
//! responses into Sword's standardized JSON response format.

use crate::ResponseFnMapper;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use axum_responses::JsonResponse;
use tower::{ServiceBuilder, util::MapResponseLayer};
use tower_layer::{Identity, Stack};

pub struct NotFoundLayer;

pub type ResponseFnMapperServiceLayer =
    ServiceBuilder<Stack<MapResponseLayer<ResponseFnMapper>, Identity>>;

impl NotFoundLayer {
    pub fn new() -> ResponseFnMapperServiceLayer {
        ServiceBuilder::new().map_response(|r: Response| {
            if r.status() != StatusCode::NOT_FOUND {
                return r;
            }

            JsonResponse::NotFound()
                .message("The requested resource was not found.")
                .into_response()
        })
    }
}
