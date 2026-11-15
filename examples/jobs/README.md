# Sword Jobs Example

Scheduled background tasks with the `jobs` application type. No database, no
external service: run it and watch the schedules fire.

```bash
cd examples/jobs
cargo run
```

## What it shows

| Attribute | Kind | Schedule in this example |
| --- | --- | --- |
| `#[cron]` | Cron expression | Every minute, on the second 0 |
| `#[repeated]` | Fixed interval | Every 5 seconds |
| `#[once]` | One-shot | Once, 10 seconds after startup |

Each handler runs inside the job engine of the application, not in a web
request, and receives the job id and a `JobScheduler` handle so it can schedule
or cancel other jobs.

```rust
#[controller(kind = Controller::Jobs)]
pub struct MaintenanceJob;

impl MaintenanceJob {
    #[cron("0 * * * * *")]
    async fn every_minute(&self, uuid: Uuid, _: JobScheduler) {
        tracing::info!(%uuid, "cron: runs once a minute");
    }
}
```

Jobs are registered like any other controller, from a module:

```rust
impl Module for JobsModule {
    fn register_controllers(controllers: &ControllerRegistry) {
        controllers.register::<MaintenanceJob>();
    }
}
```

The scheduler starts with the application and shuts down with it. Handlers can
live in the same process as web or gRPC controllers and use the same dependency
container.
