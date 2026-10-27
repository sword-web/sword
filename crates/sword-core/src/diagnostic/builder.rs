use ::miette::{LabeledSpan, NamedSource};
use std::ops::Range;

use super::Diagnostic;
use super::types::{DiagnosticCode, DiagnosticPhase, Severity};

/// Accumulates the fields of a [`Diagnostic`].
///
/// `code`, `title` and `reason` are required. They are checked in
/// [`DiagnosticBuilder::build`], which is only called from framework code where a
/// missing field would be a programming error.
#[derive(Debug, Default)]
pub struct DiagnosticBuilder {
    code: Option<DiagnosticCode>,
    severity: Option<Severity>,
    phase: Option<DiagnosticPhase>,
    engine: Option<&'static str>,
    target: Option<String>,
    title: Option<String>,
    reason: Option<String>,
    source: Option<String>,
    context: Vec<(String, String)>,
    hints: Vec<String>,
    related: Vec<Diagnostic>,
    source_code: Option<NamedSource<String>>,
    labels: Vec<LabeledSpan>,
}

impl DiagnosticBuilder {
    pub fn code(mut self, code: impl Into<DiagnosticCode>) -> Self {
        self.code = Some(code.into());
        self
    }

    pub fn severity(mut self, severity: Severity) -> Self {
        self.severity = Some(severity);
        self
    }

    pub fn phase(mut self, phase: DiagnosticPhase) -> Self {
        self.phase = Some(phase);
        self
    }

    pub fn engine(mut self, engine: &'static str) -> Self {
        self.engine = Some(engine);
        self
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    pub fn source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn source_code(mut self, name: impl AsRef<str>, text: impl Into<String>) -> Self {
        self.source_code = Some(NamedSource::new(name, text.into()));
        self
    }

    pub fn label(mut self, span: Range<usize>, message: impl Into<String>) -> Self {
        self.labels.push(LabeledSpan::new_with_span(
            Some(message.into()),
            (span.start, span.len()),
        ));
        self
    }

    /// Marks a span without label text.
    ///
    /// Use it when the message already explains the problem and the code frame
    /// only needs to show where it is.
    pub fn span(mut self, span: Range<usize>) -> Self {
        self.labels
            .push(LabeledSpan::new_with_span(None, (span.start, span.len())));
        self
    }

    /// Points the diagnostic at a source location captured by a macro.
    ///
    /// The file is read while the diagnostic is built. If it cannot be read, or
    /// the location is unknown, the path and coordinates are kept as context so
    /// the information is not lost.
    pub fn source_location(
        mut self,
        file: impl Into<String>,
        line: usize,
        column: usize,
        length: usize,
        message: impl Into<String>,
    ) -> Self {
        let file = file.into();

        if let Some((source_name, text)) = Self::read_source(&file)
            && let Some(offset) = Self::offset_for_location(&text, line, column)
        {
            self.source_code = Some(NamedSource::new(source_name, text));
            self.labels.push(LabeledSpan::new_with_span(
                Some(message.into()),
                (offset, length.max(1)),
            ));
        } else {
            self.context
                .push(("location".to_string(), format!("{file}:{line}:{column}")));
        }

        self
    }

    fn read_source(file: &str) -> Option<(String, String)> {
        if let Ok(text) = std::fs::read_to_string(file) {
            return Some((file.to_string(), text));
        }

        // Cargo runs binaries from the workspace root most of the time, but not
        // always. Walking up from the current directory makes relative `file!()`
        // paths work when a command is invoked from a subdirectory.
        let mut directory = std::env::current_dir().ok()?;

        loop {
            let candidate = directory.join(file);

            if let Ok(text) = std::fs::read_to_string(&candidate) {
                return Some((file.to_string(), text));
            }

            if !directory.pop() {
                return None;
            }
        }
    }

    /// Converts a one-based line and column into a byte offset.
    ///
    /// Columns are counted in characters, so the line is walked with
    /// `char_indices` instead of assuming ASCII. Line endings are preserved by
    /// splitting inclusively and dropping the trailing `\r\n` only for counting.
    fn offset_for_location(text: &str, line: usize, column: usize) -> Option<usize> {
        if line == 0 || column == 0 {
            return None;
        }

        let mut offset = 0;

        for (index, raw) in text.split_inclusive('\n').enumerate() {
            let content = raw
                .strip_suffix('\n')
                .unwrap_or(raw)
                .strip_suffix('\r')
                .unwrap_or(raw);

            if index + 1 == line {
                let byte_column = content
                    .char_indices()
                    .nth(column - 1)
                    .map(|(index, _)| index)
                    .unwrap_or(content.len());

                return Some(offset + byte_column.min(content.len()));
            }

            offset += raw.len();
        }

        None
    }

    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.push((key.into(), value.into()));
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn related(mut self, diagnostic: Diagnostic) -> Self {
        self.related.push(diagnostic);
        self
    }

    pub fn extend_related(mut self, diagnostics: impl IntoIterator<Item = Diagnostic>) -> Self {
        self.related.extend(diagnostics);
        self
    }

    pub fn build(self) -> Diagnostic {
        Diagnostic {
            code: self.code.expect("Diagnostic requires a `code`"),
            severity: self.severity.unwrap_or_default(),
            phase: self.phase,
            engine: self.engine,
            target: self.target,
            title: self.title.expect("Diagnostic requires a `title`"),
            reason: self.reason.expect("Diagnostic requires a `reason`"),
            source: self.source,
            context: self.context,
            hints: self.hints,
            related: self.related,
            source_code: self.source_code,
            labels: self.labels,
        }
    }
}
