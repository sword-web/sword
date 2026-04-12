use bcrypt::hash;
use serde::Deserialize;
use sword::prelude::*;

use crate::shared::errors::AppResult;

#[config(key = "hasher")]
#[derive(Clone, Deserialize)]
pub struct HasherConfig {
    pub cost: u32,
}

#[injectable(provider)]
pub struct Hasher {
    cost: u32,
}

impl Hasher {
    pub fn new(config: &HasherConfig) -> Self {
        Self { cost: config.cost }
    }

    pub fn hash(&self, password: &str) -> AppResult<String> {
        Ok(hash(password, self.cost)?)
    }
}

impl Default for HasherConfig {
    fn default() -> Self {
        Self { cost: 12 }
    }
}
