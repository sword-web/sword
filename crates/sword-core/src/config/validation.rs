use serde::de::DeserializeOwned;
use thisconfig::{Config, ConfigItem};

use super::check::{ConfigCheck, ConfigFailure, ConfigProblem};

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
