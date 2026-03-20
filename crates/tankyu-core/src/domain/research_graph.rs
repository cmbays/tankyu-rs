use std::collections::HashMap;

use async_trait::async_trait;

use crate::shared::error::TankyuError;

/// Wrapper for query parameters so nanograph types don't leak into the domain.
pub type ParamMap = HashMap<String, ParamValue>;

/// Domain-level parameter value — maps to nanograph `Literal` at the infrastructure boundary.
#[derive(Debug, Clone)]
pub enum ParamValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

/// Rows returned from a read query, already deserialized to JSON.
#[derive(Debug, Clone)]
pub struct QueryResult {
    pub rows: Vec<serde_json::Value>,
    pub num_rows: usize,
}

impl QueryResult {
    /// Extract a single integer field from the first row, defaulting to 0
    /// if no rows were returned (e.g. aggregate on an empty dataset).
    #[must_use]
    pub fn first_count(&self, field: &str) -> usize {
        self.rows
            .first()
            .and_then(|row| row.get(field))
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| usize::try_from(v).ok())
            .unwrap_or(0)
    }
}

/// Result of a write mutation.
#[derive(Debug, Clone, Copy, Default)]
pub struct MutationResult {
    pub affected_nodes: usize,
    pub affected_edges: usize,
}

/// Database health report surfaced by `doctor()`.
#[derive(Debug, Clone, Default)]
pub struct DoctorReport {
    pub healthy: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub datasets_checked: usize,
}

/// Port trait for the research graph database.
///
/// Implementations wrap a concrete graph engine (e.g. nanograph) and expose
/// only domain-level types so the rest of the codebase never depends on the
/// engine directly.
#[async_trait]
pub trait IResearchGraph: Send + Sync {
    /// Run a named read query from the given source text.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the query fails.
    async fn query(
        &self,
        source: &str,
        name: &str,
        params: &ParamMap,
    ) -> Result<QueryResult, TankyuError>;

    /// Run a named mutation from the given source text.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the mutation fails.
    async fn mutate(
        &self,
        source: &str,
        name: &str,
        params: &ParamMap,
    ) -> Result<MutationResult, TankyuError>;

    /// Load JSONL data into the graph (merge mode for keyed nodes).
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the load fails.
    async fn load(&self, jsonl: &str) -> Result<(), TankyuError>;

    /// Run database diagnostics.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the check fails.
    async fn doctor(&self) -> Result<DoctorReport, TankyuError>;
}
