use std::{marker::PhantomData, path::PathBuf};

use serde::{de::DeserializeOwned, Serialize};
use tokio::io::AsyncWriteExt;

use crate::shared::error::TankyuError;

pub struct JsonStore<T> {
    dir: PathBuf,
    _marker: PhantomData<T>,
}

impl<T: Serialize + DeserializeOwned + Send + Sync> JsonStore<T> {
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            dir,
            _marker: PhantomData,
        }
    }

    fn path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.json"))
    }

    fn tmp_path(&self, id: &str) -> PathBuf {
        self.dir.join(format!("{id}.tmp"))
    }

    /// Read and deserialize `{dir}/{id}.json`.
    ///
    /// # Errors
    ///
    /// Returns `TankyuError::NotFound` if the file does not exist, or
    /// `TankyuError::Io` / `TankyuError::Json` on other failures.
    pub async fn read(&self, id: &str) -> Result<T, TankyuError> {
        let path = self.path(id);
        let bytes = tokio::fs::read(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TankyuError::NotFound(path.display().to_string())
            } else {
                TankyuError::from(e)
            }
        })?;
        let value: T = serde_json::from_slice(&bytes)?;
        Ok(value)
    }

    /// Serialize and write `{dir}/{id}.json` atomically via `.tmp` + rename.
    ///
    /// # Errors
    ///
    /// Returns `TankyuError::Io` on filesystem errors or `TankyuError::Json` on
    /// serialization failure.
    pub async fn write(&self, id: &str, value: &T) -> Result<(), TankyuError> {
        tokio::fs::create_dir_all(&self.dir).await?;
        let json = serde_json::to_string_pretty(value)?;
        let tmp = self.tmp_path(id);
        let mut file = tokio::fs::File::create(&tmp).await?;
        file.write_all(json.as_bytes()).await?;
        file.flush().await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&tmp, self.path(id)).await?;
        Ok(())
    }

    /// Delete `{dir}/{id}.json`.
    ///
    /// # Errors
    ///
    /// Returns `TankyuError::NotFound` if the file does not exist, or
    /// `TankyuError::Io` on other filesystem errors.
    pub async fn delete(&self, id: &str) -> Result<(), TankyuError> {
        let path = self.path(id);
        tokio::fs::remove_file(&path).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                TankyuError::NotFound(path.display().to_string())
            } else {
                TankyuError::from(e)
            }
        })
    }

    /// List all IDs (filenames without `.json` extension) in the store directory.
    /// Returns an empty vec if the directory does not exist yet.
    ///
    /// # Errors
    ///
    /// Returns `TankyuError::Io` on filesystem errors.
    pub async fn list_ids(&self) -> Result<Vec<String>, TankyuError> {
        match tokio::fs::read_dir(&self.dir).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(vec![]),
            Err(e) => Err(TankyuError::from(e)),
            Ok(mut rd) => {
                let mut ids = Vec::new();
                while let Some(entry) = rd.next_entry().await? {
                    let name = entry.file_name();
                    let s = name.to_string_lossy();
                    if let Some(id) = s.strip_suffix(".json") {
                        ids.push(id.to_string());
                    }
                }
                Ok(ids)
            }
        }
    }

    /// Read all values in the store directory.
    ///
    /// # Errors
    ///
    /// Returns `TankyuError::Io` or `TankyuError::Json` on failure.
    pub async fn read_all(&self) -> Result<Vec<T>, TankyuError> {
        let ids = self.list_ids().await?;
        let mut values = Vec::with_capacity(ids.len());
        for id in &ids {
            values.push(self.read(id).await?);
        }
        Ok(values)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::TankyuConfig;
    use std::collections::HashMap;
    use tempfile::tempdir;

    fn make_config() -> TankyuConfig {
        TankyuConfig {
            version: 1,
            default_scan_limit: 20,
            stale_days: 7,
            dormant_days: 30,
            llm_classify: false,
            local_repo_paths: HashMap::new(),
            registry_path: None,
        }
    }

    #[tokio::test]
    async fn write_then_read_roundtrips() {
        let dir = tempdir().unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(dir.path().to_path_buf());
        let config = make_config();
        store.write("config", &config).await.unwrap();
        let restored = store.read("config").await.unwrap();
        assert_eq!(config.version, restored.version);
        assert_eq!(config.stale_days, restored.stale_days);
    }

    #[tokio::test]
    async fn read_missing_returns_not_found() {
        let dir = tempdir().unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(dir.path().to_path_buf());
        let result = store.read("nonexistent").await;
        assert!(matches!(
            result,
            Err(crate::shared::error::TankyuError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn write_is_atomic() {
        let dir = tempdir().unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(dir.path().to_path_buf());
        let config = make_config();
        store.write("config", &config).await.unwrap();
        assert!(!dir.path().join("config.tmp").exists());
        assert!(dir.path().join("config.json").exists());
    }

    #[tokio::test]
    async fn list_ids_returns_all_written() {
        let dir = tempdir().unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(dir.path().to_path_buf());
        store.write("a", &make_config()).await.unwrap();
        store.write("b", &make_config()).await.unwrap();
        let mut ids = store.list_ids().await.unwrap();
        ids.sort();
        assert_eq!(ids, vec!["a".to_string(), "b".to_string()]);
    }

    #[tokio::test]
    async fn delete_existing_removes_file() {
        let dir = tempdir().unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(dir.path().to_path_buf());
        let config = make_config();
        store.write("config", &config).await.unwrap();
        assert!(dir.path().join("config.json").exists());

        store.delete("config").await.unwrap();

        assert!(!dir.path().join("config.json").exists());
    }

    #[tokio::test]
    async fn delete_nonexistent_returns_not_found() {
        let dir = tempdir().unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(dir.path().to_path_buf());
        let result = store.delete("nonexistent").await;
        assert!(matches!(
            result,
            Err(crate::shared::error::TankyuError::NotFound(_))
        ));
    }

    #[tokio::test]
    async fn list_ids_on_nonexistent_dir() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("does_not_exist");
        let store: JsonStore<TankyuConfig> = JsonStore::new(missing);
        let ids = store.list_ids().await.unwrap();
        assert!(ids.is_empty());
    }

    #[tokio::test]
    async fn list_ids_propagates_non_not_found_error() {
        let dir = tempdir().unwrap();
        // Create a regular file where the store expects a directory
        let file_path = dir.path().join("not_a_dir");
        std::fs::write(&file_path, b"I am a file").unwrap();
        let store: JsonStore<TankyuConfig> = JsonStore::new(file_path);
        let result = store.list_ids().await;
        assert!(result.is_err());
        // Must NOT be Ok(empty vec) — must propagate the error
        assert!(!matches!(
            result,
            Err(crate::shared::error::TankyuError::NotFound(_))
        ));
    }
}
