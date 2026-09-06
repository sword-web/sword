use std::{fmt::Display, str::FromStr};

/// The runtime environment of a Sword application.
///
/// Determines which default configuration file is loaded by
/// [`ApplicationBuilder`](crate::ApplicationBuilder) when `SWORD_ENV` is set.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Environment {
    #[default]
    Development,

    Production,
    Testing,
}

impl Environment {
    /// Reads the current environment from the `SWORD_ENV` variable.
    ///
    /// Returns `Ok(None)` when the variable is not set and `Err` when it is
    /// set to an invalid value.
    pub fn current() -> Result<Option<Self>, String> {
        let Ok(env) = std::env::var("SWORD_ENV") else {
            return Ok(None);
        };

        Environment::from_str(&env).map(Some)
    }

    /// The default configuration file path for this environment.
    pub fn default_config_path(&self) -> &'static str {
        match self {
            Self::Production => "config/config.prod.toml",
            Self::Development => "config/config.dev.toml",
            Self::Testing => "config/config.test.toml",
        }
    }
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "dev" | "development" => Ok(Self::Development),
            "prod" | "production" => Ok(Self::Production),
            "test" | "testing" => Ok(Self::Testing),
            other => Err(format!("invalid environment value: {other}")),
        }
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Production => write!(f, "prod"),
            Self::Development => write!(f, "dev"),
            Self::Testing => write!(f, "test"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_short_names() {
        assert_eq!(
            "dev".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!(
            "prod".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!("test".parse::<Environment>().unwrap(), Environment::Testing);
    }

    #[test]
    fn parses_long_names() {
        assert_eq!(
            "development".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!(
            "production".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!(
            "testing".parse::<Environment>().unwrap(),
            Environment::Testing
        );
    }

    #[test]
    fn parses_case_insensitively() {
        assert_eq!(
            "DEV".parse::<Environment>().unwrap(),
            Environment::Development
        );
        assert_eq!(
            "Prod".parse::<Environment>().unwrap(),
            Environment::Production
        );
        assert_eq!("TEST".parse::<Environment>().unwrap(), Environment::Testing);
    }

    #[test]
    fn rejects_invalid_values() {
        assert!("staging".parse::<Environment>().is_err());
        assert!("".parse::<Environment>().is_err());
        assert!("devlopment".parse::<Environment>().is_err());
    }

    #[test]
    fn default_config_paths_match_environment() {
        assert_eq!(
            Environment::Development.default_config_path(),
            "config/config.dev.toml"
        );
        assert_eq!(
            Environment::Production.default_config_path(),
            "config/config.prod.toml"
        );
        assert_eq!(
            Environment::Testing.default_config_path(),
            "config/config.test.toml"
        );
    }
}
