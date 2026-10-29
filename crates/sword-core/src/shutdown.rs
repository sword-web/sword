use std::future::Future;
#[cfg(not(unix))]
use std::future::pending;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::watch;

/// Waits for a shutdown signal (SIGINT or SIGTERM).
///
/// This is a shared utility used by both web and gRPC application engines.
/// On Unix systems, it listens for both `Ctrl+C` (SIGINT) and `SIGTERM`.
/// On non-Unix systems, it only listens for `Ctrl+C`.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.unwrap_or_else(|err| {
            tracing::error!(
                target: "sword.shutdown",
                error = %err,
                "Failed to install Ctrl+C handler"
            );
        });
    };

    #[cfg(unix)]
    let terminate = async {
        match signal::unix::signal(signal::unix::SignalKind::terminate()) {
            Ok(mut sig) => {
                sig.recv().await;
            }
            Err(err) => {
                tracing::error!(
                    target: "sword.shutdown",
                    error = %err,
                    signal = "SIGTERM",
                    "Failed to install signal handler"
                );
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!(
                target: "sword.startup.signal",
                signal = "SIGINT",
                "Shutdown signal received"
            );
        },
        _ = terminate => {
            tracing::info!(
                target: "sword.startup.signal",
                signal = "SIGTERM",
                "Shutdown signal received"
            );
        },
    }
}

/// Shared shutdown state that every engine observes.
///
/// Built on top of a `watch` channel: `cancel` flips the value to `true` and
/// every [`Shutdown::cancelled`] waiter resolves. Because `watch` keeps the
/// latest value, a late waiter resolves immediately (idempotent, no lost
/// wake-ups).
#[derive(Clone)]
pub struct Shutdown {
    tx: Arc<watch::Sender<bool>>,
}

impl Shutdown {
    pub fn new() -> Self {
        let (tx, _rx) = watch::channel(false);
        Self { tx: Arc::new(tx) }
    }

    /// Marks the application as shutting down.
    pub fn cancel(&self) {
        let _ = self.tx.send(true);
    }

    /// Resolves once [`Shutdown::cancel`] has been called (immediately if it
    /// already happened).
    ///
    /// The returned future is `'static` (it owns a clone of the sender), so it
    /// can be used by long-lived server tasks.
    pub fn cancelled(&self) -> impl Future<Output = ()> + Send + 'static {
        let tx = self.tx.clone();

        async move {
            let mut rx = tx.subscribe();

            if *rx.borrow() {
                return;
            }

            let _ = rx.changed().await;
        }
    }
}

impl Default for Shutdown {
    fn default() -> Self {
        Self::new()
    }
}
