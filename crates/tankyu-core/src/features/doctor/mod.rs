use std::sync::Arc;

use async_trait::async_trait;

use crate::shared::error::TankyuError;

/// Database health report surfaced by `GraphDoctor::check_health()`.
#[derive(Debug, Clone, Default)]
pub struct DoctorReport {
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub datasets_checked: usize,
}

impl DoctorReport {
    /// Returns `true` when no issues were found.
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Port trait for graph database health diagnostics.
#[async_trait]
pub trait GraphDoctor: Send + Sync {
    /// Run database diagnostics and return a health report.
    async fn check_health(&self) -> Result<DoctorReport, TankyuError>;
}

pub struct DoctorUseCase {
    doctor: Arc<dyn GraphDoctor>,
}

impl DoctorUseCase {
    #[must_use]
    pub fn new(doctor: Arc<dyn GraphDoctor>) -> Self {
        Self { doctor }
    }

    /// Run database diagnostics.
    ///
    /// # Errors
    /// Returns `TankyuError::Store` if the doctor check fails.
    pub async fn run(&self) -> Result<DoctorReport, TankyuError> {
        self.doctor.check_health().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::nanograph::NanographStore;

    #[tokio::test]
    async fn doctor_returns_healthy_on_good_db() {
        let doctor: Arc<dyn GraphDoctor> =
            Arc::new(NanographStore::open_in_memory().await.unwrap());
        let uc = DoctorUseCase::new(doctor);
        let report = uc.run().await.unwrap();
        assert!(report.is_healthy());
        assert!(report.issues.is_empty());
    }
}
