use crate::{CacheFutureResult, L2CacheBackend};
use std::{
    collections::HashMap,
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(Default)]
/// Mock cache backend
pub struct MockBackend {
    store: RwLock<HashMap<String, (String, Instant)>>,
}

unsafe impl Send for MockBackend {}
#[cfg(any(feature = "tokio", feature = "smol"))]
unsafe impl Sync for MockBackend {}

impl L2CacheBackend for MockBackend {
    fn get<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, Option<String>> {
        let future = async move {
            let read_guard = self
                .store
                .read()
                .await;
            if let Some((val, expires_at)) = read_guard.get(key) {
                if Instant::now() < *expires_at {
                    return Ok(Some(val.clone()));
                }
            }
            drop(read_guard);

            // Clean up expired entry
            self.store
                .write()
                .await
                .remove(key);
            Ok(None)
        };

        Box::pin(future)
    }

    fn set<'a>(&'a self, key: &'a str, value: String, ttl: Duration) -> CacheFutureResult<'a, ()> {
        let future = async move {
            let expires_at = Instant::now() + ttl;
            self.store
                .write()
                .await
                .insert(key.to_string(), (value, expires_at));
            Ok(())
        };

        Box::pin(future)
    }

    fn delete<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, ()> {
        let future = async move {
            self.store
                .write()
                .await
                .remove(key);
            Ok(())
        };

        Box::pin(future)
    }
}
