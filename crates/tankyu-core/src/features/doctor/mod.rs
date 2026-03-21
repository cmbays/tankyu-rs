use std::sync::Arc;

use crate::domain::research_graph::{DoctorReport, IResearchGraph};
use crate::shared::error::TankyuError;

pub struct DoctorUseCase {
    graph: Arc<dyn IResearchGraph>,
}

impl DoctorUseCase {
    #[must_use]
    pub fn new(graph: Arc<dyn IResearchGraph>) -> Self {
        Self { graph }
    }

    /// Run database diagnostics.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the doctor check fails.
    pub async fn run(&self) -> Result<DoctorReport, TankyuError> {
        self.graph.doctor().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::nanograph::NanographStore;

    #[tokio::test]
    async fn doctor_returns_healthy_on_good_db() {
        let graph: Arc<dyn IResearchGraph> =
            Arc::new(NanographStore::open_in_memory().await.unwrap());
        let uc = DoctorUseCase::new(graph);
        let report = uc.run().await.unwrap();
        assert!(report.healthy);
        assert!(report.issues.is_empty());
    }
}
