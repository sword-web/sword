mod runtime;

use std::{pin::Pin, sync::Arc};
use sword_core::State;
use tokio_cron_scheduler::JobScheduler;

type BoxedFuture = Pin<Box<dyn Future<Output = ()> + Send>>;

pub struct JobControllerRegistrar {
    pub build_fn: fn(&State),
}

pub struct JobFunctionRegistrar {
    pub register_cron_fn: fn(&State, &JobScheduler) -> Arc<dyn Fn() -> BoxedFuture>,
}

inventory::collect!(JobControllerRegistrar);
inventory::collect!(JobFunctionRegistrar);

pub mod prelude {}

#[doc(hidden)]
pub mod internal {
    pub use super::{JobControllerRegistrar, JobFunctionRegistrar};
}
