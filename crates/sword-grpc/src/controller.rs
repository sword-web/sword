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
pub trait GrpcController: ControllerSpec {}

/// Registrar emitted by the `#[controller(kind = Controller::Grpc, ...)]` macro
/// and collected into the global inventory at compile time.
#[derive(Clone)]
pub struct GrpcControllerRegistrar {
    /// Join key: matches a controller registered in `ControllerRegistry` with
    /// this metadata at runtime.
    pub controller_id: TypeId,

    /// Proto descriptor blob embedded by the macro (`build.rs` output) when
    /// `grpc-reflection` is enabled, used by the reflection service.
    pub reflection_descriptor_set: Option<&'static [u8]>,

    /// Constructs the controller from the DI `State` and inserts it as
    /// `Arc<Controller>`. Runs before `register`.
    ///
    /// ```rust,ignore
    /// build: |state| {
    ///     state.insert::<Greeter>(Greeter::build(state).unwrap_or_else(|e| panic_sword_error(e)));
    /// }
    /// ```
    pub build: fn(&State),

    /// Borrows the built controller and body limit from `State`, wraps the
    /// tonic service with the declared interceptors, and adds it to the
    /// registry.
    ///
    /// ```rust,ignore
    /// register: |state, registry| {
    ///     let ctrl  = state.borrow::<Greeter>()?;            // inserted by `build`
    ///     let limit = state.borrow::<GrpcBodyLimitValue>()?; // set at startup
    ///     let svc = GreeterServer::new((*ctrl).clone())
    ///         .max_decoding_message_size(limit.max_decoding_message_size);
    ///
    ///     registry.routes_builder_mut().add_service(svc);
    ///     registry.mark_service_registered_with_name(GreeterServer::NAME);
    /// }
    /// ```
    pub register: fn(&State, &mut GrpcServiceRegistry),
}

inventory::collect!(GrpcControllerRegistrar);
