use std::fmt::{self, Display, Formatter};

use thisconfig::ConfigError;

#[derive(Debug)]
pub enum DependencyInjectionError {
    BuildFailed {
        type_name: String,
        source: Box<DependencyInjectionError>,
    },

    DependencyNotFound {
        type_name: String,
    },

    ConfigInjectionError {
        source: ConfigError,
    },

    CircularDependency,
}

impl DependencyInjectionError {
    pub fn build_failed(type_name: impl Into<String>, source: DependencyInjectionError) -> Self {
        Self::BuildFailed {
            type_name: type_name.into(),
            source: Box::new(source),
        }
    }

    pub fn dependency_not_found(type_name: impl Into<String>) -> Self {
        Self::DependencyNotFound {
            type_name: type_name.into(),
        }
    }

    pub fn diagnostic_context(&self) -> Vec<(String, String)> {
        let mut context = Vec::new();
        self.collect_diagnostic_context(&mut context);
        context
    }

    pub fn dependency_path(&self) -> Option<&str> {
        match self {
            Self::BuildFailed { type_name, .. } => Some(type_name.as_str()),
            Self::DependencyNotFound { .. }
            | Self::ConfigInjectionError { .. }
            | Self::CircularDependency => None,
        }
    }

    pub fn missing_dependency_path(&self) -> Option<&str> {
        match self {
            Self::BuildFailed { source, .. } => source.missing_dependency_path(),
            Self::DependencyNotFound { type_name } => Some(type_name.as_str()),
            Self::ConfigInjectionError { .. } | Self::CircularDependency => None,
        }
    }

    fn collect_diagnostic_context(&self, context: &mut Vec<(String, String)>) {
        match self {
            Self::BuildFailed { type_name, source } => {
                context.push(("dependency_path".to_string(), type_name.clone()));
                source.collect_diagnostic_context(context);
            }
            Self::DependencyNotFound { type_name } => {
                context.push(("missing_dependency_path".to_string(), type_name.clone()));
            }
            Self::ConfigInjectionError { source } => {
                context.push(("config_error".to_string(), source.to_string()));
            }
            Self::CircularDependency => {}
        }
    }
}

impl Display for DependencyInjectionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::BuildFailed { type_name, source } => write!(
                f,
                "Failed to build dependency '{}': {}",
                short_type_name(type_name),
                source
            ),
            Self::DependencyNotFound { type_name } => write!(
                f,
                "Dependency '{}' not found in dependency container",
                short_type_name(type_name)
            ),
            Self::ConfigInjectionError { source } => {
                write!(f, "Failed to inject config: {source}")
            }
            Self::CircularDependency => {
                write!(f, "Circular dependency detected in dependency container")
            }
        }
    }
}

impl std::error::Error for DependencyInjectionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BuildFailed { source, .. } => Some(source.as_ref()),
            Self::ConfigInjectionError { source } => Some(source),
            Self::DependencyNotFound { .. } | Self::CircularDependency => None,
        }
    }
}

impl From<ConfigError> for DependencyInjectionError {
    fn from(source: ConfigError) -> Self {
        Self::ConfigInjectionError { source }
    }
}

fn short_type_name(type_name: &str) -> &str {
    type_name.rsplit("::").next().unwrap_or(type_name)
}
