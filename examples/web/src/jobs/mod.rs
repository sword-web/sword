use std::time::Duration;
use sword::jobs::{JobScheduler, Uuid, cron, once, repeated};
use sword::prelude::*;

/// Example job controller exercising the three scheduling attributes.
#[controller(kind = Controller::Jobs)]
pub struct CleanupJob {}

impl CleanupJob {
    /// Runs every hour (cron: second minute hour day month weekday).
    #[cron("0 0 * * * *")]
    async fn hourly(&self, uuid: Uuid, _: JobScheduler) {
        tracing::info!(%uuid, "cleanup job: hourly tick");
    }

    /// Runs every 5 seconds.
    #[repeated(Duration::from_secs(5))]
    async fn heartbeat(&self, _: Uuid, _: JobScheduler) {
        tracing::info!("cleanup job: heartbeat");
    }

    /// Runs once, 10 seconds after startup.
    #[once(Duration::from_secs(10))]
    async fn warmup(&self, _: Uuid, _: JobScheduler) {
        tracing::info!("cleanup job: warmup");
    }
}

pub struct JobsModule;

impl Module for JobsModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<CleanupJob>();
    }
}
