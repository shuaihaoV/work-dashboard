use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::error::AppError;

#[derive(Debug, Clone)]
struct CacheEntry {
    expires_at: Instant,
    payload: Value,
}

#[derive(Debug, Default)]
pub struct ApiCache {
    inner: RwLock<HashMap<String, CacheEntry>>,
}

impl ApiCache {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let now = Instant::now();
        {
            let guard = self.inner.read().await;
            if let Some(entry) = guard.get(key) {
                if entry.expires_at > now {
                    return serde_json::from_value(entry.payload.clone()).ok();
                }
            }
        }

        let mut guard = self.inner.write().await;
        if let Some(entry) = guard.get(key) {
            if entry.expires_at <= now {
                guard.remove(key);
            }
        }
        None
    }

    pub async fn set<T: Serialize>(
        &self,
        key: String,
        ttl: Duration,
        value: &T,
    ) -> Result<(), AppError> {
        let payload = serde_json::to_value(value)?;
        let mut guard = self.inner.write().await;
        guard.insert(
            key,
            CacheEntry {
                expires_at: Instant::now() + ttl,
                payload,
            },
        );
        Ok(())
    }
}
