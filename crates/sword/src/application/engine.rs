use sword_core::Shutdown;

/// One runnable piece of the application.
///
/// The set is closed (framework-owned), so a plain enum with dispatch is enough:
/// no `dyn`, no boxing and no `async` trait.
pub(crate) enum Engine {
    #[cfg(any(feature = "web", feature = "socketio"))]
    Web(sword_web::application::WebApplication),

    #[cfg(feature = "grpc")]
    Grpc(sword_grpc::application::GrpcApplication),

    #[cfg(feature = "events")]
    Events(sword_events::EventEngine),

    #[cfg(feature = "jobs")]
    Jobs(sword_jobs::JobSchedulerRuntime),
}

impl Engine {
    pub(crate) fn name(&self) -> &'static str {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(any(feature = "web", feature = "socketio"))]
            Engine::Web(_) => "web",

            #[cfg(feature = "grpc")]
            Engine::Grpc(_) => "grpc",

            #[cfg(feature = "events")]
            Engine::Events(_) => "events",

            #[cfg(feature = "jobs")]
            Engine::Jobs(_) => "jobs",

            _ => "unknown",
        }
    }

    /// Runs the engine until `shutdown` is cancelled.
    #[allow(unused_variables)]
    pub(crate) async fn run(self, shutdown: Shutdown) {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(any(feature = "web", feature = "socketio"))]
            Engine::Web(web) => web.run(shutdown).await,

            #[cfg(feature = "grpc")]
            Engine::Grpc(grpc) => grpc.run(shutdown).await,

            #[cfg(feature = "events")]
            Engine::Events(events) => events.run(shutdown).await,

            #[cfg(feature = "jobs")]
            Engine::Jobs(jobs) => jobs.run(shutdown).await,

            _ => {}
        }
    }
}
