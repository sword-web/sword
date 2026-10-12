mod runtime;

use std::any::TypeId;
use sword_core::State;
use tokio_cron_scheduler::Job;

pub use runtime::JobSchedulerRuntime;

/// The kind of schedule declared by a job handler.
pub enum JobKind {
    Cron,
    Repeated,
    Once,
}

/// Builds the `Job` for a single handler method.
pub type JobHandlerFn = fn(&State) -> Job;

/// Builds a job controller into the `State`.
pub struct JobControllerRegistrar {
    pub type_id: TypeId,
    pub build_fn: fn(&State),
}

/// Registers a single job belonging to a controller.
pub struct JobFunctionRegistrar {
    pub handler: JobHandlerFn,
    pub controller_type_id: TypeId,
}

inventory::collect!(JobControllerRegistrar);
inventory::collect!(JobFunctionRegistrar);

pub mod prelude {
    pub use sword_macros::{cron, once, repeated};
    pub use tokio_cron_scheduler::{Job, JobScheduler};
    pub use uuid::Uuid;
}

#[doc(hidden)]
pub mod internal {
    pub use super::JobHandlerFn;
    pub use super::JobKind;
    pub use super::JobSchedulerRuntime;
    pub use super::{JobControllerRegistrar, JobFunctionRegistrar};
    pub use tokio_cron_scheduler::Job;
}
