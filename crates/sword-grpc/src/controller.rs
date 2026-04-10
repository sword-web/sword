use crate::registry::GrpcServiceRegistry;
pub use sword_macros::GrpcError;

use std::any::TypeId;
use std::pin::Pin;
use stream::Stream;
use sword_core::State;

pub mod stream {
    pub use tokio_stream::{Stream, StreamExt, wrappers::ReceiverStream};
}

pub use crate::response::GrpcResponse;
pub use sword_layers::body_limit::GrpcBodyLimitValue;
pub use tonic::{Code, Extensions, Request, Response, Status, Streaming, include_proto};

use sword_core::ControllerSpec;

pub type GrpcResult<T> = Result<Response<T>, Status>;
pub type GrpcStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

/// Trait implemented by gRPC controllers declared with `#[controller(kind = Controller::Grpc, ...)]`.
pub trait GrpcController: ControllerSpec {
    fn service_name() -> &'static str;
}

#[derive(Clone)]
pub struct GrpcControllerRegistrar {
    pub controller_id: TypeId,
    pub service_name: &'static str,
    pub reflection_descriptor_set: Option<&'static [u8]>,
    pub build: fn(&State),
    pub register: fn(&State, &mut GrpcServiceRegistry),
}

inventory::collect!(GrpcControllerRegistrar);
