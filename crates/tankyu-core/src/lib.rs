#![forbid(unsafe_code)]

pub mod domain;
pub mod features;
pub mod infrastructure;
pub mod shared;

pub use features::health::{
    HealthManager, HealthReport, HealthThresholds, HealthWarning, HealthWarningKind,
};
