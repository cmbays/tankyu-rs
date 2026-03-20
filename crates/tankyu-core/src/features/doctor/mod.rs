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
