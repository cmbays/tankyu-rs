use std::sync::Arc;

use async_trait::async_trait;
use serde::Serialize;

use crate::shared::error::TankyuError;

/// Port trait for aggregate count statistics on the research graph.
#[async_trait]
pub trait CountStats: Send + Sync {
    /// Count the number of topics in the graph.
    async fn count_topics(&self) -> Result<usize, TankyuError>;
    /// Count the number of sources in the graph.
    async fn count_sources(&self) -> Result<usize, TankyuError>;
    /// Count the number of entries in the graph.
    async fn count_entries(&self) -> Result<usize, TankyuError>;
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusReport {
    pub topics: usize,
    pub sources: usize,
    pub entries: usize,
}

pub struct StatusUseCase {
    stats: Arc<dyn CountStats>,
}

impl StatusUseCase {
    #[must_use]
    pub fn new(stats: Arc<dyn CountStats>) -> Self {
        Self { stats }
    }

    /// Gather counts for the status dashboard.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if any count query fails.
    pub async fn run(&self) -> Result<StatusReport, TankyuError> {
        let topics = self.stats.count_topics().await?;
        let sources = self.stats.count_sources().await?;
        let entries = self.stats.count_entries().await?;

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
        let store: Arc<dyn CountStats> = Arc::new(NanographStore::open_in_memory().await.unwrap());
        let uc = StatusUseCase::new(store);
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
{"type": "Entry", "data": {"slug": "tokio-intro", "sourceSlug": "tokio", "entryType": "article", "title": "Intro to Tokio", "url": "https://tokio.rs/intro", "state": "active"}}
"#,
            )
            .await
            .unwrap();

        let stats: Arc<dyn CountStats> = Arc::new(store);
        let uc = StatusUseCase::new(stats);
        let report = uc.run().await.unwrap();
        assert_eq!(report.topics, 1);
        assert_eq!(report.sources, 1);
        assert_eq!(report.entries, 1);
    }
}
