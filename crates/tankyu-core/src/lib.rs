#![forbid(unsafe_code)]
#![recursion_limit = "512"]

pub mod domain;
pub mod features;
pub mod infrastructure;
pub mod shared;

pub use domain::research_graph::IResearchGraph;
pub use features::health::{
    HealthManager, HealthReport, HealthThresholds, HealthWarning, HealthWarningKind,
};
pub use infrastructure::nanograph::NanographStore;
