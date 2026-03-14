use std::sync::Arc;

use anyhow::Result;

use crate::domain::{ports::ITopicStore, types::Topic};

/// Coordinates topic read operations.
pub struct TopicManager {
    store: Arc<dyn ITopicStore>,
}

impl TopicManager {
    /// Create a `TopicManager` backed by `store`.
    #[must_use]
    pub fn new(store: Arc<dyn ITopicStore>) -> Self {
        Self { store }
    }

    /// List all topics.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_all(&self) -> Result<Vec<Topic>> {
        self.store.list().await
    }

    /// Look up a topic by name. Returns `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Topic>> {
        self.store.get_by_name(name).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports::ITopicStore,
        types::{Topic, TopicUpdate},
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    struct StubTopicStore {
        topics: Vec<Topic>,
    }

    #[async_trait]
    impl ITopicStore for StubTopicStore {
        async fn create(&self, _topic: Topic) -> Result<()> {
            unimplemented!()
        }
        async fn get(&self, id: Uuid) -> Result<Option<Topic>> {
            Ok(self.topics.iter().find(|t| t.id == id).cloned())
        }
        async fn get_by_name(&self, name: &str) -> Result<Option<Topic>> {
            Ok(self.topics.iter().find(|t| t.name == name).cloned())
        }
        async fn list(&self) -> Result<Vec<Topic>> {
            Ok(self.topics.clone())
        }
        async fn update(&self, _id: Uuid, _updates: TopicUpdate) -> Result<Topic> {
            unimplemented!()
        }
    }

    fn make_topic(name: &str) -> Topic {
        Topic {
            id: Uuid::new_v4(),
            name: name.to_string(),
            description: String::new(),
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
    async fn test_list_returns_all_topics() {
        let store = Arc::new(StubTopicStore {
            topics: vec![make_topic("alpha"), make_topic("beta")],
        });
        let mgr = TopicManager::new(store);
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_by_name_found() {
        let t = make_topic("found");
        let store = Arc::new(StubTopicStore { topics: vec![t.clone()] });
        let mgr = TopicManager::new(store);
        let result = mgr.get_by_name("found").await.unwrap();
        assert_eq!(result.unwrap().name, "found");
    }

    #[tokio::test]
    async fn test_get_by_name_not_found() {
        let store = Arc::new(StubTopicStore { topics: vec![] });
        let mgr = TopicManager::new(store);
        assert!(mgr.get_by_name("missing").await.unwrap().is_none());
    }
}
