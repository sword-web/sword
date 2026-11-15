use std::time::Duration;

use sword::jobs::{JobScheduler, Uuid, cron, once, repeated};
use sword::prelude::*;

/// A job controller groups the scheduled tasks of one capability. The three
/// attributes map to the three kinds of schedule Sword supports.
#[controller(kind = Controller::Jobs)]
pub struct MaintenanceJob;

impl MaintenanceJob {
    /// Cron expression: second minute hour day month weekday.
    /// This one runs every minute, on the second 0.
    #[cron("0 * * * * *")]
    async fn every_minute(&self, uuid: Uuid, _: JobScheduler) {
        tracing::info!(%uuid, "cron: runs once a minute");
    }

    /// A repeated task re-runs at a fixed interval.
    #[repeated(Duration::from_secs(5))]
    async fn heartbeat(&self, _: Uuid, _: JobScheduler) {
        tracing::info!("repeated: heartbeat every 5 seconds");
    }

    /// A one-shot task runs once, after the given delay.
    #[once(Duration::from_secs(10))]
    async fn warmup(&self, _: Uuid, _: JobScheduler) {
        tracing::info!("once: warmup finished");
    }
}

pub struct JobsModule;

impl Module for JobsModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<MaintenanceJob>();
    }
}
