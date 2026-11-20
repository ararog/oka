use crate::{CacheError, CacheFutureResult, CacheResult, L2CacheBackend};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;

#[derive(Serialize, Deserialize)]
struct DiskCacheEntry {
    expires_at_unix: u64,
    payload: String,
}

/// FileSystemBackend
pub struct FileSystemBackend {
    base_dir: PathBuf,
}

unsafe impl Send for FileSystemBackend {}
unsafe impl Sync for FileSystemBackend {}

impl FileSystemBackend {
    /// Create a new FileSystemBackend
    pub async fn new(base_dir: impl Into<PathBuf>) -> CacheResult<Self> {
        let base_dir = base_dir.into();
        fs::create_dir_all(&base_dir)
            .await
            .map_err(|e| CacheError::BackendError(format!("Failed to create cache dir: {e}")))?;
        Ok(Self { base_dir })
    }

    fn key_to_path(&self, key: &str) -> PathBuf {
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let hash = hex::encode(hasher.finalize());
        self.base_dir
            .join(format!("{hash}.json"))
    }

    fn now_unix() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }
}

impl L2CacheBackend for FileSystemBackend {
    fn get<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, Option<String>> {
        let future = async move {
            let path = self.key_to_path(key);

            let data = match fs::read_to_string(&path).await {
                Ok(content) => content,
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
                Err(e) => return Err(CacheError::BackendError(e.to_string())),
            };

            let entry: DiskCacheEntry = serde_json::from_str(&data)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            if Self::now_unix() >= entry.expires_at_unix {
                // Lazy cleanup of expired file
                let _ = fs::remove_file(&path).await;
                return Ok(None);
            }

            Ok(Some(entry.payload))
        };

        Box::pin(future)
    }

    fn set<'a>(&'a self, key: &'a str, value: String, ttl: Duration) -> CacheFutureResult<'a, ()> {
        let future = async move {
            let path = self.key_to_path(key);
            let temp_path = path.with_extension("tmp");

            let entry = DiskCacheEntry {
                expires_at_unix: Self::now_unix() + ttl.as_secs(),
                payload: value,
            };

            let serialized = serde_json::to_string(&entry)
                .map_err(|e| CacheError::SerializationError(e.to_string()))?;

            // Write to temp file first, then atomically rename to avoid partial reads
            fs::write(&temp_path, serialized)
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))?;

            fs::rename(&temp_path, &path)
                .await
                .map_err(|e| CacheError::BackendError(e.to_string()))?;

            Ok(())
        };

        Box::pin(future)
    }

    fn delete<'a>(&'a self, key: &'a str) -> CacheFutureResult<'a, ()> {
        let future = async move {
            let path = self.key_to_path(key);
            match fs::remove_file(path).await {
                Ok(()) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(CacheError::BackendError(e.to_string())),
            }
        };

        Box::pin(future)
    }
}
