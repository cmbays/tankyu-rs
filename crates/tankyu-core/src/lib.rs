#![forbid(unsafe_code)]
#![recursion_limit = "512"]

pub mod domain;
pub mod features;
pub mod infrastructure;
pub mod shared;

pub use features::doctor::GraphDoctor;
pub use features::health::{
    HealthManager, HealthReport, HealthThresholds, HealthWarning, HealthWarningKind,
};
pub use features::status::CountStats;
pub use infrastructure::nanograph::NanographStore;
pub use infrastructure::stores::JsonCountStats;
