use anyhow::Result;
use async_trait::async_trait;
use std::path::PathBuf;
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::domain::{
    ports::IGraphStore,
    types::{Edge, EdgeType, GraphIndex, GraphQuery},
};

/// JSON-backed store for the knowledge graph, backed by a `GraphIndex` envelope.
///
/// Reads and writes `{edges_path}` as `{ "version": 1, "edges": [...] }`.
/// A `Mutex` serializes concurrent writes to prevent interleaved atomic renames.
pub struct JsonGraphStore {
    path: PathBuf,
    lock: Mutex<()>,
}

impl JsonGraphStore {
    /// Create a new `JsonGraphStore` backed by the file at `path`.
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            lock: Mutex::new(()),
        }
    }

    async fn read_index(&self) -> Result<GraphIndex> {
        match tokio::fs::read(&self.path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(GraphIndex {
                version: 1,
                edges: vec![],
            }),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(serde_json::from_slice(&bytes)?),
        }
    }

    async fn write_index(&self, index: &GraphIndex) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(index)?;
        let tmp = self.path.with_extension("tmp");
        tokio::fs::write(&tmp, json).await?;
        tokio::fs::rename(&tmp, &self.path).await?;
        Ok(())
    }
}

#[async_trait]
impl IGraphStore for JsonGraphStore {
    async fn add_edge(&self, edge: Edge) -> Result<()> {
        let _guard = self.lock.lock().await;
        let mut index = self.read_index().await?;
        index.edges.push(edge);
        self.write_index(&index).await
    }

    async fn remove_edge(&self, id: Uuid) -> Result<()> {
        let _guard = self.lock.lock().await;
        let mut index = self.read_index().await?;
        index.edges.retain(|e| e.id != id);
        self.write_index(&index).await
    }

    async fn get_edges_by_node(&self, node_id: Uuid) -> Result<Vec<Edge>> {
        let index = self.read_index().await?;
        Ok(index
            .edges
            .into_iter()
            .filter(|e| e.from_id == node_id || e.to_id == node_id)
            .collect())
    }

    async fn get_neighbors(&self, node_id: Uuid, edge_type: Option<EdgeType>) -> Result<Vec<Edge>> {
        let index = self.read_index().await?;
        Ok(index
            .edges
            .into_iter()
            .filter(|e| {
                (e.from_id == node_id || e.to_id == node_id)
                    && edge_type.as_ref().is_none_or(|et| &e.edge_type == et)
            })
            .collect())
    }

    async fn query(&self, opts: GraphQuery) -> Result<Vec<Edge>> {
        let index = self.read_index().await?;
        Ok(index
            .edges
            .into_iter()
            .filter(|e| {
                opts.from_type.as_ref().is_none_or(|ft| &e.from_type == ft)
                    && opts.to_type.as_ref().is_none_or(|tt| &e.to_type == tt)
                    && opts.edge_type.as_ref().is_none_or(|et| &e.edge_type == et)
                    && opts.from_id.is_none_or(|id| e.from_id == id)
                    && opts.to_id.is_none_or(|id| e.to_id == id)
            })
            .collect())
    }

    async fn list(&self) -> Result<Vec<Edge>> {
        let index = self.read_index().await?;
        Ok(index.edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::{EdgeType, NodeType};
    use chrono::Utc;
    use tempfile::tempdir;
    use uuid::Uuid;

    fn make_edge() -> Edge {
        Edge {
            id: Uuid::new_v4(),
            from_id: Uuid::new_v4(),
            from_type: NodeType::Topic,
            to_id: Uuid::new_v4(),
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn add_then_list() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let edge = make_edge();
        store.add_edge(edge.clone()).await.unwrap();
        let edges = store.list().await.unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].id, edge.id);
    }

    #[tokio::test]
    async fn remove_edge() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let edge = make_edge();
        store.add_edge(edge.clone()).await.unwrap();
        store.remove_edge(edge.id).await.unwrap();
        let edges = store.list().await.unwrap();
        assert!(edges.is_empty());
    }

    #[tokio::test]
    async fn reads_graphindex_envelope() {
        // Verify that the store reads the versioned envelope { version: 1, edges: [...] }
        let dir = tempdir().unwrap();
        let path = dir.path().join("edges.json");
        let edge = make_edge();
        let index = crate::domain::types::GraphIndex {
            version: 1,
            edges: vec![edge.clone()],
        };
        let json = serde_json::to_string(&index).unwrap();
        tokio::fs::write(&path, json).await.unwrap();
        let store = JsonGraphStore::new(path);
        let edges = store.list().await.unwrap();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].id, edge.id);
    }

    #[tokio::test]
    async fn empty_store_returns_empty_list() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("edges.json"));
        let edges = store.list().await.unwrap();
        assert!(edges.is_empty());
    }

    /// Helper to create an edge with specific endpoints and types.
    fn make_edge_between(
        from_id: Uuid,
        to_id: Uuid,
        from_type: NodeType,
        to_type: NodeType,
        edge_type: EdgeType,
    ) -> Edge {
        Edge {
            id: Uuid::new_v4(),
            from_id,
            from_type,
            to_id,
            to_type,
            edge_type,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn get_edges_by_node_returns_matching() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();
        let e = Uuid::new_v4();

        // A→B, B→C, D→E
        store
            .add_edge(make_edge_between(a, b, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();
        store
            .add_edge(make_edge_between(b, c, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();
        store
            .add_edge(make_edge_between(d, e, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();

        let result = store.get_edges_by_node(b).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|edge| edge.from_id == b || edge.to_id == b));
    }

    #[tokio::test]
    async fn get_edges_by_node_empty_when_no_match() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        store
            .add_edge(make_edge_between(a, b, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();

        let result = store.get_edges_by_node(c).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn get_neighbors_unfiltered() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();
        let d = Uuid::new_v4();

        // Outgoing from a
        store
            .add_edge(make_edge_between(a, b, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();
        // Outgoing from a
        store
            .add_edge(make_edge_between(a, c, NodeType::Topic, NodeType::Entry, EdgeType::Produced))
            .await
            .unwrap();
        // Incoming to a (tests to_id == node_id path)
        store
            .add_edge(make_edge_between(d, a, NodeType::Source, NodeType::Topic, EdgeType::Monitors))
            .await
            .unwrap();

        let result = store.get_neighbors(a, None).await.unwrap();
        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn get_neighbors_filtered_by_type() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();
        let c = Uuid::new_v4();

        store
            .add_edge(make_edge_between(a, b, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();
        store
            .add_edge(make_edge_between(a, c, NodeType::Topic, NodeType::Entry, EdgeType::Produced))
            .await
            .unwrap();

        let result = store.get_neighbors(a, Some(EdgeType::Monitors)).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edge_type, EdgeType::Monitors);
    }

    #[tokio::test]
    async fn get_neighbors_filtered_no_match() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let a = Uuid::new_v4();
        let b = Uuid::new_v4();

        store
            .add_edge(make_edge_between(a, b, NodeType::Topic, NodeType::Source, EdgeType::Monitors))
            .await
            .unwrap();

        let result = store.get_neighbors(a, Some(EdgeType::Produced)).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn query_filters_by_from_type() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));

        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), Uuid::new_v4(),
                NodeType::Topic, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();
        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), Uuid::new_v4(),
                NodeType::Entry, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        let result = store
            .query(GraphQuery {
                from_type: Some(NodeType::Topic),
                to_type: None,
                edge_type: None,
                from_id: None,
                to_id: None,
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].from_type, NodeType::Topic);
    }

    #[tokio::test]
    async fn query_filters_by_to_type() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));

        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), Uuid::new_v4(),
                NodeType::Topic, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();
        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), Uuid::new_v4(),
                NodeType::Topic, NodeType::Entry, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        let result = store
            .query(GraphQuery {
                from_type: None,
                to_type: Some(NodeType::Entry),
                edge_type: None,
                from_id: None,
                to_id: None,
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].to_type, NodeType::Entry);
    }

    #[tokio::test]
    async fn query_filters_by_edge_type() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));

        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), Uuid::new_v4(),
                NodeType::Topic, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();
        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), Uuid::new_v4(),
                NodeType::Topic, NodeType::Source, EdgeType::Produced,
            ))
            .await
            .unwrap();

        let result = store
            .query(GraphQuery {
                from_type: None,
                to_type: None,
                edge_type: Some(EdgeType::Produced),
                from_id: None,
                to_id: None,
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].edge_type, EdgeType::Produced);
    }

    #[tokio::test]
    async fn query_combines_all_filters() {
        let dir = tempdir().unwrap();
        let store = JsonGraphStore::new(dir.path().join("graph").join("edges.json"));
        let target_from = Uuid::new_v4();
        let target_to = Uuid::new_v4();

        // The one edge that matches all 5 filters
        store
            .add_edge(make_edge_between(
                target_from, target_to,
                NodeType::Topic, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        // Wrong from_type
        store
            .add_edge(make_edge_between(
                target_from, target_to,
                NodeType::Entry, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        // Wrong to_type
        store
            .add_edge(make_edge_between(
                target_from, target_to,
                NodeType::Topic, NodeType::Entry, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        // Wrong edge_type
        store
            .add_edge(make_edge_between(
                target_from, target_to,
                NodeType::Topic, NodeType::Source, EdgeType::Produced,
            ))
            .await
            .unwrap();

        // Wrong from_id
        store
            .add_edge(make_edge_between(
                Uuid::new_v4(), target_to,
                NodeType::Topic, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        // Wrong to_id
        store
            .add_edge(make_edge_between(
                target_from, Uuid::new_v4(),
                NodeType::Topic, NodeType::Source, EdgeType::Monitors,
            ))
            .await
            .unwrap();

        let result = store
            .query(GraphQuery {
                from_type: Some(NodeType::Topic),
                to_type: Some(NodeType::Source),
                edge_type: Some(EdgeType::Monitors),
                from_id: Some(target_from),
                to_id: Some(target_to),
            })
            .await
            .unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].from_id, target_from);
        assert_eq!(result[0].to_id, target_to);
    }

    #[tokio::test]
    async fn read_index_non_not_found_error_propagates() {
        let dir = tempdir().unwrap();
        // Point the store at a path that is a directory, not a file.
        // Reading a directory produces an io error that is NOT NotFound,
        // so it should propagate rather than returning an empty vec.
        let dir_path = dir.path().join("a_directory");
        tokio::fs::create_dir_all(&dir_path).await.unwrap();
        let store = JsonGraphStore::new(dir_path);

        let result = store.list().await;
        assert!(result.is_err(), "expected an error for non-NotFound io failure");
    }
}
