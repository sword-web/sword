use super::Diagnostic;
use super::types::Severity;

use ::miette::{LabeledSpan, SourceCode};
use std::fmt::Display;

impl std::error::Error for Diagnostic {}

impl ::miette::Diagnostic for Diagnostic {
    /// The header miette puts above the message.
    ///
    /// It always carries the diagnostic code and a category, and appends the
    /// target only when there is no snippet to point at.
    fn code(&self) -> Option<Box<dyn Display + '_>> {
        let severity = self.severity();
        let category = match self.phase() {
            Some(phase) => format!("{} {}", phase.title(), severity.as_str()),
            None => severity.title().to_string(),
        };

        let mut header = format!("{} — {category}", self.code());

        if self.source_code().is_none()
            && self.labels().is_empty()
            && let Some(target) = self.target().or_else(|| self.source())
        {
            header.push_str(" — ");
            header.push_str(target);
        }

        Some(Box::new(header))
    }

    fn severity(&self) -> Option<::miette::Severity> {
        Some(match self.severity() {
            Severity::Error => ::miette::Severity::Error,
            Severity::Warning => ::miette::Severity::Warning,
            Severity::Advice => ::miette::Severity::Advice,
        })
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        self.source_code().map(|source| source as &dyn SourceCode)
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        if self.labels().is_empty() {
            return None;
        }

        Some(Box::new(self.labels().iter().cloned()))
    }

    /// Context and hints share the help block.
    ///
    /// Context is only shown when there is no snippet, because the snippet
    /// already carries the local detail and repeating it would be noise.
    fn help(&self) -> Option<Box<dyn Display + '_>> {
        let has_location = self.source_code().is_some() || !self.labels().is_empty();
        let mut lines = Vec::new();

        if !has_location {
            lines.extend(
                self.context()
                    .iter()
                    .map(|(key, value)| format!("{key}: {value}")),
            );
        }

        lines.extend(self.hints().iter().cloned());

        if lines.is_empty() {
            return None;
        }

        Some(Box::new(lines.join("\n")))
    }

    fn related(&self) -> Option<Box<dyn Iterator<Item = &dyn ::miette::Diagnostic> + '_>> {
        if self.related().is_empty() {
            return None;
        }

        Some(Box::new(
            self.related()
                .iter()
                .map(|related| related as &dyn ::miette::Diagnostic),
        ))
    }
}
