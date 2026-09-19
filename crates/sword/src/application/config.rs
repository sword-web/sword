pub use sword_core::ApplicationConfig;

/// Engine of the running application.
///
/// Sword runs one application type at a time, selected by feature flag.
pub enum ApplicationEngine {
    /// Web application built on axum. Serves `Controller::Web` controllers and,
    /// when the socketio feature is enabled, Socket.IO over the same router.
    #[cfg(any(feature = "web", feature = "socketio"))]
    Web(sword_web::application::WebApplication),

    /// gRPC application built on tonic.
    #[cfg(feature = "grpc")]
    Grpc(sword_grpc::application::GrpcApplication),
}
