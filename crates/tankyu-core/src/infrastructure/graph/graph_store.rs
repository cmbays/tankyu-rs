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
        Self { path, lock: Mutex::new(()) }
    }

    async fn read_index(&self) -> Result<GraphIndex> {
        match tokio::fs::read(&self.path).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(GraphIndex { version: 1, edges: vec![] })
            }
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
        let index = crate::domain::types::GraphIndex { version: 1, edges: vec![edge.clone()] };
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
}
