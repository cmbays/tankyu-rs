use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    domain::{
        ports::ISourceStore,
        types::{Source, SourceUpdate},
    },
    infrastructure::persistence::JsonStore,
    shared::error::TankyuError,
};

/// JSON-backed store for [`Source`] records.
pub struct SourceStore {
    store: JsonStore<Source>,
}

impl SourceStore {
    /// Create a new `SourceStore` rooted at `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            store: JsonStore::new(dir),
        }
    }
}

#[async_trait]
impl ISourceStore for SourceStore {
    async fn create(&self, source: Source) -> Result<()> {
        self.store
            .write(&source.id.to_string(), &source)
            .await
            .context("failed to write source")
    }

    async fn get(&self, id: Uuid) -> Result<Option<Source>> {
        match self.store.read(&id.to_string()).await {
            Ok(s) => Ok(Some(s)),
            Err(TankyuError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_by_url(&self, url: &str) -> Result<Option<Source>> {
        let all = self.store.read_all().await?;
        Ok(all.into_iter().find(|s| s.url == url))
    }

    async fn list(&self) -> Result<Vec<Source>> {
        self.store
            .read_all()
            .await
            .context("failed to list sources")
    }

    async fn update(&self, id: Uuid, updates: SourceUpdate) -> Result<Source> {
        let mut source = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("source {id} not found"))?;
        if let Some(role) = updates.role {
            source.role = Some(role);
        }
        if let Some(state) = updates.state {
            source.state = state;
        }
        if let Some(m) = updates.poll_interval_minutes {
            source.poll_interval_minutes = Some(m);
        }
        if let Some(at) = updates.last_checked_at {
            source.last_checked_at = Some(at);
        }
        if let Some(at) = updates.last_new_content_at {
            source.last_new_content_at = Some(at);
        }
        if let Some(c) = updates.check_count {
            source.check_count = c;
        }
        if let Some(h) = updates.hit_count {
            source.hit_count = h;
        }
        if let Some(m) = updates.miss_count {
            source.miss_count = m;
        }
        self.store.write(&id.to_string(), &source).await?;
        Ok(source)
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::domain::types::{SourceRole, SourceState, SourceType};
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn make_source(url: &str) -> Source {
        Source {
            id: Uuid::new_v4(),
            r#type: SourceType::GithubRepo,
            role: None,
            name: "test-source".to_string(),
            url: url.to_string(),
            config: None,
            state: SourceState::Active,
            poll_interval_minutes: None,
            discovered_via: None,
            discovery_reason: None,
            last_checked_at: None,
            last_new_content_at: None,
            check_count: 0,
            hit_count: 0,
            miss_count: 0,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn create_then_get_by_url() {
        let dir = tempdir().unwrap();
        let store = SourceStore::new(dir.path().to_path_buf());
        let source = make_source("https://github.com/cmbays/tankyu");
        store.create(source.clone()).await.unwrap();
        let found = store
            .get_by_url("https://github.com/cmbays/tankyu")
            .await
            .unwrap();
        assert_eq!(found.unwrap().id, source.id);
    }

    #[tokio::test]
    async fn update_role() {
        let dir = tempdir().unwrap();
        let store = SourceStore::new(dir.path().to_path_buf());
        let source = make_source("https://example.com");
        store.create(source.clone()).await.unwrap();
        let updated = store
            .update(
                source.id,
                SourceUpdate {
                    role: Some(SourceRole::Starred),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.role, Some(SourceRole::Starred));
    }
}
