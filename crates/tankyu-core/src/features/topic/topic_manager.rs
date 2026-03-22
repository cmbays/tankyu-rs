use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    ports::{IGraphStore, ITopicStore},
    types::{EdgeType, Topic},
};
use crate::shared::error::TankyuError;

/// Input for creating a new topic.
pub struct CreateTopicInput {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Coordinates topic operations.
pub struct TopicManager {
    store: Arc<dyn ITopicStore>,
    graph: Arc<dyn IGraphStore>,
}

impl TopicManager {
    /// Create a `TopicManager` backed by `store` and `graph`.
    #[must_use]
    pub fn new(store: Arc<dyn ITopicStore>, graph: Arc<dyn IGraphStore>) -> Self {
        Self { store, graph }
    }

    /// List all topics.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_all(&self) -> Result<Vec<Topic>> {
        self.store.list().await
    }

    /// Look up a topic by ID. Returns `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<Topic>> {
        self.store.get(id).await
    }

    /// Look up a topic by name. Returns `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Topic>> {
        self.store.get_by_name(name).await
    }

    /// List topics that monitor a given source (via `Monitors` edges in the graph).
    ///
    /// Symmetric with `SourceManager::list_by_topic`.
    ///
    /// # Errors
    /// Returns an error if the store or graph fails.
    pub async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Topic>> {
        let edges = self.graph.get_edges_by_node(source_id).await?;
        let topic_ids: Vec<Uuid> = edges
            .into_iter()
            .filter(|e| e.to_id == source_id && e.edge_type == EdgeType::Monitors)
            .map(|e| e.from_id)
            .collect();
        let mut topics = Vec::with_capacity(topic_ids.len());
        for id in topic_ids {
            match self.store.get(id).await? {
                Some(t) => topics.push(t),
                None => {
                    eprintln!("warning: orphaned edge references topic {id} which no longer exists")
                }
            }
        }
        Ok(topics)
    }

    /// Create a new topic. Errors with `TankyuError::Duplicate` if name already exists.
    ///
    /// # Errors
    /// Returns `TankyuError::Duplicate` if a topic with this name already exists.
    /// Returns an error if the store write fails.
    pub async fn create(&self, input: CreateTopicInput) -> Result<Topic> {
        if self.store.get_by_name(&input.name).await?.is_some() {
            return Err(TankyuError::Duplicate {
                kind: "topic".to_string(),
                name: input.name,
            }
            .into());
        }
        let now = Utc::now();
        let topic = Topic {
            id: Uuid::new_v4(),
            name: input.name,
            description: input.description,
            tags: input.tags,
            projects: vec![],
            routing: None,
            created_at: now,
            updated_at: now,
            last_scanned_at: None,
            scan_count: 0,
        };
        self.store.create(topic.clone()).await?;
        Ok(topic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports::{IGraphStore, ITopicStore},
        types::{Edge, EdgeType, GraphQuery, NodeType, Topic, TopicUpdate},
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
            Ok(()) // stub — real durability tested in store_compat
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

    struct StubGraphStore {
        edges: Vec<Edge>,
    }

    #[async_trait]
    impl IGraphStore for StubGraphStore {
        async fn add_edge(&self, _e: Edge) -> Result<()> {
            Ok(())
        }
        async fn remove_edge(&self, _id: Uuid) -> Result<()> {
            Ok(())
        }
        async fn get_edges_by_node(&self, node_id: Uuid) -> Result<Vec<Edge>> {
            Ok(self
                .edges
                .iter()
                .filter(|e| e.from_id == node_id || e.to_id == node_id)
                .cloned()
                .collect())
        }
        async fn get_neighbors(&self, _id: Uuid, _et: Option<EdgeType>) -> Result<Vec<Edge>> {
            unimplemented!()
        }
        async fn query(&self, _q: GraphQuery) -> Result<Vec<Edge>> {
            unimplemented!()
        }
        async fn list(&self) -> Result<Vec<Edge>> {
            Ok(self.edges.clone())
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

    fn make_mgr(topics: Vec<Topic>, edges: Vec<Edge>) -> TopicManager {
        let store = Arc::new(StubTopicStore { topics });
        let graph = Arc::new(StubGraphStore { edges });
        TopicManager::new(store, graph)
    }

    #[tokio::test]
    async fn test_list_returns_all_topics() {
        let mgr = make_mgr(vec![make_topic("alpha"), make_topic("beta")], vec![]);
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_get_by_id_found() {
        let t = make_topic("by-id");
        let id = t.id;
        let mgr = make_mgr(vec![t], vec![]);
        let result = mgr.get_by_id(id).await.unwrap();
        assert_eq!(result.unwrap().name, "by-id");
    }

    #[tokio::test]
    async fn test_get_by_id_not_found() {
        let mgr = make_mgr(vec![], vec![]);
        assert!(mgr.get_by_id(Uuid::new_v4()).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_get_by_name_found() {
        let t = make_topic("found");
        let mgr = make_mgr(vec![t], vec![]);
        let result = mgr.get_by_name("found").await.unwrap();
        assert_eq!(result.unwrap().name, "found");
    }

    #[tokio::test]
    async fn test_get_by_name_not_found() {
        let mgr = make_mgr(vec![], vec![]);
        assert!(mgr.get_by_name("missing").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_create_returns_topic_with_correct_fields() {
        let mgr = make_mgr(vec![], vec![]);
        let result = mgr
            .create(CreateTopicInput {
                name: "Rust Async".to_string(),
                description: "Async Rust patterns".to_string(),
                tags: vec!["rust".to_string(), "async".to_string()],
            })
            .await
            .unwrap();
        assert_eq!(result.name, "Rust Async");
        assert_eq!(result.description, "Async Rust patterns");
        assert_eq!(result.tags, vec!["rust", "async"]);
        assert_eq!(result.scan_count, 0);
        assert!(result.last_scanned_at.is_none());
        assert!(result.projects.is_empty());
    }

    #[tokio::test]
    async fn test_create_duplicate_name_returns_error() {
        let existing = make_topic("Duplicate");
        let mgr = make_mgr(vec![existing], vec![]);
        let err = mgr
            .create(CreateTopicInput {
                name: "Duplicate".to_string(),
                description: String::new(),
                tags: vec![],
            })
            .await
            .unwrap_err();
        let tankyu_err = err.downcast::<TankyuError>().unwrap();
        assert!(matches!(tankyu_err, TankyuError::Duplicate { .. }));
    }

    // ── list_by_source() tests ──────────────────────────────────────────

    #[tokio::test]
    async fn test_list_by_source_returns_monitoring_topics() {
        let topic = make_topic("rust");
        let topic_id = topic.id;
        let source_id = Uuid::new_v4();
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        let mgr = make_mgr(vec![topic], vec![edge]);
        let result = mgr.list_by_source(source_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "rust");
    }

    #[tokio::test]
    async fn test_list_by_source_ignores_non_monitors_edges() {
        let topic = make_topic("rust");
        let topic_id = topic.id;
        let source_id = Uuid::new_v4();
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::TaggedWith,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        let mgr = make_mgr(vec![topic], vec![edge]);
        let result = mgr.list_by_source(source_id).await.unwrap();
        assert!(
            result.is_empty(),
            "TaggedWith should not be treated as Monitors"
        );
    }

    #[tokio::test]
    async fn test_list_by_source_empty_when_no_edges() {
        let mgr = make_mgr(vec![], vec![]);
        let result = mgr.list_by_source(Uuid::new_v4()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_source_ignores_edges_to_different_source() {
        let topic = make_topic("rust");
        let topic_id = topic.id;
        let other_source_id = Uuid::new_v4();
        let target_source_id = Uuid::new_v4();
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: other_source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        let mgr = make_mgr(vec![topic], vec![edge]);
        let result = mgr.list_by_source(target_source_id).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_source_skips_orphaned_topic() {
        let source_id = Uuid::new_v4();
        let nonexistent_topic_id = Uuid::new_v4();
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: nonexistent_topic_id,
            from_type: NodeType::Topic,
            to_id: source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        // Store has no topics — the edge references a topic that doesn't exist
        let mgr = make_mgr(vec![], vec![edge]);
        let result = mgr.list_by_source(source_id).await.unwrap();
        assert!(
            result.is_empty(),
            "orphaned edge should be skipped, not cause a panic"
        );
    }
}
