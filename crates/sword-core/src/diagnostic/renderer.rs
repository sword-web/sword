use super::Error;
use super::types::{DiagnosticPhase, DiagnosticsMode};
use parking_lot::RwLock;
use std::sync::OnceLock;

/// The renderer currently used for process-level diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Renderer {
    Miette,
    Tracing,
}

/// Process-wide renderer selection.
///
/// It lives behind a lock rather than a plain `OnceLock` so tests and embedders
/// can switch modes between applications. In a normal binary it is set once,
/// during bootstrap.
static RENDERER: OnceLock<RwLock<Renderer>> = OnceLock::new();

fn state() -> &'static RwLock<Renderer> {
    RENDERER.get_or_init(|| RwLock::new(Renderer::Miette))
}

pub(crate) fn set(renderer: Renderer) {
    *state().write() = renderer;
}

pub(crate) fn current() -> Renderer {
    *state().read()
}

impl DiagnosticsMode {
    /// Installs the renderer for this process.
    ///
    /// `tracing` only makes sense once a subscriber is active. If it is not, we
    /// return a configuration error instead of silently losing diagnostics.
    pub fn apply(self) -> Result<(), Error> {
        let renderer = match self {
            Self::Miette => Renderer::Miette,
            Self::Tracing if tracing::dispatcher::has_been_set() => Renderer::Tracing,
            Self::Tracing => {
                return Err(crate::sword_error! {
                    code: "sword.build.diagnostics",
                    phase: DiagnosticPhase::Build,
                    target: "diagnostics configuration",
                    title: "Invalid diagnostics configuration",
                    reason: "diagnostics = \"tracing\" requires an active tracing subscriber",
                    hints: [
                        "Set [tracing] enabled = true, or use diagnostics = \"miette\"",
                    ],
                });
            }
        };

        set(renderer);

        Ok(())
    }
}
