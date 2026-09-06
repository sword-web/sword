use crate::shared::errors::AppResult;

use argon2::{Algorithm, Argon2, ParamsBuilder, PasswordHasher, PasswordVerifier, Version};
use sword::prelude::*;
use tokio::task;

const DEFAULT_ARGON_MEMORY_KIB: u32 = 19 * 1024;

#[injectable(provider)]
pub struct Hasher {
    argon2: Argon2<'static>,
}

impl Hasher {
    pub fn new() -> Self {
        let params = ParamsBuilder::new()
            .m_cost(DEFAULT_ARGON_MEMORY_KIB)
            .build()
            .expect("Invalid Argon2 parameters in hasher config");

        Self {
            argon2: Argon2::new(Algorithm::Argon2id, Version::default(), params),
        }
    }

    pub async fn hash(&self, password: &str) -> AppResult<String> {
        let password = password.to_owned();
        let argon2 = self.argon2.clone();

        let password_result_result =
            task::spawn_blocking(move || argon2.hash_password(password.as_bytes())).await?;

        Ok(password_result_result.map(|hash| hash.to_string())?)
    }

    pub async fn verify(&self, password: &str, hash: &str) -> AppResult<bool> {
        let password = password.to_owned();
        let hash = hash.to_owned();
        let argon2 = self.argon2.clone();

        let result = tokio::task::spawn_blocking(move || {
            argon2.verify_password(password.as_bytes(), hash.as_str())
        })
        .await?;

        Ok(result.is_ok())
    }
}
