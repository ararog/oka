#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
use moka::future::Cache as MokaCache;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;
use std::{pin::Pin, time::Duration};
use thiserror::Error;

/// Backend module
pub mod backend;

/// Type alias for result
pub type CacheResult<T> = Result<T, CacheError>;
/// Type alias for boxed future result
pub type CacheFutureResult<'a, T> = Pin<Box<dyn Future<Output = CacheResult<T>> + Send + 'a>>;

#[cfg(any(feature = "tokio", feature = "smol"))]
/// Typed lias for Shared data
pub type Shared<T> = Arc<T>;

#[cfg(any(feature = "tokio", feature = "smol"))]
/// Trait bounds alias for L2 cache backend
pub trait SharedValue: Send + Sync {}

#[cfg(any(feature = "tokio", feature = "smol"))]
/// Trait bounds alias for L2 cache backend
pub trait SharedL2Backend: L2CacheBackend + Send + Sync {}

#[cfg(any(feature = "tokio", feature = "smol"))]
impl<T> SharedL2Backend for T where T: L2CacheBackend + Send + Sync {}

#[derive(Error, Debug)]
/// CacheError type and its variants
pub enum CacheError {
    #[error("L2 storage error: {0}")]
    /// Variant for backend errors
    BackendError(String),
    #[error("Serialization error: {0}")]
    /// Variant for serialization errors
    SerializationError(String),
}

/// L2 Baackend trait
pub trait L2CacheBackend {
    /// Get a entry
    fn get<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, Option<String>>;
    /// Set a entry
    fn set<'a>(&'a self, key: &'a str, value: String, ttl: Duration) -> CacheFutureResult<'a, ()>;
    /// Delete a entry
    fn delete<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, ()>;
}

/// OkaCache is our main type
pub struct OkaCache<V> {
    l1: MokaCache<String, V>,
    l2: Shared<dyn SharedL2Backend>,
    l2_ttl: Duration,
}

impl<V> OkaCache<V>
where
    V: Clone + SharedValue + Serialize + DeserializeOwned + 'static,
{
    /// Crate a enw OkaCache
    pub fn new(
        l2: Arc<dyn SharedL2Backend>,
        l1_max_capacity: u64,
        l1_ttl: Duration,
        l2_ttl: Duration,
    ) -> Self {
        let l1 = MokaCache::builder()
            .max_capacity(l1_max_capacity)
            .time_to_live(l1_ttl)
            .build();

        Self { l1, l2, l2_ttl }
    }

    /// Retrieve a value from the cache
    pub fn get<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, Option<V>> {
        let future = async move {
            if let Some(val) = self.l1.get(key).await {
                return Ok(Some(val));
            }

            if let Some(raw) = self.l2.get(key).await? {
                let val: V = serde_json::from_str(&raw)
                    .map_err(|e| CacheError::SerializationError(e.to_string()))?;
                // Backfill L1
                self.l1.insert(key.to_string(), val.clone()).await;
                return Ok(Some(val));
            }

            Ok(None)
        };

        Box::pin(future)
    }

    /// Set a value on cache
    pub fn set<'a>(&'a self, key: &'a str, value: V) -> CacheFutureResult<'a, ()> {
        let future = async move {
            let raw = serde_json::to_string(&value)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;
            self.l2.set(key, raw, self.l2_ttl).await?;
            self.l1.insert(key.to_string(), value).await;
            Ok(())
        };

        Box::pin(future)
    }

    /// Invalidate an entry
    pub fn invalidate<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, ()> {
        let future = async move {
            self.l1.invalidate(key).await;
            self.l2.delete(key).await?;
            Ok(())
        };

        Box::pin(future)
    }
}
