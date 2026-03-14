use tankyu_core::shared::error::TankyuError;

#[allow(dead_code)]
pub const fn error_hint(err: &TankyuError) -> Option<&'static str> {
    match err {
        TankyuError::NotFound(_) => Some("Run `tankyu init` to initialize the data directory"),
        TankyuError::Config(_) => Some("Run `tankyu doctor` to diagnose configuration issues"),
        _ => None,
    }
}
