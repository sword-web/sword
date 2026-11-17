//! Request ID propagation middleware.
//!
//! This module provides a layer stack that assigns an `x-request-id` to each
//! request and propagates it through the response pipeline.

use tower::ServiceBuilder;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_layer::{Identity, Stack};

type RequestIdServiceType = ServiceBuilder<
    Stack<PropagateRequestIdLayer, Stack<SetRequestIdLayer<MakeRequestUuid>, Identity>>,
>;

pub use tower_http::request_id::RequestId;
pub use uuid::Uuid;

pub struct RequestIdLayer;

impl RequestIdLayer {
    pub fn new() -> RequestIdServiceType {
        ServiceBuilder::new()
            .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
            .layer(PropagateRequestIdLayer::x_request_id())
    }
}
