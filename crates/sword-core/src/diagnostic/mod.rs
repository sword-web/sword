use ::miette::{LabeledSpan, NamedSource};
use std::fmt::{Display, Formatter};

#[macro_use]
mod macros;
mod builder;
mod error;
mod miette;
mod renderer;
mod report;
mod types;

pub use builder::DiagnosticBuilder;
pub use error::Error;
pub use types::{DiagnosticCode, DiagnosticPhase, DiagnosticsMode, Severity};

/// A location in a Rust source file.
///
/// Procedural macros capture this while expanding user code, so diagnostics can
/// point at the item that failed instead of showing a floating message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceLocation {
    file: &'static str,
    line: usize,
    column: usize,
    length: usize,
}

impl SourceLocation {
    pub const fn new(file: &'static str, line: usize, column: usize, length: usize) -> Self {
        Self {
            file,
            line,
            column,
            length,
        }
    }

    pub const fn file(self) -> &'static str {
        self.file
    }

    pub const fn line(self) -> usize {
        self.line
    }

    pub const fn column(self) -> usize {
        self.column
    }

    pub const fn length(self) -> usize {
        self.length
    }
}

/// The content of a process-level diagnostic.
///
/// A diagnostic is data. It does not implement [`std::error::Error`] and is not
/// returned directly: fallible APIs wrap it in [`Error`], and the renderer
/// decides how to present it.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    code: DiagnosticCode,
    severity: Severity,
    phase: Option<DiagnosticPhase>,
    engine: Option<&'static str>,
    target: Option<String>,
    title: String,
    reason: String,
    source: Option<String>,
    context: Vec<(String, String)>,
    hints: Vec<String>,
    related: Vec<Diagnostic>,
    source_code: Option<NamedSource<String>>,
    labels: Vec<LabeledSpan>,
}

impl Diagnostic {
    pub fn builder() -> DiagnosticBuilder {
        DiagnosticBuilder::default()
    }

    pub fn code(&self) -> DiagnosticCode {
        self.code
    }

    pub fn severity(&self) -> Severity {
        self.severity
    }

    pub fn phase(&self) -> Option<DiagnosticPhase> {
        self.phase
    }

    pub fn engine(&self) -> Option<&'static str> {
        self.engine
    }

    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn source(&self) -> Option<&str> {
        self.source.as_deref()
    }

    pub fn context(&self) -> &[(String, String)] {
        &self.context
    }

    pub fn hints(&self) -> &[String] {
        &self.hints
    }

    pub fn related(&self) -> &[Diagnostic] {
        &self.related
    }

    pub(crate) fn source_code(&self) -> Option<&NamedSource<String>> {
        self.source_code.as_ref()
    }

    pub(crate) fn labels(&self) -> &[LabeledSpan] {
        &self.labels
    }
}

impl Display for Diagnostic {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title, self.reason)
    }
}
