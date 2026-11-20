use crate::{CacheError, CacheFutureResult, CacheResult, L2CacheBackend};
use redis::{AsyncCommands, aio::ConnectionManager};
use std::time::Duration;

/// Valkey cache backend
pub struct ValkeyBackend {
    conn: ConnectionManager,
}

unsafe impl Send for ValkeyBackend {}
#[cfg(any(feature = "tokio", feature = "smol"))]
unsafe impl Sync for ValkeyBackend {}

impl ValkeyBackend {
    /// Create a new VakleyBackend
    pub async fn new(url: &str) -> CacheResult<Self> {
        let client =
            redis::Client::open(url).map_err(|e| CacheError::BackendError(e.to_string()))?;
        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| CacheError::BackendError(e.to_string()))?;
        Ok(Self { conn })
    }
}

impl L2CacheBackend for ValkeyBackend {
    fn get<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, Option<String>> {
        let future = async move {
            let mut conn = self.conn.clone();
            conn.get(key)
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))
        };

        Box::pin(future)
    }

    fn set<'a>(&'a self, key: &'a str, value: String, ttl: Duration) -> CacheFutureResult<'a, ()> {
        let future = async move {
            let mut conn = self.conn.clone();
            conn.set_ex::<_, _, ()>(key, value, ttl.as_secs())
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))
        };

        Box::pin(future)
    }

    fn delete<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, ()> {
        let future = async move {
            let mut conn = self.conn.clone();
            conn.del::<_, ()>(key)
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))
        };

        Box::pin(future)
    }
}
