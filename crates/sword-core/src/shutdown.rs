/// Waits for a shutdown signal (SIGINT or SIGTERM) and logs which signal was received.
///
/// This is a shared utility used by both web and gRPC application engines.
/// On Unix systems, it listens for both `Ctrl+C` (SIGINT) and `SIGTERM`.
/// On non-Unix systems, it only listens for `Ctrl+C`.
pub async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c().await.unwrap_or_else(|err| {
            tracing::error!(
                target: "sword.shutdown",
                error = %err,
                "Failed to install Ctrl+C handler"
            );
        });
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
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
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!(
                target: "sword.startup.signal",
                signal = "SIGINT",
                "Shutdown signal received, starting graceful shutdown"
            );
        },
        _ = terminate => {
            tracing::info!(
                target: "sword.startup.signal",
                signal = "SIGTERM",
                "Shutdown signal received, starting graceful shutdown"
            );
        },
    }
}
