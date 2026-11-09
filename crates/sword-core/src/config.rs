use serde::de::DeserializeOwned;
use thisconfig::ConfigError;

pub use thisconfig::{ByteConfig, Config, ConfigItem, ConfigSourceInfo, TimeConfig};

use crate::State;
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

/// Reads and checks one configuration section.
pub trait ConfigValidateExt {
    fn get_validated_section<T>(&self) -> Result<Option<T>, ConfigFailure>
    where
        T: ConfigCheck + ConfigItem + DeserializeOwned;
}

impl ConfigValidateExt for Config {
    fn get_validated_section<T>(&self) -> Result<Option<T>, ConfigFailure>
    where
        T: ConfigCheck + ConfigItem + DeserializeOwned,
    {
        match self.try_get::<T>() {
            Ok(None) => Ok(None),
            Ok(Some(value)) => match value.check() {
                Ok(()) => Ok(Some(value)),
                Err(problems) => Err(ConfigFailure {
                    section: T::SECTION,
                    code: T::CODE,
                    problems,
                    cause: None,
                }),
            },
            Err(cause) => Err(ConfigFailure {
                section: T::SECTION,
                code: T::CODE,
                problems: vec![
                    ConfigProblem::for_section(format!(
                        "failed to deserialize `[{}]`: {cause}",
                        T::SECTION
                    ))
                    .with_hint("Check the types of the values in this section"),
                ],
                cause: Some(Box::new(cause)),
            }),
        }
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
