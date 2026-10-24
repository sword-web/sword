use sword_core::Shutdown;
use sword_core::diagnostic::Error;

/// One runnable piece of the application.
pub(crate) enum Engine {
    #[cfg(any(feature = "web", feature = "socketio"))]
    Web(sword_web::application::WebApplication),

    #[cfg(feature = "grpc")]
    Grpc(sword_grpc::application::GrpcApplication),

    #[cfg(any(feature = "events-memory", feature = "events-nats"))]
    Events(sword_events::EventApplication),

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

            #[cfg(any(feature = "events-memory", feature = "events-nats"))]
            Engine::Events(_) => "events",

            #[cfg(feature = "jobs")]
            Engine::Jobs(_) => "jobs",

            _ => "unknown",
        }
    }

    /// Runs the engine until `shutdown` is cancelled.
    ///
    /// Engines that have not been migrated yet still report their own failures
    /// by returning `()`; their arms wrap that in `Ok(())` for now.
    #[allow(unused_variables)]
    pub(crate) async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(any(feature = "web", feature = "socketio"))]
            Engine::Web(web) => web.run(shutdown).await,

            #[cfg(feature = "grpc")]
            Engine::Grpc(grpc) => {
                grpc.run(shutdown).await;
                Ok(())
            }

            #[cfg(any(feature = "events-memory", feature = "events-nats"))]
            Engine::Events(events) => {
                events.run(shutdown).await;
                Ok(())
            }

            #[cfg(feature = "jobs")]
            Engine::Jobs(jobs) => {
                jobs.run(shutdown).await;
                Ok(())
            }

            _ => Ok(()),
        }
    }
}
