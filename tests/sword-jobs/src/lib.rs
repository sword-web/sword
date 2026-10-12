use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use sword::jobs::{JobScheduler, Uuid, repeated};
use sword::prelude::*;

static RUNS: AtomicUsize = AtomicUsize::new(0);

#[controller(kind = Controller::JobHandler)]
pub struct TestJob {}

impl TestJob {
    #[repeated(Duration::from_millis(50))]
    async fn tick(&self, _: Uuid, _: JobScheduler) {
        RUNS.fetch_add(1, Ordering::SeqCst);
    }
}

#[tokio::test]
async fn repeated_job_runs() {
    let state = sword::internal::core::State::new();
    let registry = ControllerRegistry::new();
    registry.register::<TestJob>();

    let runtime = sword::internal::jobs::JobSchedulerRuntime::new(
        state,
        registry.get_by_kind(Controller::JobHandler),
    );

    let mut scheduler = runtime.start().await;

    // The scheduler ticks every 500ms, so allow some slack.
    tokio::time::sleep(Duration::from_millis(1500)).await;

    assert!(RUNS.load(Ordering::SeqCst) >= 1, "job did not run");

    scheduler.shutdown().await.unwrap();
}
