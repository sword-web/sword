use crate::State;
use crate::diagnostic::{Diagnostic, DiagnosticPhase, Error};
use serde::de::DeserializeOwned;
use std::ops::Range;
use std::path::PathBuf;
use thisconfig::ConfigError;
use toml_edit::{Document, Item};

pub use thisconfig::{ByteConfig, Config, ConfigItem, ConfigSourceInfo, TimeConfig};

/// One problem found while validating a configuration section.
///
/// A section can have several issues. They are collected into a single
/// diagnostic with one label per issue, so the user sees everything that is
/// wrong in that section at once.
#[derive(Debug, Clone)]
pub struct ConfigIssue {
    section: Option<&'static str>,
    key: Option<&'static str>,
    reason: String,
    label: Option<String>,
    hint: Option<String>,
}

impl ConfigIssue {
    /// Creates an issue attached to a specific key.
    ///
    /// The reason is what the report shows as the problem; the span marks the
    /// key. A label is only added when it says something the reason does not.
    pub fn new(key: &'static str, reason: impl Into<String>) -> Self {
        Self {
            section: None,
            key: Some(key),
            reason: reason.into(),
            label: None,
            hint: None,
        }
    }

    /// Creates an issue that belongs to the whole section.
    pub fn for_section(reason: impl Into<String>) -> Self {
        Self {
            section: None,
            key: None,
            reason: reason.into(),
            label: None,
            hint: None,
        }
    }

    /// Adds text under the arrow. Keep it short and non-redundant.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Points the issue at a nested section, such as `events.memory`.
    pub fn with_section(mut self, section: &'static str) -> Self {
        self.section = Some(section);
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn section(&self) -> Option<&'static str> {
        self.section
    }

    pub fn key(&self) -> Option<&'static str> {
        self.key
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub fn hint(&self) -> Option<&str> {
        self.hint.as_deref()
    }
}

/// The issues found in one configuration section.
#[derive(Debug, Clone, Default)]
pub struct ConfigErrors {
    issues: Vec<ConfigIssue>,
}

impl ConfigErrors {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, issue: ConfigIssue) {
        self.issues.push(issue);
    }

    pub fn is_empty(&self) -> bool {
        self.issues.is_empty()
    }

    pub fn into_issues(self) -> Vec<ConfigIssue> {
        self.issues
    }
}

/// Hints that appear more than once are only shown once.
fn unique_hints(issues: &[ConfigIssue]) -> Vec<&str> {
    issues
        .iter()
        .filter_map(ConfigIssue::hint)
        .fold(Vec::new(), |mut hints, hint| {
            if !hints.contains(&hint) {
                hints.push(hint);
            }

            hints
        })
}

/// One issue stays on the message line; several become a list.
fn issues_reason(issues: &[ConfigIssue]) -> String {
    match issues {
        [issue] => issue.reason().to_string(),
        _ => issues
            .iter()
            .map(|issue| format!("- {}", issue.reason()))
            .collect::<Vec<_>>()
            .join("\n"),
    }
}

/// Semantic rules for a configuration section.
///
/// `SECTION` is the TOML section the rules apply to and `CODE` is the
/// diagnostic code reported when validation fails. Keeping both on the trait
/// lets each crate own its section without the facade knowing about it.
pub trait ValidateConfig {
    const SECTION: &'static str;
    const CODE: &'static str;

    fn validate(&self) -> ConfigErrors;
}

/// The parsed TOML file, kept around to place labels.
///
/// Values are always read from [`Config`], because that is the merged and
/// interpolated result the application actually uses. This document only
/// answers "where is this key written?", never "what does it contain?".
#[derive(Debug, Clone)]
pub struct ConfigDocument {
    path: PathBuf,
    document: Document<String>,
}

impl ConfigDocument {
    /// Loads the primary source file, if there is one and it parses.
    ///
    /// A missing or unparsable document is not an error: diagnostics simply
    /// render without a snippet.
    pub fn load(config: &Config) -> Option<Self> {
        let path = config.primary_file_source()?.to_path_buf();
        let text = std::fs::read_to_string(&path).ok()?;
        let document = Document::parse(text).ok()?;

        Some(Self { path, document })
    }

    pub fn contains_section(&self, section: &str) -> bool {
        self.item(section).is_some()
    }

    /// Returns the span of the last key in the section path.
    pub fn section_span(&self, section: &str) -> Option<Range<usize>> {
        let mut parts = section.split('.');
        let first = parts.next()?;

        let mut key = self.document.as_table().get_key_value(first)?.0;
        let mut item = self.document.as_table().get(first)?;

        for part in parts {
            let (next_key, next_item) = item.as_table()?.get_key_value(part)?;

            key = next_key;
            item = next_item;
        }

        key.span()
    }

    /// Returns the span of the value assigned to `key`.
    pub fn value_span(&self, section: &str, key: &str) -> Option<Range<usize>> {
        let item = self.item(section)?;

        if let Some(table) = item.as_table() {
            return table.get_key_value(key)?.1.span();
        }

        // `[section]` and `section = { ... }` are both valid TOML. Inline
        // tables need their own lookup because they are not `Table`s.
        item.as_inline_table()?.get_key_value(key)?.1.span()
    }

    fn item(&self, section: &str) -> Option<&Item> {
        let mut parts = section.split('.');
        let mut item = self.document.as_table().get(parts.next()?)?;

        for part in parts {
            item = item.get(part)?;
        }

        Some(item)
    }

    /// Builds a diagnostic for a failed section, adding one label per issue.
    fn diagnostic(
        &self,
        section: &'static str,
        code: &'static str,
        title: impl Into<String>,
        errors: ConfigErrors,
    ) -> Error {
        let issues = errors.into_issues();
        let reason = issues_reason(&issues);

        let mut builder = Diagnostic::builder()
            .code(code)
            .phase(DiagnosticPhase::Build)
            .target(self.path.display().to_string())
            .title(title)
            .reason(reason)
            .source("ConfigDocument::diagnostic");

        let labels = issues
            .iter()
            .filter_map(|issue| {
                let issue_section = issue.section().unwrap_or(section);
                let span = match issue.key() {
                    Some(key) => self.value_span(issue_section, key),
                    None => self.section_span(issue_section),
                };

                span.map(|span| (span, issue.label().map(ToOwned::to_owned)))
            })
            .collect::<Vec<_>>();

        if !labels.is_empty() {
            builder = builder.source_code(self.path.display().to_string(), self.document.raw());

            for (span, label) in labels {
                builder = match label {
                    Some(label) => builder.label(span, label),
                    None => builder.span(span),
                };
            }
        }

        for hint in unique_hints(&issues) {
            builder = builder.hint(hint);
        }

        Error::new(builder.build())
    }
}

/// Loads and validates one configuration section.
pub struct ConfigSection;

impl ConfigSection {
    /// Returns `None` when the section is absent.
    ///
    /// Absent sections keep using their default values and are not validated:
    /// there is no user-written configuration to complain about. Present
    /// sections are checked for deserialization errors and then for semantic
    /// rules.
    pub fn load<T>(config: &Config, document: Option<&ConfigDocument>) -> Result<Option<T>, Error>
    where
        T: ValidateConfig + ConfigItem + DeserializeOwned,
    {
        match config.try_get::<T>() {
            Ok(None) => Ok(None),
            Ok(Some(value)) => {
                let errors = value.validate();

                if errors.is_empty() {
                    Ok(Some(value))
                } else {
                    Err(section_error(document, T::SECTION, T::CODE, errors, None))
                }
            }
            Err(error) => {
                let reason = error
                    .to_string()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ");

                let mut errors = ConfigErrors::new();
                errors.push(
                    ConfigIssue::for_section(format!(
                        "failed to deserialize `[{}]`: {reason}",
                        T::SECTION
                    ))
                    .with_hint("Check the types of the values in this section"),
                );

                Err(section_error(
                    document,
                    T::SECTION,
                    T::CODE,
                    errors,
                    Some(error),
                ))
            }
        }
    }
}

/// Builds the diagnostic for a section, with a snippet when the TOML file is
/// available and a plain target otherwise.
fn section_error(
    document: Option<&ConfigDocument>,
    section: &'static str,
    code: &'static str,
    errors: ConfigErrors,
    cause: Option<ConfigError>,
) -> Error {
    let title = format!("Invalid [{section}] configuration");

    let error = match document {
        Some(document) => document.diagnostic(section, code, title, errors),
        None => {
            let issues = errors.into_issues();
            let reason = issues_reason(&issues);

            let mut builder = Diagnostic::builder()
                .code(code)
                .phase(DiagnosticPhase::Build)
                .target(format!("[{section}]"))
                .title(title)
                .reason(reason)
                .source("ConfigSection::load");

            for hint in unique_hints(&issues) {
                builder = builder.hint(hint);
            }

            Error::new(builder.build())
        }
    };

    match cause {
        Some(cause) => error.with_source(cause),
        None => error,
    }
}

/// A struct that holds a function to register a config type.
/// Used by the inventory system to collect all config types at compile time.
pub struct ConfigRegistrar {
    pub register: fn(&State, &Config) -> Result<(), Error>,
}

impl ConfigRegistrar {
    pub const fn new(register: fn(&State, &Config) -> Result<(), Error>) -> Self {
        Self { register }
    }
}

inventory::collect!(ConfigRegistrar);
