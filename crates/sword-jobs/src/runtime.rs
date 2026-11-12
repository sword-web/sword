use std::{any::TypeId, collections::HashSet};

use tokio_cron_scheduler::{Job, JobScheduler};

use sword_core::*;

/// Builds the `Job` for a single handler method.
pub type JobHandlerFn = fn(&State) -> Result<Job, Error>;

/// Builds a job controller into the `State`.
pub struct JobControllerRegistrar {
    pub type_id: TypeId,
    pub build_fn: fn(&State) -> Result<(), Error>,
}

/// Registers a single job belonging to a controller.
pub struct JobFunctionRegistrar {
    pub handler: JobHandlerFn,
    pub controller_type_id: TypeId,
}

inventory::collect!(JobControllerRegistrar);
inventory::collect!(JobFunctionRegistrar);

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
    pub async fn start(&self) -> Result<JobScheduler, Error> {
        tracing::info!(
            target: "sword.startup.job-scheduler",
            controllers = self.controller_ids.len(),
            "Starting sword job scheduler"
        );

        let scheduler = JobScheduler::new().await.map_err(|err| {
            sword_error! {
                code: "sword.runtime.jobs",
                phase: DiagnosticPhase::Runtime,
                target: "job scheduler",
                title: "Could not initialize the job scheduler",
                reason: err,
                source: "JobSchedulerRuntime::start",
                hints: ["Check the job scheduler configuration and available system resources"],
            }
        })?;

        // Build only the controllers the user actually registered in a module.
        for registrar in inventory::iter::<JobControllerRegistrar>() {
            if self.controller_ids.contains(&registrar.type_id) {
                (registrar.build_fn)(&self.state)?;
            }
        }

        // Register every job that belongs to a registered controller.
        for registrar in inventory::iter::<JobFunctionRegistrar>() {
            if !self.controller_ids.contains(&registrar.controller_type_id) {
                continue;
            }

            let job = (registrar.handler)(&self.state)?;

            if let Err(err) = scheduler.add(job).await {
                tracing::error!(
                    target: "sword.jobs",
                    error = %err,
                    "Failed to add job to the scheduler"
                );
            }
        }

        scheduler.start().await.map_err(|err| {
            sword_error! {
                code: "sword.runtime.jobs",
                phase: DiagnosticPhase::Runtime,
                target: "job scheduler",
                title: "Could not start the job scheduler",
                reason: err,
                source: "JobSchedulerRuntime::start",
                hints: ["Check the registered job schedules"],
            }
        })?;

        Ok(scheduler)
    }

    /// Runs the scheduler until `shutdown` is cancelled.
    pub async fn run(self, shutdown: Shutdown) -> Result<(), Error> {
        let _scheduler = self.start().await?;

        shutdown.cancelled().await;

        Ok(())
    }
}

impl From<EngineBuildContext> for JobSchedulerRuntime {
    fn from(value: EngineBuildContext) -> Self {
        Self {
            state: value.state,
            controller_ids: value.controllers.get_by_kind(Controller::Jobs),
        }
    }
}
