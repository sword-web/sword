use crate::{JobControllerRegistrar, JobFunctionRegistrar};

use std::{any::TypeId, collections::HashSet};
use sword_core::{Controller, EngineBuildContext, State, sword_error};
use tokio_cron_scheduler::JobScheduler;

pub struct JobSchedulerRuntime {
    pub state: State,
    pub controller_ids: HashSet<TypeId>,
}

impl JobSchedulerRuntime {
    pub async fn start(&self) {
        tracing::info!(
            target = "sword.startup.job-scheduler",
            "Starting sword job scheduler"
        );

        let scheduler = JobScheduler::new().await.unwrap_or_else(|e| {
            sword_error! {
                title: "Unexpected error while initializing sword's job scheduler",
                reason: e
            }
        });

        for job_controller in inventory::iter::<JobControllerRegistrar>() {
            (job_controller.build_fn)(&self.state);
        }

        for job in inventory::iter::<JobFunctionRegistrar>() {
            (job.register_cron_fn)(&self.state, &scheduler);
        }
    }
}

impl From<EngineBuildContext> for JobSchedulerRuntime {
    fn from(value: EngineBuildContext) -> Self {
        Self {
            state: value.state,
            controller_ids: value.controllers.get_by_kind(Controller::Job),
        }
    }
}
