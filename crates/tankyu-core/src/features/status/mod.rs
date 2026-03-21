use std::sync::Arc;

use serde::Serialize;

use crate::domain::research_graph::{IResearchGraph, ParamMap};
use crate::shared::error::TankyuError;

const STATUS_QUERIES: &str = include_str!("../../infrastructure/nanograph/queries/status.gq");

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReport {
    pub topics: usize,
    pub sources: usize,
    pub entries: usize,
}

pub struct StatusUseCase {
    graph: Arc<dyn IResearchGraph>,
}

impl StatusUseCase {
    #[must_use]
    pub fn new(graph: Arc<dyn IResearchGraph>) -> Self {
        Self { graph }
    }

    /// Gather counts for the status dashboard.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if any count query fails.
    pub async fn run(&self) -> Result<StatusReport, TankyuError> {
        let empty = ParamMap::new();

        let topics = self
            .graph
            .query(STATUS_QUERIES, "topicCount", &empty)
            .await?
            .first_count("count");

        let sources = self
            .graph
            .query(STATUS_QUERIES, "sourceCount", &empty)
            .await?
            .first_count("count");

        let entries = self
            .graph
            .query(STATUS_QUERIES, "entryCount", &empty)
            .await?
            .first_count("count");

        Ok(StatusReport {
            topics,
            sources,
            entries,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::nanograph::NanographStore;

    #[tokio::test]
    async fn status_returns_zeros_on_empty_graph() {
        let graph: Arc<dyn IResearchGraph> =
            Arc::new(NanographStore::open_in_memory().await.unwrap());
        let uc = StatusUseCase::new(graph);
        let report = uc.run().await.unwrap();
        assert_eq!(report.topics, 0);
        assert_eq!(report.sources, 0);
        assert_eq!(report.entries, 0);
    }

    #[tokio::test]
    async fn status_reflects_loaded_data() {
        let store = NanographStore::open_in_memory().await.unwrap();
        store
            .load(
                r#"{"type": "Topic", "data": {"slug": "rust", "name": "Rust"}}
{"type": "Source", "data": {"slug": "tokio", "name": "tokio", "url": "https://github.com/tokio-rs/tokio", "sourceType": "github-repo", "state": "active"}}
"#,
            )
            .await
            .unwrap();

        let graph: Arc<dyn IResearchGraph> = Arc::new(store);
        let uc = StatusUseCase::new(graph);
        let report = uc.run().await.unwrap();
        assert_eq!(report.topics, 1);
        assert_eq!(report.sources, 1);
        assert_eq!(report.entries, 0);
    }
}
