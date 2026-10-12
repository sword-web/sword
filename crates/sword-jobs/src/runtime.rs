use crate::{JobControllerRegistrar, JobFunctionRegistrar};

use std::{any::TypeId, collections::HashSet};
use sword_core::{Controller, EngineBuildContext, Shutdown, State, sword_error};
use tokio_cron_scheduler::JobScheduler;

/// Runtime that builds job controllers, registers their jobs and keeps the
/// scheduler alive.
///
/// The scheduler is created lazily in [`Self::start`] because its constructor
/// is async, while the application builder is synchronous.
#[derive(Clone)]
pub struct JobSchedulerRuntime {
    state: State,
    controller_ids: HashSet<TypeId>,
}

impl JobSchedulerRuntime {
    /// Creates the runtime from the state and the set of job controller
    /// `TypeId`s the user registered in a module.
    pub fn new(state: State, controller_ids: HashSet<TypeId>) -> Self {
        Self {
            state,
            controller_ids,
        }
    }

    /// Builds the registered controllers, adds their jobs and starts the
    /// scheduler.
    ///
    /// Returns the running scheduler so the caller can shut it down gracefully.
    pub async fn start(&self) -> JobScheduler {
        tracing::info!(
            target: "sword.startup.job-scheduler",
            controllers = self.controller_ids.len(),
            "Starting sword job scheduler"
        );

        let scheduler = JobScheduler::new().await.unwrap_or_else(|err| {
            sword_error! {
                title: "Unexpected error while initializing sword's job scheduler",
                reason: err,
            }
        });

        // Build only the controllers the user actually registered in a module.
        for registrar in inventory::iter::<JobControllerRegistrar>() {
            if self.controller_ids.contains(&registrar.type_id) {
                (registrar.build_fn)(&self.state);
            }
        }

        // Register every job that belongs to a registered controller.
        for registrar in inventory::iter::<JobFunctionRegistrar>() {
            if !self.controller_ids.contains(&registrar.controller_type_id) {
                continue;
            }

            let job = (registrar.handler)(&self.state);

            if let Err(err) = scheduler.add(job).await {
                tracing::error!(
                    target: "sword.jobs",
                    error = %err,
                    "Failed to add job to the scheduler"
                );
            }
        }

        scheduler.start().await.unwrap_or_else(|err| {
            sword_error! {
                title: "Failed to start sword's job scheduler",
                reason: err,
            }
        });

        scheduler
    }

    /// Runs the scheduler until `shutdown` is cancelled, then stops it.
    pub async fn run(self, shutdown: Shutdown) {
        let mut scheduler = self.start().await;

        shutdown.cancelled().await;

        if let Err(err) = scheduler.shutdown().await {
            tracing::error!(
                target: "sword.jobs",
                error = %err,
                "Failed to shut down job scheduler"
            );
        }
    }
}

impl From<EngineBuildContext> for JobSchedulerRuntime {
    fn from(value: EngineBuildContext) -> Self {
        Self {
            state: value.state,
            controller_ids: value.controllers.get_by_kind(Controller::JobHandler),
        }
    }
}
