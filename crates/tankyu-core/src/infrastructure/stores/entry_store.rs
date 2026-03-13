use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    domain::{
        ports::IEntryStore,
        types::{Entry, EntryUpdate},
    },
    infrastructure::persistence::JsonStore,
    shared::error::TankyuError,
};

/// JSON-backed store for [`Entry`] records.
pub struct EntryStore {
    store: JsonStore<Entry>,
}

impl EntryStore {
    /// Create a new `EntryStore` rooted at `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            store: JsonStore::new(dir),
        }
    }
}

#[async_trait]
impl IEntryStore for EntryStore {
    async fn create(&self, entry: Entry) -> Result<()> {
        self.store
            .write(&entry.id.to_string(), &entry)
            .await
            .context("failed to write entry")
    }

    async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
        match self.store.read(&id.to_string()).await {
            Ok(e) => Ok(Some(e)),
            Err(TankyuError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_by_url(&self, url: &str) -> Result<Option<Entry>> {
        let all = self.store.read_all().await?;
        Ok(all.into_iter().find(|e| e.url == url))
    }

    async fn get_by_content_hash(&self, hash: &str) -> Result<Option<Entry>> {
        let all = self.store.read_all().await?;
        Ok(all
            .into_iter()
            .find(|e| e.content_hash.as_deref() == Some(hash)))
    }

    async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>> {
        let all = self.store.read_all().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.source_id == source_id)
            .collect())
    }

    async fn list(&self) -> Result<Vec<Entry>> {
        self.store
            .read_all()
            .await
            .context("failed to list entries")
    }

    async fn update(&self, id: Uuid, updates: EntryUpdate) -> Result<Entry> {
        let mut entry = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("entry {id} not found"))?;
        if let Some(state) = updates.state {
            entry.state = state;
        }
        if let Some(signal) = updates.signal {
            entry.signal = Some(signal);
        }
        if let Some(summary) = updates.summary {
            entry.summary = Some(summary);
        }
        self.store.write(&id.to_string(), &entry).await?;
        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::domain::types::{EntryState, EntryType, Signal};
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn make_entry() -> Entry {
        Entry {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            r#type: EntryType::Commit,
            title: "test commit".to_string(),
            url: "https://github.com/x/y/commit/abc".to_string(),
            summary: None,
            content_hash: Some("abc123".to_string()),
            state: EntryState::New,
            signal: None,
            scanned_at: Utc::now(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_and_list() {
        let dir = tempdir().unwrap();
        let store = EntryStore::new(dir.path().to_path_buf());
        store.create(make_entry()).await.unwrap();
        store.create(make_entry()).await.unwrap();
        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn list_by_source() {
        let dir = tempdir().unwrap();
        let store = EntryStore::new(dir.path().to_path_buf());
        let source_id = Uuid::new_v4();
        let mut e1 = make_entry();
        e1.source_id = source_id;
        store.create(e1).await.unwrap();
        store.create(make_entry()).await.unwrap();
        let results = store.list_by_source(source_id).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn get_by_content_hash_finds_correct_entry() {
        let dir = tempdir().unwrap();
        let store = EntryStore::new(dir.path().to_path_buf());
        let mut target = make_entry();
        target.content_hash = Some("abc123".to_string());
        let mut other = make_entry();
        other.content_hash = Some("def456".to_string());
        store.create(target.clone()).await.unwrap();
        store.create(other.clone()).await.unwrap();
        let found = store.get_by_content_hash("abc123").await.unwrap();
        assert_eq!(found.unwrap().id, target.id);
    }

    #[tokio::test]
    async fn update_signal() {
        let dir = tempdir().unwrap();
        let store = EntryStore::new(dir.path().to_path_buf());
        let entry = make_entry();
        store.create(entry.clone()).await.unwrap();
        let updated = store
            .update(
                entry.id,
                EntryUpdate {
                    signal: Some(Signal::High),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.signal, Some(Signal::High));
    }
}
