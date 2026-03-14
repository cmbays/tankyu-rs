use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::domain::{
    ports::{IGraphStore, ISourceStore},
    types::{EdgeType, Source, SourceRole},
};

/// Coordinates source read operations.
pub struct SourceManager {
    store: Arc<dyn ISourceStore>,
    graph: Arc<dyn IGraphStore>,
}

impl SourceManager {
    /// Create a `SourceManager` backed by `store` and `graph`.
    #[must_use]
    pub fn new(store: Arc<dyn ISourceStore>, graph: Arc<dyn IGraphStore>) -> Self {
        Self { store, graph }
    }

    /// List all sources.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_all(&self) -> Result<Vec<Source>> {
        self.store.list().await
    }

    /// List sources filtered by role.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_by_role(&self, role: SourceRole) -> Result<Vec<Source>> {
        let all = self.store.list().await?;
        Ok(all
            .into_iter()
            .filter(|s| s.role.as_ref() == Some(&role))
            .collect())
    }

    /// Find a source by its name (case-sensitive, first match).
    ///
    /// Returns `None` if no source with that name exists.
    /// Note: name uniqueness is not enforced; this returns the first match.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Source>> {
        let all = self.store.list().await?;
        Ok(all.into_iter().find(|s| s.name == name))
    }

    /// List sources monitored by a topic (via `Monitors` edges in the graph).
    ///
    /// # Errors
    /// Returns an error if the store or graph fails.
    pub async fn list_by_topic(&self, topic_id: Uuid) -> Result<Vec<Source>> {
        let edges = self.graph.get_edges_by_node(topic_id).await?;
        let source_ids: Vec<Uuid> = edges
            .into_iter()
            .filter(|e| e.from_id == topic_id && e.edge_type == EdgeType::Monitors)
            .map(|e| e.to_id)
            .collect();
        let mut sources = Vec::with_capacity(source_ids.len());
        for id in source_ids {
            if let Some(s) = self.store.get(id).await? {
                sources.push(s);
            }
        }
        Ok(sources)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports::{IGraphStore, ISourceStore},
        types::{
            Edge, EdgeType, GraphQuery, NodeType, Source, SourceRole, SourceState, SourceType,
            SourceUpdate,
        },
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    struct StubSourceStore {
        sources: Vec<Source>,
    }
    struct StubGraphStore {
        edges: Vec<Edge>,
    }

    #[async_trait]
    impl ISourceStore for StubSourceStore {
        async fn create(&self, _s: Source) -> Result<()> {
            unimplemented!()
        }
        async fn get(&self, id: Uuid) -> Result<Option<Source>> {
            Ok(self.sources.iter().find(|s| s.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Source>> {
            unimplemented!()
        }
        async fn list(&self) -> Result<Vec<Source>> {
            Ok(self.sources.clone())
        }
        async fn update(&self, _id: Uuid, _u: SourceUpdate) -> Result<Source> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl IGraphStore for StubGraphStore {
        async fn add_edge(&self, _e: Edge) -> Result<()> {
            unimplemented!()
        }
        async fn remove_edge(&self, _id: Uuid) -> Result<()> {
            unimplemented!()
        }
        async fn get_edges_by_node(&self, node_id: Uuid) -> Result<Vec<Edge>> {
            Ok(self
                .edges
                .iter()
                .filter(|e| e.from_id == node_id || e.to_id == node_id)
                .cloned()
                .collect())
        }
        async fn get_neighbors(
            &self,
            _id: Uuid,
            _et: Option<EdgeType>,
        ) -> Result<Vec<Edge>> {
            unimplemented!()
        }
        async fn query(&self, _opts: GraphQuery) -> Result<Vec<Edge>> {
            unimplemented!()
        }
        async fn list(&self) -> Result<Vec<Edge>> {
            Ok(self.edges.clone())
        }
    }

    fn make_source(id: Uuid, role: Option<SourceRole>) -> Source {
        Source {
            id,
            r#type: SourceType::GithubRepo,
            role,
            name: "test".to_string(),
            url: format!("https://example.com/{id}"),
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
    async fn test_list_all_returns_all_sources() {
        let store = Arc::new(StubSourceStore {
            sources: vec![
                make_source(Uuid::new_v4(), None),
                make_source(Uuid::new_v4(), Some(SourceRole::Starred)),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_role_filters_correctly() {
        let store = Arc::new(StubSourceStore {
            sources: vec![
                make_source(Uuid::new_v4(), Some(SourceRole::Starred)),
                make_source(Uuid::new_v4(), Some(SourceRole::RoleModel)),
                make_source(Uuid::new_v4(), None),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.list_by_role(SourceRole::Starred).await.unwrap();
        assert_eq!(result.len(), 1);
    }

    #[tokio::test]
    async fn test_list_by_topic_returns_monitored_sources() {
        let topic_id = Uuid::new_v4();
        let s1_id = Uuid::new_v4();
        let s2_id = Uuid::new_v4();
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: s1_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        let store = Arc::new(StubSourceStore {
            sources: vec![make_source(s1_id, None), make_source(s2_id, None)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![edge] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.list_by_topic(topic_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, s1_id);
    }

    #[tokio::test]
    async fn test_get_by_name_found() {
        let target_id = Uuid::new_v4();
        let mut target = make_source(target_id, None);
        target.name = "rust-lang/rust".to_string();
        let store = Arc::new(StubSourceStore {
            sources: vec![target, make_source(Uuid::new_v4(), None)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.get_by_name("rust-lang/rust").await.unwrap();
        assert_eq!(result.unwrap().id, target_id);
    }

    #[tokio::test]
    async fn test_get_by_name_not_found() {
        let store = Arc::new(StubSourceStore { sources: vec![] });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.get_by_name("nonexistent").await.unwrap();
        assert!(result.is_none());
    }
}
