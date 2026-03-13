use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    domain::{
        ports::IInsightStore,
        types::{Insight, InsightUpdate},
    },
    infrastructure::persistence::JsonStore,
    shared::error::TankyuError,
};

/// JSON-backed store for [`Insight`] records.
pub struct InsightStore {
    store: JsonStore<Insight>,
}

impl InsightStore {
    /// Create a new `InsightStore` rooted at `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            store: JsonStore::new(dir),
        }
    }
}

#[async_trait]
impl IInsightStore for InsightStore {
    async fn create(&self, insight: Insight) -> Result<()> {
        self.store
            .write(&insight.id.to_string(), &insight)
            .await
            .context("failed to write insight")
    }

    async fn get(&self, id: Uuid) -> Result<Option<Insight>> {
        match self.store.read(&id.to_string()).await {
            Ok(i) => Ok(Some(i)),
            Err(TankyuError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn list(&self) -> Result<Vec<Insight>> {
        self.store
            .read_all()
            .await
            .context("failed to list insights")
    }

    async fn update(&self, id: Uuid, updates: InsightUpdate) -> Result<Insight> {
        let mut insight = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("insight {id} not found"))?;
        if let Some(title) = updates.title {
            insight.title = title;
        }
        if let Some(body) = updates.body {
            insight.body = body;
        }
        if let Some(kp) = updates.key_points {
            insight.key_points = kp;
        }
        if let Some(cit) = updates.citations {
            insight.citations = cit;
        }
        if let Some(at) = updates.updated_at {
            insight.updated_at = at;
        }
        self.store.write(&id.to_string(), &insight).await?;
        Ok(insight)
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use crate::domain::types::InsightType;
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn make_insight() -> Insight {
        Insight {
            id: Uuid::new_v4(),
            r#type: InsightType::ResearchNote,
            title: "test insight".to_string(),
            body: "body text".to_string(),
            key_points: vec!["point 1".to_string()],
            citations: vec![],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            metadata: None,
        }
    }

    #[tokio::test]
    async fn create_then_get() {
        let dir = tempdir().unwrap();
        let store = InsightStore::new(dir.path().to_path_buf());
        let insight = make_insight();
        store.create(insight.clone()).await.unwrap();
        let got = store.get(insight.id).await.unwrap();
        assert_eq!(got.unwrap().title, "test insight");
    }

    #[tokio::test]
    async fn list_returns_all() {
        let dir = tempdir().unwrap();
        let store = InsightStore::new(dir.path().to_path_buf());
        store.create(make_insight()).await.unwrap();
        store.create(make_insight()).await.unwrap();
        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }
}
