use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::domain::{
    ports::{IEntryStore, IGraphStore},
    types::{EdgeType, Entry, EntryState, Signal},
};

/// Coordinates entry read operations.
pub struct EntryManager {
    store: Arc<dyn IEntryStore>,
    graph: Arc<dyn IGraphStore>,
}

impl EntryManager {
    /// Create an `EntryManager` backed by `store` and `graph`.
    #[must_use]
    pub fn new(store: Arc<dyn IEntryStore>, graph: Arc<dyn IGraphStore>) -> Self {
        Self { store, graph }
    }

    /// Return all entries.
    ///
    /// # Errors
    /// Returns an error if the underlying store read fails.
    pub async fn list_all(&self) -> Result<Vec<Entry>> {
        self.store.list().await
    }

    /// Return entries filtered by lifecycle state.
    ///
    /// # Errors
    /// Returns an error if the underlying store read fails.
    pub async fn list_by_state(&self, state: EntryState) -> Result<Vec<Entry>> {
        let all = self.store.list().await?;
        Ok(all.into_iter().filter(|e| e.state == state).collect())
    }

    /// Return entries with the given signal strength. Entries with `signal == None` are excluded.
    ///
    /// # Errors
    /// Returns an error if the underlying store read fails.
    pub async fn list_by_signal(&self, signal: Signal) -> Result<Vec<Entry>> {
        let all = self.store.list().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.signal.as_ref() == Some(&signal))
            .collect())
    }

    /// Return entries belonging to the given source.
    ///
    /// # Errors
    /// Returns an error if the underlying store read fails.
    pub async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>> {
        self.store.list_by_source(source_id).await
    }

    /// Return entries from sources monitored by the given topic.
    /// Walks `Monitors` edges from the topic node, then performs a single `store.list()` + filter.
    ///
    /// # Errors
    /// Returns an error if the graph store or entry store read fails.
    pub async fn list_by_topic(&self, topic_id: Uuid) -> Result<Vec<Entry>> {
        let edges = self.graph.get_edges_by_node(topic_id).await?;
        let source_ids: std::collections::HashSet<Uuid> = edges
            .into_iter()
            .filter(|e| e.from_id == topic_id && e.edge_type == EdgeType::Monitors)
            .map(|e| e.to_id)
            .collect();
        if source_ids.is_empty() {
            return Ok(vec![]);
        }
        let all = self.store.list().await?;
        Ok(all
            .into_iter()
            .filter(|e| source_ids.contains(&e.source_id))
            .collect())
    }

    /// Retrieve a single entry by UUID. Returns `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the underlying store read fails.
    pub async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
        self.store.get(id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::{
        ports::{IEntryStore, IGraphStore},
        types::{
            Edge, EdgeType, Entry, EntryState, EntryType, EntryUpdate, GraphQuery, NodeType, Signal,
        },
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    // ── Stubs ─────────────────────────────────────────────────────────────────

    struct StubEntryStore {
        entries: Vec<Entry>,
    }

    #[async_trait]
    impl IEntryStore for StubEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> {
            unimplemented!()
        }
        async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
            Ok(self.entries.iter().find(|e| e.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Entry>> {
            unimplemented!()
        }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> {
            unimplemented!()
        }
        async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>> {
            Ok(self
                .entries
                .iter()
                .filter(|e| e.source_id == source_id)
                .cloned()
                .collect())
        }
        async fn list(&self) -> Result<Vec<Entry>> {
            Ok(self.entries.clone())
        }
        async fn update(&self, _id: Uuid, _u: EntryUpdate) -> Result<Entry> {
            unimplemented!()
        }
    }

    struct StubGraphStore {
        edges: Vec<Edge>,
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

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_entry(title: &str, state: EntryState) -> Entry {
        Entry {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            r#type: EntryType::Article,
            title: title.to_string(),
            url: format!("https://example.com/{title}"),
            summary: None,
            content_hash: None,
            state,
            signal: None,
            scanned_at: Utc::now(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    fn make_entry_with_signal(title: &str, signal: Signal) -> Entry {
        let mut e = make_entry(title, EntryState::New);
        e.signal = Some(signal);
        e
    }

    fn make_monitors_edge(topic_id: Uuid, source_id: Uuid) -> Edge {
        Edge {
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
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    use super::EntryManager;

    #[tokio::test]
    async fn test_list_all_returns_all_entries() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry("Alpha", EntryState::New),
                make_entry("Beta", EntryState::Read),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_all_empty_returns_empty() {
        let store = Arc::new(StubEntryStore { entries: vec![] });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        assert!(mgr.list_all().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_by_state_returns_only_matching() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry("New one", EntryState::New),
                make_entry("Read one", EntryState::Read),
                make_entry("Also new", EntryState::New),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_state(EntryState::New).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|e| e.state == EntryState::New));
    }

    #[tokio::test]
    async fn test_list_by_state_excludes_none_matching() {
        let store = Arc::new(StubEntryStore {
            entries: vec![make_entry("Read one", EntryState::Read)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_state(EntryState::New).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_signal_returns_only_matching() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry_with_signal("High one", Signal::High),
                make_entry_with_signal("Low one", Signal::Low),
                make_entry("No signal", EntryState::New),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_signal(Signal::High).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "High one");
    }

    #[tokio::test]
    async fn test_list_by_signal_excludes_none_signal() {
        let store = Arc::new(StubEntryStore {
            entries: vec![make_entry("No signal", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_signal(Signal::High).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_source_returns_only_matching() {
        let source_id = Uuid::new_v4();
        let mut entry_for_source = make_entry("For source", EntryState::New);
        entry_for_source.source_id = source_id;
        let store = Arc::new(StubEntryStore {
            entries: vec![
                entry_for_source.clone(),
                make_entry("Other", EntryState::New),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_source(source_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, entry_for_source.id);
    }

    #[tokio::test]
    async fn test_list_by_topic_returns_entries_from_monitored_sources() {
        let topic_id = Uuid::new_v4();
        let source_id = Uuid::new_v4();
        let unrelated_source_id = Uuid::new_v4();

        let mut e1 = make_entry("Monitored entry", EntryState::New);
        e1.source_id = source_id;
        let mut e2 = make_entry("Unrelated entry", EntryState::New);
        e2.source_id = unrelated_source_id;

        let store = Arc::new(StubEntryStore {
            entries: vec![e1.clone(), e2],
        });
        let graph = Arc::new(StubGraphStore {
            edges: vec![make_monitors_edge(topic_id, source_id)],
        });
        let mgr = EntryManager::new(store, graph);

        let result = mgr.list_by_topic(topic_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, e1.id);
    }

    #[tokio::test]
    async fn test_list_by_topic_empty_graph_returns_empty() {
        let store = Arc::new(StubEntryStore {
            entries: vec![make_entry("Some entry", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_topic(Uuid::new_v4()).await.unwrap();
        assert!(result.is_empty());
    }

    /// Kills the `&&` → `||` mutant on line 70: a non-Monitors edge FROM the topic
    /// must NOT cause its `to_id` source to be included in results.
    #[tokio::test]
    async fn test_list_by_topic_ignores_non_monitors_edges() {
        let topic_id = Uuid::new_v4();
        let source_id = Uuid::new_v4();

        // Edge FROM topic but with a non-Monitors edge type (Produced).
        let non_monitors_edge = Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Produced, // not Monitors
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };

        let mut entry_for_source = make_entry("Should be excluded", EntryState::New);
        entry_for_source.source_id = source_id;

        let store = Arc::new(StubEntryStore {
            entries: vec![entry_for_source],
        });
        let graph = Arc::new(StubGraphStore {
            edges: vec![non_monitors_edge],
        });
        let mgr = EntryManager::new(store, graph);

        let result = mgr.list_by_topic(topic_id).await.unwrap();
        // With the correct `&&`, no Monitors edges exist → result is empty.
        // With the mutated `||`, the Produced edge would match → result would have 1 entry.
        assert!(
            result.is_empty(),
            "Non-Monitors edge must not cause entries to be returned"
        );
    }

    /// Kills the `&&` → `||` mutant on line 70: a Monitors edge pointing TO the topic
    /// (reverse direction) must NOT cause its `from_id` to be used as a source.
    #[tokio::test]
    async fn test_list_by_topic_ignores_incoming_monitors_edges() {
        let topic_id = Uuid::new_v4();
        let other_id = Uuid::new_v4(); // some other node that monitors the topic

        // Edge from other_id TO topic_id (topic is the target, not origin).
        let incoming_monitors_edge = Edge {
            id: Uuid::new_v4(),
            from_id: other_id,
            from_type: NodeType::Source,
            to_id: topic_id, // topic is the destination
            to_type: NodeType::Topic,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        };

        let mut entry_for_other = make_entry("Also excluded", EntryState::New);
        entry_for_other.source_id = other_id;

        let store = Arc::new(StubEntryStore {
            entries: vec![entry_for_other],
        });
        let graph = Arc::new(StubGraphStore {
            edges: vec![incoming_monitors_edge],
        });
        let mgr = EntryManager::new(store, graph);

        let result = mgr.list_by_topic(topic_id).await.unwrap();
        // With correct `&&`: from_id != topic_id, so no source_ids collected → empty.
        // With mutated `||`: edge_type == Monitors satisfies ||, so other_id would
        // be collected as a source_id → "Also excluded" would appear.
        assert!(
            result.is_empty(),
            "Incoming Monitors edge (from_id != topic_id) must not cause entries to be returned"
        );
    }

    #[tokio::test]
    async fn test_get_returns_entry_by_id() {
        let entry = make_entry("Target", EntryState::New);
        let entry_id = entry.id;
        let store = Arc::new(StubEntryStore {
            entries: vec![entry, make_entry("Other", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.get(entry_id).await.unwrap();
        assert_eq!(result.unwrap().id, entry_id);
    }

    #[tokio::test]
    async fn test_get_returns_none_for_missing_id() {
        let store = Arc::new(StubEntryStore { entries: vec![] });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.get(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }
}
