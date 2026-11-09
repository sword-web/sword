use std::any::TypeId;
use std::pin::Pin;

use tokio_stream::Stream;
use tonic::{Response, Status};

use sword_core::{ControllerSpec, Error, State};

use crate::registry::GrpcServiceRegistry;

pub type GrpcResult<T> = Result<Response<T>, Status>;
pub type GrpcStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

/// Trait implemented by gRPC controllers declared with `#[controller]`.
pub trait GrpcController: ControllerSpec {}

/// Registrar emitted by the `#[controller]` macro.
#[derive(Clone)]
pub struct GrpcControllerRegistrar {
    pub controller_id: TypeId,
    pub reflection_descriptor_set: Option<&'static [u8]>,
    pub build: fn(&State) -> Result<(), Error>,
    pub register: fn(&State, &mut GrpcServiceRegistry) -> Result<(), Error>,
}

inventory::collect!(GrpcControllerRegistrar);
