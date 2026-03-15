use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::domain::{
    ports::{IGraphStore, ISourceStore},
    types::{Edge, EdgeType, NodeType, Source, SourceRole, SourceState, SourceType, SourceUpdate},
};
use crate::features::source::url_detect::{detect_source_type, name_from_url};

/// Input for adding a new source.
pub struct AddSourceInput {
    pub url: String,
    /// Override the auto-detected name.
    pub name: Option<String>,
    /// Override the auto-detected source type.
    pub source_type: Option<SourceType>,
    pub role: Option<SourceRole>,
    /// If provided, a `Monitors` edge is created from this topic to the source.
    pub topic_id: Option<Uuid>,
}

/// Coordinates source operations.
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

    /// Add a source. Idempotent by URL: returns existing source if URL already known.
    /// If `input.role` differs from existing, updates the role.
    /// If `input.topic_id` provided, creates a `Monitors` edge (deduplicated).
    ///
    /// # Errors
    /// Returns an error if the store or graph fails.
    pub async fn add(&self, input: AddSourceInput) -> Result<Source> {
        // Idempotency: return existing source if URL already tracked
        if let Some(existing) = self.store.get_by_url(&input.url).await? {
            // Update role if provided and different
            let source = if let Some(role) = &input.role {
                if existing.role.as_ref() == Some(role) {
                    existing
                } else {
                    self.store
                        .update(
                            existing.id,
                            SourceUpdate {
                                role: Some(role.clone()),
                                ..Default::default()
                            },
                        )
                        .await?
                }
            } else {
                existing
            };
            // Add monitors edge if topic provided
            if let Some(topic_id) = input.topic_id {
                self.ensure_monitors_edge(source.id, topic_id).await?;
            }
            return Ok(source);
        }

        // New source
        let source_type = input
            .source_type
            .unwrap_or_else(|| detect_source_type(&input.url));
        let name = input.name.unwrap_or_else(|| name_from_url(&input.url));
        let source = Source {
            id: Uuid::new_v4(),
            r#type: source_type,
            role: input.role,
            name,
            url: input.url,
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
        };
        self.store.create(source.clone()).await?;

        if let Some(topic_id) = input.topic_id {
            self.ensure_monitors_edge(source.id, topic_id).await?;
        }
        Ok(source)
    }

    /// Create a `Monitors` edge from `topic_id` → `source_id` unless one already exists.
    async fn ensure_monitors_edge(&self, source_id: Uuid, topic_id: Uuid) -> Result<()> {
        // Query by source_id: get_edges_by_node returns all edges where from_id or to_id
        // matches, so this finds any existing Monitors edge targeting this source.
        let edges = self.graph.get_edges_by_node(source_id).await?;
        let already_exists = edges.iter().any(|e| {
            e.from_id == topic_id && e.to_id == source_id && e.edge_type == EdgeType::Monitors
        });
        if already_exists {
            return Ok(());
        }
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "Topic monitors source".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        self.graph.add_edge(edge).await
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
    use std::sync::{Arc, Mutex};
    use uuid::Uuid;

    /// A stub source store that actually persists created sources in memory.
    struct StubSourceStore {
        sources: Mutex<Vec<Source>>,
    }

    impl StubSourceStore {
        fn with_sources(sources: Vec<Source>) -> Self {
            Self {
                sources: Mutex::new(sources),
            }
        }
        fn empty() -> Self {
            Self {
                sources: Mutex::new(vec![]),
            }
        }
    }

    struct StubGraphStore {
        edges: Vec<Edge>,
    }

    #[async_trait]
    impl ISourceStore for StubSourceStore {
        async fn create(&self, s: Source) -> Result<()> {
            self.sources.lock().unwrap().push(s);
            Ok(())
        }
        async fn get(&self, id: Uuid) -> Result<Option<Source>> {
            Ok(self
                .sources
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.id == id)
                .cloned())
        }
        async fn get_by_url(&self, url: &str) -> Result<Option<Source>> {
            Ok(self
                .sources
                .lock()
                .unwrap()
                .iter()
                .find(|s| s.url == url)
                .cloned())
        }
        async fn list(&self) -> Result<Vec<Source>> {
            Ok(self.sources.lock().unwrap().clone())
        }
        async fn update(&self, id: Uuid, u: SourceUpdate) -> Result<Source> {
            let mut sources = self.sources.lock().unwrap();
            let source = sources
                .iter_mut()
                .find(|s| s.id == id)
                .expect("source not found in stub");
            if let Some(role) = u.role {
                source.role = Some(role);
            }
            Ok(source.clone())
        }
    }

    #[async_trait]
    impl IGraphStore for StubGraphStore {
        async fn add_edge(&self, _e: Edge) -> Result<()> {
            Ok(())
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
        async fn get_neighbors(&self, _id: Uuid, _et: Option<EdgeType>) -> Result<Vec<Edge>> {
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
        let store = Arc::new(StubSourceStore::with_sources(vec![
            make_source(Uuid::new_v4(), None),
            make_source(Uuid::new_v4(), Some(SourceRole::Starred)),
        ]));
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_role_filters_correctly() {
        let store = Arc::new(StubSourceStore::with_sources(vec![
            make_source(Uuid::new_v4(), Some(SourceRole::Starred)),
            make_source(Uuid::new_v4(), Some(SourceRole::RoleModel)),
            make_source(Uuid::new_v4(), None),
        ]));
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
        let store = Arc::new(StubSourceStore::with_sources(vec![
            make_source(s1_id, None),
            make_source(s2_id, None),
        ]));
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
        let store = Arc::new(StubSourceStore::with_sources(vec![
            target,
            make_source(Uuid::new_v4(), None),
        ]));
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.get_by_name("rust-lang/rust").await.unwrap();
        assert_eq!(result.unwrap().id, target_id);
    }

    #[tokio::test]
    async fn test_get_by_name_not_found() {
        let store = Arc::new(StubSourceStore::empty());
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.get_by_name("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    // ── add() tests ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_add_duplicate_url_updates_role_when_different() {
        let existing = make_source(Uuid::new_v4(), Some(SourceRole::Starred));
        let url = existing.url.clone();
        let store = Arc::new(StubSourceStore::with_sources(vec![existing.clone()]));
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr
            .add(AddSourceInput {
                url,
                name: None,
                source_type: None,
                role: Some(SourceRole::RoleModel),
                topic_id: None,
            })
            .await
            .unwrap();
        assert_eq!(result.id, existing.id);
        assert_eq!(result.role, Some(SourceRole::RoleModel));
    }

    #[tokio::test]
    async fn test_add_creates_new_source_for_unknown_url() {
        let store = Arc::new(StubSourceStore::empty());
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr
            .add(AddSourceInput {
                url: "https://github.com/rust-lang/rust".to_string(),
                name: None,
                source_type: None,
                role: None,
                topic_id: None,
            })
            .await
            .unwrap();
        assert_eq!(result.url, "https://github.com/rust-lang/rust");
        assert_eq!(result.name, "rust-lang/rust");
        assert!(matches!(result.state, SourceState::Active));
        assert_eq!(result.check_count, 0);
    }

    #[tokio::test]
    async fn test_add_returns_existing_source_for_duplicate_url() {
        let existing = make_source(Uuid::new_v4(), None);
        let url = existing.url.clone();
        let store = Arc::new(StubSourceStore::with_sources(vec![existing.clone()]));
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr
            .add(AddSourceInput {
                url,
                name: None,
                source_type: None,
                role: None,
                topic_id: None,
            })
            .await
            .unwrap();
        assert_eq!(result.id, existing.id);
    }

    #[tokio::test]
    async fn test_add_with_name_override_uses_provided_name() {
        let store = Arc::new(StubSourceStore::empty());
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr
            .add(AddSourceInput {
                url: "https://example.com/page".to_string(),
                name: Some("My Custom Name".to_string()),
                source_type: None,
                role: None,
                topic_id: None,
            })
            .await
            .unwrap();
        assert_eq!(result.name, "My Custom Name");
    }

    #[tokio::test]
    async fn test_add_with_topic_id_creates_monitors_edge() {
        struct CountingGraphStore {
            edges: Mutex<Vec<Edge>>,
        }
        #[async_trait]
        impl IGraphStore for CountingGraphStore {
            async fn add_edge(&self, e: Edge) -> Result<()> {
                self.edges.lock().unwrap().push(e);
                Ok(())
            }
            async fn remove_edge(&self, _id: Uuid) -> Result<()> {
                Ok(())
            }
            async fn get_edges_by_node(&self, _id: Uuid) -> Result<Vec<Edge>> {
                Ok(self.edges.lock().unwrap().clone())
            }
            async fn get_neighbors(&self, _id: Uuid, _et: Option<EdgeType>) -> Result<Vec<Edge>> {
                Ok(vec![])
            }
            async fn query(&self, _q: GraphQuery) -> Result<Vec<Edge>> {
                Ok(self.edges.lock().unwrap().clone())
            }
            async fn list(&self) -> Result<Vec<Edge>> {
                Ok(self.edges.lock().unwrap().clone())
            }
        }

        let topic_id = Uuid::new_v4();
        let store = Arc::new(StubSourceStore::empty());
        let graph = Arc::new(CountingGraphStore {
            edges: Mutex::new(vec![]),
        });
        let mgr = SourceManager::new(store, Arc::clone(&graph) as Arc<dyn IGraphStore>);
        mgr.add(AddSourceInput {
            url: "https://github.com/rust-lang/rust".to_string(),
            name: None,
            source_type: None,
            role: None,
            topic_id: Some(topic_id),
        })
        .await
        .unwrap();
        assert_eq!(
            graph.edges.lock().unwrap().len(),
            1,
            "expected exactly one Monitors edge"
        );
        let edge = graph.edges.lock().unwrap()[0].clone();
        assert_eq!(edge.from_id, topic_id);
        assert!(matches!(edge.edge_type, EdgeType::Monitors));
    }

    #[tokio::test]
    async fn test_add_duplicate_topic_edge_is_skipped() {
        struct CountingGraphStore2 {
            edges: Mutex<Vec<Edge>>,
        }
        #[async_trait]
        impl IGraphStore for CountingGraphStore2 {
            async fn add_edge(&self, e: Edge) -> Result<()> {
                self.edges.lock().unwrap().push(e);
                Ok(())
            }
            async fn remove_edge(&self, _id: Uuid) -> Result<()> {
                Ok(())
            }
            async fn get_edges_by_node(&self, _id: Uuid) -> Result<Vec<Edge>> {
                Ok(self.edges.lock().unwrap().clone())
            }
            async fn get_neighbors(&self, _id: Uuid, _et: Option<EdgeType>) -> Result<Vec<Edge>> {
                Ok(vec![])
            }
            async fn query(&self, _q: GraphQuery) -> Result<Vec<Edge>> {
                Ok(self.edges.lock().unwrap().clone())
            }
            async fn list(&self) -> Result<Vec<Edge>> {
                Ok(self.edges.lock().unwrap().clone())
            }
        }

        let topic_id = Uuid::new_v4();
        let url = "https://github.com/rust-lang/rust".to_string();
        let store = Arc::new(StubSourceStore::empty());
        let graph = Arc::new(CountingGraphStore2 {
            edges: Mutex::new(vec![]),
        });
        let mgr = SourceManager::new(store, Arc::clone(&graph) as Arc<dyn IGraphStore>);
        mgr.add(AddSourceInput {
            url: url.clone(),
            name: None,
            source_type: None,
            role: None,
            topic_id: Some(topic_id),
        })
        .await
        .unwrap();
        // Second call (same URL + same topic): edge must NOT be created again
        mgr.add(AddSourceInput {
            url,
            name: None,
            source_type: None,
            role: None,
            topic_id: Some(topic_id),
        })
        .await
        .unwrap();
        assert_eq!(
            graph.edges.lock().unwrap().len(),
            1,
            "dedup guard should prevent a second Monitors edge"
        );
    }
}
