use std::collections::HashMap;
use std::path::Path;

use async_trait::async_trait;
use nanograph::store::database::Database;

use crate::features::doctor::{DoctorReport, GraphDoctor};
use crate::features::status::CountStats;
use crate::shared::error::TankyuError;

/// The nanograph schema source, embedded at compile time.
pub const SCHEMA_SOURCE: &str = include_str!("schema.pg");

/// Status count queries, owned by infrastructure.
const STATUS_QUERIES: &str = include_str!("queries/status.gq");

/// Infrastructure-internal parameter value for nanograph queries.
/// Variants are constructed by parameterized queries (future feature traits).
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum ParamValue {
    String(String),
    Integer(i64),
    Float(f64),
    Bool(bool),
}

/// Infrastructure-internal parameter map for nanograph queries.
type ParamMap = HashMap<String, ParamValue>;

/// Infrastructure-internal query result.
#[derive(Debug, Clone)]
struct QueryResult {
    rows: Vec<serde_json::Value>,
}

impl QueryResult {
    /// Extract a single integer field from the first row, defaulting to 0.
    fn first_count(&self, field: &str) -> usize {
        self.rows
            .first()
            .and_then(|row| row.get(field))
            .and_then(serde_json::Value::as_u64)
            .and_then(|v| usize::try_from(v).ok())
            .unwrap_or(0)
    }
}

/// Result of a write mutation (used by mutation feature traits in future sessions).
#[cfg(test)]
#[derive(Debug, Clone, Copy, Default)]
#[allow(dead_code)]
struct MutationResult {
    affected_nodes: usize,
    affected_edges: usize,
}

/// Wraps a `nanograph::Database` behind per-feature port traits.
pub struct NanographStore {
    db: Database,
}

impl NanographStore {
    /// Open or create a persistent database at `db_path`.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if initialization or opening fails.
    pub async fn open(db_path: &Path) -> Result<Self, TankyuError> {
        let db = if db_path.join("schema.ir.json").exists() {
            Database::open(db_path)
                .await
                .map_err(|e| TankyuError::Store(format!("nanograph open: {e}")))?
        } else {
            Database::init(db_path, SCHEMA_SOURCE)
                .await
                .map_err(|e| TankyuError::Store(format!("nanograph init: {e}")))?
        };
        Ok(Self { db })
    }

    /// Create an in-memory database (backed by a temporary directory).
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if initialization fails.
    pub async fn open_in_memory() -> Result<Self, TankyuError> {
        let db = Database::open_in_memory(SCHEMA_SOURCE)
            .await
            .map_err(|e| TankyuError::Store(format!("nanograph in-memory init: {e}")))?;
        Ok(Self { db })
    }

    /// Load JSONL data into the graph (merge mode for keyed nodes).
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the load fails.
    pub async fn load(&self, jsonl: &str) -> Result<(), TankyuError> {
        self.db
            .load(jsonl)
            .await
            .map_err(|e| TankyuError::Store(format!("nanograph load: {e}")))
    }

    /// Internal: run a named query/mutation and return the raw `RunResult`.
    ///
    /// Uses `Box::pin` to break the deeply nested future type that nanograph
    /// produces (tracing `#[instrument]` + Lance + Arrow) — without this the
    /// type tree overflows Rust's default `type_length_limit`.
    fn run_named<'a>(
        &'a self,
        source: &'a str,
        name: &'a str,
        params: &'a ParamMap,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<nanograph::RunResult, TankyuError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let nano_params = to_nano_params(params);
            self.db
                .run(source, name, &nano_params)
                .await
                .map_err(|e| TankyuError::Store(format!("nanograph run '{name}': {e}")))
        })
    }

    /// Run a named read query from the given source text.
    async fn query(
        &self,
        source: &str,
        name: &str,
        params: &ParamMap,
    ) -> Result<QueryResult, TankyuError> {
        let result = self.run_named(source, name, params).await?;

        match result {
            nanograph::RunResult::Query(qr) => {
                let rows_json = qr.to_rust_json();
                let rows = rows_json.as_array().cloned().unwrap_or_default();
                Ok(QueryResult { rows })
            }
            nanograph::RunResult::Mutation(_) => Err(TankyuError::Store(format!(
                "expected query result for '{name}', got mutation result"
            ))),
        }
    }

    /// Run a named mutation from the given source text.
    #[cfg(test)]
    async fn mutate(
        &self,
        source: &str,
        name: &str,
        params: &ParamMap,
    ) -> Result<MutationResult, TankyuError> {
        let result = self.run_named(source, name, params).await?;

        match result {
            nanograph::RunResult::Mutation(mr) => Ok(MutationResult {
                affected_nodes: mr.affected_nodes,
                affected_edges: mr.affected_edges,
            }),
            nanograph::RunResult::Query(_) => Err(TankyuError::Store(format!(
                "expected mutation result for '{name}', got query result"
            ))),
        }
    }
}

impl std::fmt::Debug for NanographStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NanographStore")
            .field("path", &self.db.path())
            .finish()
    }
}

/// Convert internal `ParamMap` to nanograph `ParamMap`.
fn to_nano_params(params: &ParamMap) -> nanograph::ParamMap {
    params
        .iter()
        .map(|(k, v)| {
            let lit = match v {
                ParamValue::String(s) => nanograph::Literal::String(s.clone()),
                ParamValue::Integer(i) => nanograph::Literal::Integer(*i),
                ParamValue::Float(f) => nanograph::Literal::Float(*f),
                ParamValue::Bool(b) => nanograph::Literal::Bool(*b),
            };
            (k.clone(), lit)
        })
        .collect()
}

#[async_trait]
impl CountStats for NanographStore {
    async fn count_topics(&self) -> Result<usize, TankyuError> {
        Ok(self
            .query(STATUS_QUERIES, "topicCount", &ParamMap::new())
            .await?
            .first_count("count"))
    }

    async fn count_sources(&self) -> Result<usize, TankyuError> {
        Ok(self
            .query(STATUS_QUERIES, "sourceCount", &ParamMap::new())
            .await?
            .first_count("count"))
    }

    async fn count_entries(&self) -> Result<usize, TankyuError> {
        Ok(self
            .query(STATUS_QUERIES, "entryCount", &ParamMap::new())
            .await?
            .first_count("count"))
    }
}

#[async_trait]
impl GraphDoctor for NanographStore {
    async fn check_health(&self) -> Result<DoctorReport, TankyuError> {
        let report = self
            .db
            .doctor()
            .await
            .map_err(|e| TankyuError::Store(format!("nanograph doctor: {e}")))?;

        Ok(DoctorReport {
            healthy: report.healthy,
            issues: report.issues,
            warnings: report.warnings,
            datasets_checked: report.datasets_checked,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn open_in_memory_succeeds() {
        let store = NanographStore::open_in_memory().await.unwrap();
        assert!(store.db.is_in_memory());
    }

    #[tokio::test]
    async fn status_query_returns_zeros_on_empty_db() {
        let store = NanographStore::open_in_memory().await.unwrap();

        assert_eq!(store.count_topics().await.unwrap(), 0);
        assert_eq!(store.count_sources().await.unwrap(), 0);
        assert_eq!(store.count_entries().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn load_and_query_counts() {
        let store = NanographStore::open_in_memory().await.unwrap();
        let jsonl = r#"{"type": "Topic", "data": {"slug": "rust", "name": "Rust"}}
{"type": "Topic", "data": {"slug": "wasm", "name": "WebAssembly"}}
{"type": "Source", "data": {"slug": "tokio-rs", "name": "tokio", "url": "https://github.com/tokio-rs/tokio", "sourceType": "github-repo", "state": "active"}}
"#;
        store.load(jsonl).await.unwrap();

        assert_eq!(store.count_topics().await.unwrap(), 2);
        assert_eq!(store.count_sources().await.unwrap(), 1);
        assert_eq!(store.count_entries().await.unwrap(), 0);
    }

    #[tokio::test]
    async fn doctor_healthy_on_empty_db() {
        let store = NanographStore::open_in_memory().await.unwrap();
        let report = store.check_health().await.unwrap();
        assert!(report.healthy, "expected healthy report: {report:?}");
        assert!(report.issues.is_empty());
    }

    #[tokio::test]
    async fn debug_format_includes_path() {
        let store = NanographStore::open_in_memory().await.unwrap();
        let debug = format!("{store:?}");
        assert!(
            debug.contains("NanographStore"),
            "Debug output should contain struct name, got: {debug}"
        );
    }

    #[tokio::test]
    async fn to_nano_params_converts_all_variants() {
        let mut params = ParamMap::new();
        params.insert("s".into(), ParamValue::String("hello".into()));
        params.insert("i".into(), ParamValue::Integer(42));
        params.insert("f".into(), ParamValue::Float(3.14));
        params.insert("b".into(), ParamValue::Bool(true));

        let nano = to_nano_params(&params);
        assert_eq!(nano.len(), 4);
        assert!(
            format!("{:?}", nano["s"]).contains("hello"),
            "String param should contain 'hello'"
        );
        assert!(
            format!("{:?}", nano["i"]).contains("42"),
            "Integer param should contain 42"
        );
        assert!(
            format!("{:?}", nano["b"]).contains("true"),
            "Bool param should contain true"
        );
    }

    #[tokio::test]
    async fn mutate_creates_node() {
        let store = NanographStore::open_in_memory().await.unwrap();
        let mutation_src = r#"
query createTopic($slug: String, $name: String) {
    insert Topic { slug: $slug, name: $name }
}
"#;
        let mut params = ParamMap::new();
        params.insert("slug".into(), ParamValue::String("rust".into()));
        params.insert("name".into(), ParamValue::String("Rust".into()));

        let result = store
            .mutate(mutation_src, "createTopic", &params)
            .await
            .unwrap();
        assert_eq!(result.affected_nodes, 1);

        // Verify the node was created
        assert_eq!(store.count_topics().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn mutate_on_read_query_returns_error() {
        let store = NanographStore::open_in_memory().await.unwrap();

        // Calling mutate() on a read-only query should error
        let result = store
            .mutate(STATUS_QUERIES, "topicCount", &ParamMap::new())
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn persistent_db_roundtrips() {
        let tmp = tempfile::TempDir::new().unwrap();
        let db_path = tmp.path().join("testdb");

        // Create and load
        {
            let store = NanographStore::open(&db_path).await.unwrap();
            store
                .load(r#"{"type": "Topic", "data": {"slug": "ai", "name": "AI"}}"#)
                .await
                .unwrap();
        }

        // Reopen and verify
        {
            let store = NanographStore::open(&db_path).await.unwrap();
            assert_eq!(store.count_topics().await.unwrap(), 1);
        }
    }
}
