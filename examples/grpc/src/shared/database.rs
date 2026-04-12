use std::sync::Arc;

use sword::prelude::*;
use tokio::sync::RwLock;

use crate::users::User;

#[injectable(provider)]
pub struct Database {
    pool: Arc<RwLock<Vec<User>>>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get(&self, id: &str) -> Option<User> {
        let pool = self.pool.read().await;
        pool.iter().find(|u| u.id == id).cloned()
    }

    pub async fn find_by_username(&self, username: &str) -> Option<User> {
        let pool = self.pool.read().await;
        pool.iter().find(|u| u.username == username).cloned()
    }

    pub async fn get_all(&self) -> Vec<User> {
        let pool = self.pool.read().await;
        pool.clone()
    }

    pub async fn set(&self, user: User) {
        let mut pool = self.pool.write().await;
        pool.push(user);
    }

    pub async fn upsert(&self, user: User) {
        let mut pool = self.pool.write().await;

        if let Some(existing) = pool.iter_mut().find(|u| u.id == user.id) {
            *existing = user;
            return;
        }

        pool.push(user);
    }

    pub async fn delete(&self, id: &str) -> bool {
        let mut pool = self.pool.write().await;
        let prev_len = pool.len();
        pool.retain(|u| u.id != id);
        prev_len != pool.len()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
