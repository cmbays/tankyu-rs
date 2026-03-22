use std::sync::Arc;

use async_trait::async_trait;

use crate::domain::ports::{IEntryStore, ISourceStore, ITopicStore};
use crate::features::status::CountStats;
use crate::shared::error::TankyuError;

/// [`CountStats`] implementation that counts records from JSON file stores.
pub struct JsonCountStats {
    topics: Arc<dyn ITopicStore>,
    sources: Arc<dyn ISourceStore>,
    entries: Arc<dyn IEntryStore>,
}

impl JsonCountStats {
    #[must_use]
    pub fn new(
        topics: Arc<dyn ITopicStore>,
        sources: Arc<dyn ISourceStore>,
        entries: Arc<dyn IEntryStore>,
    ) -> Self {
        Self {
            topics,
            sources,
            entries,
        }
    }
}

#[async_trait]
impl CountStats for JsonCountStats {
    async fn count_topics(&self) -> Result<usize, TankyuError> {
        self.topics
            .list()
            .await
            .map(|v| v.len())
            .map_err(|e| TankyuError::Store(e.to_string()))
    }

    async fn count_sources(&self) -> Result<usize, TankyuError> {
        self.sources
            .list()
            .await
            .map(|v| v.len())
            .map_err(|e| TankyuError::Store(e.to_string()))
    }

    async fn count_entries(&self) -> Result<usize, TankyuError> {
        self.entries
            .list()
            .await
            .map(|v| v.len())
            .map_err(|e| TankyuError::Store(e.to_string()))
    }
}
