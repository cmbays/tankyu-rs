use std::path::Path;

use async_trait::async_trait;
use nanograph::store::database::Database;

use crate::domain::research_graph::{
    self, DoctorReport, IResearchGraph, MutationResult, ParamMap, ParamValue, QueryResult,
};
use crate::shared::error::TankyuError;

/// The nanograph schema source, embedded at compile time.
pub const SCHEMA_SOURCE: &str = include_str!("schema.pg");

/// Wraps a `nanograph::Database` behind the `IResearchGraph` port.
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
}

impl std::fmt::Debug for NanographStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NanographStore")
            .field("path", &self.db.path())
            .finish()
    }
}

/// Convert domain `ParamMap` to nanograph `ParamMap`.
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
impl IResearchGraph for NanographStore {
    async fn query(
        &self,
        source: &str,
        name: &str,
        params: &research_graph::ParamMap,
    ) -> Result<QueryResult, TankyuError> {
        let result = self.run_named(source, name, params).await?;

        match result {
            nanograph::RunResult::Query(qr) => {
                let rows_json = qr.to_rust_json();
                let rows = rows_json.as_array().cloned().unwrap_or_default();
                let num_rows = rows.len();
                Ok(QueryResult { rows, num_rows })
            }
            nanograph::RunResult::Mutation(_) => Err(TankyuError::Store(format!(
                "expected query result for '{name}', got mutation result"
            ))),
        }
    }

    async fn mutate(
        &self,
        source: &str,
        name: &str,
        params: &research_graph::ParamMap,
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

    async fn load(&self, jsonl: &str) -> Result<(), TankyuError> {
        self.db
            .load(jsonl)
            .await
            .map_err(|e| TankyuError::Store(format!("nanograph load: {e}")))
    }

    async fn doctor(&self) -> Result<DoctorReport, TankyuError> {
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
        let query_src = include_str!("queries/status.gq");

        // On an empty DB, count queries return 0 rows (no matching bindings),
        // so we treat num_rows == 0 as count = 0.
        let topics = store
            .query(query_src, "topicCount", &ParamMap::new())
            .await
            .unwrap();
        assert_eq!(topics.num_rows, 0);

        let sources = store
            .query(query_src, "sourceCount", &ParamMap::new())
            .await
            .unwrap();
        assert_eq!(sources.num_rows, 0);

        let entries = store
            .query(query_src, "entryCount", &ParamMap::new())
            .await
            .unwrap();
        assert_eq!(entries.num_rows, 0);
    }

    #[tokio::test]
    async fn load_and_query_counts() {
        let store = NanographStore::open_in_memory().await.unwrap();
        let jsonl = r#"{"type": "Topic", "data": {"slug": "rust", "name": "Rust"}}
{"type": "Topic", "data": {"slug": "wasm", "name": "WebAssembly"}}
{"type": "Source", "data": {"slug": "tokio-rs", "name": "tokio", "url": "https://github.com/tokio-rs/tokio", "sourceType": "github-repo", "state": "active"}}
"#;
        store.load(jsonl).await.unwrap();

        let query_src = include_str!("queries/status.gq");

        let topics = store
            .query(query_src, "topicCount", &ParamMap::new())
            .await
            .unwrap();
        assert_eq!(topics.rows[0]["count"], 2);

        let sources = store
            .query(query_src, "sourceCount", &ParamMap::new())
            .await
            .unwrap();
        assert_eq!(sources.rows[0]["count"], 1);
    }

    #[tokio::test]
    async fn doctor_healthy_on_empty_db() {
        let store = NanographStore::open_in_memory().await.unwrap();
        let report = store.doctor().await.unwrap();
        assert!(report.healthy, "expected healthy report: {report:?}");
        assert!(report.issues.is_empty());
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
            let query_src = include_str!("queries/status.gq");
            let topics = store
                .query(query_src, "topicCount", &ParamMap::new())
                .await
                .unwrap();
            assert_eq!(topics.rows[0]["count"], 1);
        }
    }
}
