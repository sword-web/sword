pub use sword_core::ApplicationConfig;

pub enum ApplicationEngine {
    #[cfg(any(feature = "web", feature = "socketio"))]
    Web(sword_web::application::WebApplication),

    #[cfg(feature = "grpc")]
    Grpc(sword_grpc::application::GrpcApplication),
}
