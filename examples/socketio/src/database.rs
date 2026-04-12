use crate::chat::Message;
use std::sync::Arc;
use sword::prelude::*;
use tokio::sync::RwLock;

#[injectable(provider)]
pub struct Database {
    pool: Arc<RwLock<Vec<Message>>>,
}

impl Database {
    pub fn new() -> Self {
        Database {
            pool: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn get(&self, key: &str) -> Option<Message> {
        let pool = self.pool.read().await;

        pool.iter().find(|m| m.id == key).cloned()
    }

    pub async fn get_all(&self) -> Vec<Message> {
        let pool = self.pool.read().await;

        pool.clone()
    }

    pub async fn set(&self, value: Message) {
        let mut pool = self.pool.write().await;
        pool.push(value);
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}
