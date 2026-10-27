use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A stable identifier for a diagnostic.
///
/// Codes are kept stable so users can search for them, link to them, or match
/// them in tests and alerts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DiagnosticCode(&'static str);

impl DiagnosticCode {
    pub const fn new(code: &'static str) -> Self {
        Self(code)
    }

    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl Display for DiagnosticCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl From<&'static str> for DiagnosticCode {
    fn from(code: &'static str) -> Self {
        Self::new(code)
    }
}

/// How loud a diagnostic is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Severity {
    #[default]
    Error,
    Warning,
    Advice,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Advice => "advice",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Error => "Error",
            Self::Warning => "Warning",
            Self::Advice => "Advice",
        }
    }
}

impl Display for Severity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The point in the application lifecycle where a diagnostic was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticPhase {
    Build,
    Startup,
    Runtime,
    Shutdown,
}

impl DiagnosticPhase {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Startup => "startup",
            Self::Runtime => "runtime",
            Self::Shutdown => "shutdown",
        }
    }

    pub const fn title(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Startup => "Startup",
            Self::Runtime => "Runtime",
            Self::Shutdown => "Shutdown",
        }
    }
}

impl Display for DiagnosticPhase {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// The renderer used for process-level diagnostics.
#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticsMode {
    /// Human-readable reports, rendered with miette.
    #[default]
    Miette,

    /// Structured events sent to the tracing subscriber.
    Tracing,
}
