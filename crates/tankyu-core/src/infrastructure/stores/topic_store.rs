use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::PathBuf;
use uuid::Uuid;

use crate::{
    domain::{
        ports::ITopicStore,
        types::{Topic, TopicUpdate},
    },
    infrastructure::persistence::JsonStore,
    shared::error::TankyuError,
};

/// JSON-backed store for [`Topic`] records.
pub struct TopicStore {
    store: JsonStore<Topic>,
}

impl TopicStore {
    /// Create a new `TopicStore` rooted at `dir`.
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self {
            store: JsonStore::new(dir),
        }
    }
}

#[async_trait]
impl ITopicStore for TopicStore {
    async fn create(&self, topic: Topic) -> Result<()> {
        self.store
            .write(&topic.id.to_string(), &topic)
            .await
            .context("failed to write topic")
    }

    async fn get(&self, id: Uuid) -> Result<Option<Topic>> {
        match self.store.read(&id.to_string()).await {
            Ok(t) => Ok(Some(t)),
            Err(TankyuError::NotFound(_)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Topic>> {
        let all = self.store.read_all().await?;
        Ok(all.into_iter().find(|t| t.name == name))
    }

    async fn list(&self) -> Result<Vec<Topic>> {
        self.store.read_all().await.context("failed to list topics")
    }

    async fn update(&self, id: Uuid, updates: TopicUpdate) -> Result<Topic> {
        let mut topic = self
            .get(id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("topic {id} not found"))?;
        if let Some(name) = updates.name {
            topic.name = name;
        }
        if let Some(desc) = updates.description {
            topic.description = desc;
        }
        if let Some(tags) = updates.tags {
            topic.tags = tags;
        }
        if let Some(routing) = updates.routing {
            topic.routing = Some(routing);
        }
        if let Some(at) = updates.updated_at {
            topic.updated_at = at;
        }
        if let Some(at) = updates.last_scanned_at {
            topic.last_scanned_at = Some(at);
        }
        if let Some(count) = updates.scan_count {
            topic.scan_count = count;
        }
        self.store.write(&id.to_string(), &topic).await?;
        Ok(topic)
    }
}

#[cfg(test)]
mod tests {
    #[allow(clippy::wildcard_imports)]
    use super::*;
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn make_topic(name: &str) -> Topic {
        Topic {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: "test".to_string(),
            tags: vec![],
            projects: vec![],
            routing: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_scanned_at: None,
            scan_count: 0,
        }
    }

    #[tokio::test]
    async fn create_then_get_by_id() {
        let dir = tempdir().unwrap();
        let store = TopicStore::new(dir.path().to_path_buf());
        let topic = make_topic("rust");
        store.create(topic.clone()).await.unwrap();
        let got = store.get(topic.id).await.unwrap();
        assert_eq!(got.unwrap().name, "rust");
    }

    #[tokio::test]
    async fn get_by_name_finds_correct_topic() {
        let dir = tempdir().unwrap();
        let store = TopicStore::new(dir.path().to_path_buf());
        let target = make_topic("nanograph");
        let other = make_topic("rust");
        store.create(target.clone()).await.unwrap();
        store.create(other.clone()).await.unwrap();
        let found = store.get_by_name("nanograph").await.unwrap();
        assert_eq!(found.unwrap().id, target.id);
    }

    #[tokio::test]
    async fn list_returns_all() {
        let dir = tempdir().unwrap();
        let store = TopicStore::new(dir.path().to_path_buf());
        store.create(make_topic("a")).await.unwrap();
        store.create(make_topic("b")).await.unwrap();
        let list = store.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn update_changes_name() {
        let dir = tempdir().unwrap();
        let store = TopicStore::new(dir.path().to_path_buf());
        let topic = make_topic("old-name");
        store.create(topic.clone()).await.unwrap();
        let updated = store
            .update(
                topic.id,
                TopicUpdate {
                    name: Some("new-name".to_string()),
                    ..Default::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(updated.name, "new-name");
    }
}
