use std::sync::Arc;

use crate::domain::research_graph::{IResearchGraph, ParamMap};
use crate::shared::error::TankyuError;

const STATUS_QUERIES: &str = include_str!("../../infrastructure/nanograph/queries/status.gq");

#[derive(Debug, Clone, Default)]
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
