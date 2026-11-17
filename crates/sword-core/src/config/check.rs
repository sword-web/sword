use thisconfig::{Config, ConfigError};

use crate::diagnostic::{Diagnostic, DiagnosticPhase, Error};

/// One problem found while checking a configuration section.
#[derive(Debug, Clone)]
pub struct ConfigProblem {
    pub key: Option<&'static str>,
    pub reason: String,
    pub hint: Option<String>,
}

impl ConfigProblem {
    pub fn new(key: &'static str, reason: impl Into<String>) -> Self {
        Self {
            key: Some(key),
            reason: reason.into(),
            hint: None,
        }
    }

    pub fn for_section(reason: impl Into<String>) -> Self {
        Self {
            key: None,
            reason: reason.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Semantic rules for one configuration section.
pub trait ConfigCheck {
    const SECTION: &'static str;
    const CODE: &'static str;

    fn check(&self) -> Result<(), Vec<ConfigProblem>>;
}

/// A failed configuration section check.
#[derive(Debug)]
pub struct ConfigFailure {
    pub section: &'static str,
    pub code: &'static str,
    pub problems: Vec<ConfigProblem>,
    pub cause: Option<Box<ConfigError>>,
}

impl ConfigFailure {
    pub fn into_error(self) -> Error {
        let reason = match self.problems.as_slice() {
            [problem] => problem.reason.clone(),
            problems => problems
                .iter()
                .map(|problem| format!("- {}", problem.reason))
                .collect::<Vec<_>>()
                .join("\n"),
        };

        let mut builder = Diagnostic::builder()
            .code(self.code)
            .phase(DiagnosticPhase::Build)
            .target(format!("[{}]", self.section))
            .title(format!("Invalid [{}] configuration", self.section))
            .reason(reason)
            .source("ConfigValidateExt::get_validated_section");

        for problem in &self.problems {
            if let Some(hint) = &problem.hint {
                builder = builder.hint(hint.clone());
            }

            if let Some(key) = problem.key {
                builder = builder.context("key", format!("{}.{}", self.section, key));
            }
        }

        let error = Error::new(builder.build());

        match self.cause {
            Some(cause) => error.with_source(*cause),
            None => error,
        }
    }
}

/// Builds a single diagnostic for every failed section.
pub fn config_failures_error(failures: Vec<ConfigFailure>, config: &Config) -> Error {
    if failures.len() == 1 {
        return failures
            .into_iter()
            .next()
            .expect("one failure")
            .into_error();
    }

    let mut reasons = Vec::new();
    let mut hints = Vec::new();

    for failure in &failures {
        for problem in &failure.problems {
            reasons.push(format!("[{}] {}", failure.section, problem.reason));

            if let Some(hint) = &problem.hint
                && !hints.contains(hint)
            {
                hints.push(hint.clone());
            }
        }
    }

    let target = config
        .primary_file_source()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "configuration".to_string());

    let mut builder = Diagnostic::builder()
        .code("sword.build.config")
        .phase(DiagnosticPhase::Build)
        .target(target)
        .title("Invalid configuration")
        .reason(reasons.join("\n"))
        .source("validate_all");

    for hint in hints {
        builder = builder.hint(hint);
    }

    Error::new(builder.build())
}
