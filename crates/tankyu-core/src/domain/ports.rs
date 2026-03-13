use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::domain::types::{
    Edge, EdgeType, Entity, Entry, EntryUpdate, GraphQuery, Insight, InsightUpdate, Source,
    SourceUpdate, Topic, TopicUpdate,
};

/// Store for persisting and querying [`Topic`] records.
#[async_trait]
pub trait ITopicStore: Send + Sync {
    /// Persist a new topic.
    ///
    /// # Errors
    /// Returns an error if the write fails.
    async fn create(&self, topic: Topic) -> Result<()>;

    /// Retrieve a topic by its UUID.
    ///
    /// # Errors
    /// Returns an error if the read fails (not-found yields `Ok(None)`).
    async fn get(&self, id: Uuid) -> Result<Option<Topic>>;

    /// Find a topic by its name (case-sensitive).
    ///
    /// # Errors
    /// Returns an error if the scan fails.
    async fn get_by_name(&self, name: &str) -> Result<Option<Topic>>;

    /// Return all topics.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn list(&self) -> Result<Vec<Topic>>;

    /// Apply partial updates to a topic and persist the result.
    ///
    /// # Errors
    /// Returns an error if the topic is not found or the write fails.
    async fn update(&self, id: Uuid, updates: TopicUpdate) -> Result<Topic>;
}

/// Store for persisting and querying [`Source`] records.
#[async_trait]
pub trait ISourceStore: Send + Sync {
    /// Persist a new source.
    ///
    /// # Errors
    /// Returns an error if the write fails.
    async fn create(&self, source: Source) -> Result<()>;

    /// Retrieve a source by its UUID.
    ///
    /// # Errors
    /// Returns an error if the read fails (not-found yields `Ok(None)`).
    async fn get(&self, id: Uuid) -> Result<Option<Source>>;

    /// Find a source by its URL.
    ///
    /// # Errors
    /// Returns an error if the scan fails.
    async fn get_by_url(&self, url: &str) -> Result<Option<Source>>;

    /// Return all sources.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn list(&self) -> Result<Vec<Source>>;

    /// Apply partial updates to a source and persist the result.
    ///
    /// # Errors
    /// Returns an error if the source is not found or the write fails.
    async fn update(&self, id: Uuid, updates: SourceUpdate) -> Result<Source>;
}

/// Store for persisting and querying [`Entry`] records.
#[async_trait]
pub trait IEntryStore: Send + Sync {
    /// Persist a new entry.
    ///
    /// # Errors
    /// Returns an error if the write fails.
    async fn create(&self, entry: Entry) -> Result<()>;

    /// Retrieve an entry by its UUID.
    ///
    /// # Errors
    /// Returns an error if the read fails (not-found yields `Ok(None)`).
    async fn get(&self, id: Uuid) -> Result<Option<Entry>>;

    /// Find an entry by its URL.
    ///
    /// # Errors
    /// Returns an error if the scan fails.
    async fn get_by_url(&self, url: &str) -> Result<Option<Entry>>;

    /// Find an entry by its content hash (for deduplication).
    ///
    /// # Errors
    /// Returns an error if the scan fails.
    async fn get_by_content_hash(&self, hash: &str) -> Result<Option<Entry>>;

    /// Return all entries belonging to a specific source.
    ///
    /// # Errors
    /// Returns an error if the scan fails.
    async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>>;

    /// Return all entries.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn list(&self) -> Result<Vec<Entry>>;

    /// Apply partial updates to an entry and persist the result.
    ///
    /// # Errors
    /// Returns an error if the entry is not found or the write fails.
    async fn update(&self, id: Uuid, updates: EntryUpdate) -> Result<Entry>;
}

/// Store for persisting and querying [`Insight`] records.
#[async_trait]
pub trait IInsightStore: Send + Sync {
    /// Persist a new insight.
    ///
    /// # Errors
    /// Returns an error if the write fails.
    async fn create(&self, insight: Insight) -> Result<()>;

    /// Retrieve an insight by its UUID.
    ///
    /// # Errors
    /// Returns an error if the read fails (not-found yields `Ok(None)`).
    async fn get(&self, id: Uuid) -> Result<Option<Insight>>;

    /// Return all insights.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn list(&self) -> Result<Vec<Insight>>;

    /// Apply partial updates to an insight and persist the result.
    ///
    /// # Errors
    /// Returns an error if the insight is not found or the write fails.
    async fn update(&self, id: Uuid, updates: InsightUpdate) -> Result<Insight>;
}

/// Store for persisting and querying [`Entity`] records.
#[async_trait]
pub trait IEntityStore: Send + Sync {
    /// Persist a new entity.
    ///
    /// # Errors
    /// Returns an error if the write fails.
    async fn create(&self, entity: Entity) -> Result<()>;

    /// Retrieve an entity by its UUID.
    ///
    /// # Errors
    /// Returns an error if the read fails (not-found yields `Ok(None)`).
    async fn get(&self, id: Uuid) -> Result<Option<Entity>>;

    /// Find an entity by its name (case-sensitive).
    ///
    /// # Errors
    /// Returns an error if the scan fails.
    async fn get_by_name(&self, name: &str) -> Result<Option<Entity>>;

    /// Return all entities.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn list(&self) -> Result<Vec<Entity>>;
}

/// Store for persisting and querying graph [`Edge`] records.
#[async_trait]
pub trait IGraphStore: Send + Sync {
    /// Add a new edge to the graph.
    ///
    /// # Errors
    /// Returns an error if the write fails.
    async fn add_edge(&self, edge: Edge) -> Result<()>;

    /// Remove an edge by its UUID.
    ///
    /// # Errors
    /// Returns an error if the edge is not found or the write fails.
    async fn remove_edge(&self, id: Uuid) -> Result<()>;

    /// Return all edges connected to the given node.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn get_edges_by_node(&self, node_id: Uuid) -> Result<Vec<Edge>>;

    /// Return neighboring edges from a node, optionally filtered by edge type.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn get_neighbors(&self, node_id: Uuid, edge_type: Option<EdgeType>) -> Result<Vec<Edge>>;

    /// Query edges using structured filter options.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn query(&self, opts: GraphQuery) -> Result<Vec<Edge>>;

    /// Return all edges in the graph.
    ///
    /// # Errors
    /// Returns an error if the read fails.
    async fn list(&self) -> Result<Vec<Edge>>;
}
