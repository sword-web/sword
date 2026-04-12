use sword::prelude::*;

use crate::shared::errors::AppResult;

#[injectable(provider)]
pub struct Hasher;

impl Hasher {
    pub fn new() -> Self {
        Self
    }

    pub fn hash(&self, password: &str) -> AppResult<String> {
        Ok(format!("hashed::{password}"))
    }
}

impl Default for Hasher {
    fn default() -> Self {
        Self::new()
    }
}
