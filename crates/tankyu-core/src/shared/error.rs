// Io and Json are boxed to keep the enum's stack size small.
#[derive(Debug, thiserror::Error)]
pub enum TankyuError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("store error: {0}")]
    Store(String),

    #[error("scan error: {0}")]
    Scan(String),

    #[error("duplicate {kind}: '{name}' already exists")]
    Duplicate { kind: String, name: String },

    #[error("io error: {0}")]
    Io(Box<std::io::Error>),

    #[error("json error: {0}")]
    Json(Box<serde_json::Error>),
}

impl From<std::io::Error> for TankyuError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(Box::new(e))
    }
}

impl From<serde_json::Error> for TankyuError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(Box::new(e))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_displays_message() {
        let err = TankyuError::NotFound("topics/abc.json".to_string());
        assert_eq!(err.to_string(), "not found: topics/abc.json");
    }

    #[test]
    fn io_error_converts_via_from() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
        let err: TankyuError = io_err.into();
        assert!(matches!(err, TankyuError::Io(_)));
    }

    #[test]
    fn json_error_converts_via_from() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err: TankyuError = json_err.into();
        assert!(matches!(err, TankyuError::Json(_)));
    }

    #[test]
    fn duplicate_displays_message() {
        let err = TankyuError::Duplicate {
            kind: "topic".to_string(),
            name: "Rust".to_string(),
        };
        assert_eq!(err.to_string(), "duplicate topic: 'Rust' already exists");
    }
}
