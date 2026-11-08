mod runtime;

pub use runtime::JobSchedulerRuntime;

pub mod prelude {
    pub use sword_macros::{cron, once, repeated};
    pub use tokio_cron_scheduler::{Job, JobScheduler};
    pub use uuid::Uuid;
}

/// Items required by generated code.
///
/// This module is public so procedural macros can name these items from the
/// user's crate, but it is not part of the stable API.
#[doc(hidden)]
pub mod internal {
    pub use crate::runtime::{
        JobControllerRegistrar, JobFunctionRegistrar, JobHandlerFn, JobSchedulerRuntime,
    };
    pub use tokio_cron_scheduler::Job;
}
