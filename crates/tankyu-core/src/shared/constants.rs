use std::path::{Path, PathBuf};

/// Returns the tankyu data directory.
/// Reads `TANKYU_DIR` env var; falls back to `~/.tankyu`.
#[must_use]
pub fn tankyu_dir() -> PathBuf {
    tankyu_dir_from_env(std::env::var("TANKYU_DIR").ok().as_deref())
}

/// Testable core of `tankyu_dir`. Pass `Some(path)` to override, `None` for default.
///
/// # Panics
///
/// Panics if `override_path` is `None` and the home directory cannot be determined.
#[must_use]
pub fn tankyu_dir_from_env(override_path: Option<&str>) -> PathBuf {
    override_path.map_or_else(
        || {
            dirs::home_dir()
                .expect("could not determine home directory")
                .join(".tankyu")
        },
        PathBuf::from,
    )
}

#[must_use]
pub fn topics_dir(base: &Path) -> PathBuf {
    base.join("topics")
}
#[must_use]
pub fn sources_dir(base: &Path) -> PathBuf {
    base.join("sources")
}
#[must_use]
pub fn entries_dir(base: &Path) -> PathBuf {
    base.join("entries")
}
#[must_use]
pub fn insights_dir(base: &Path) -> PathBuf {
    base.join("insights")
}
#[must_use]
pub fn entities_dir(base: &Path) -> PathBuf {
    base.join("entities")
}
#[must_use]
pub fn graph_dir(base: &Path) -> PathBuf {
    base.join("graph")
}
#[must_use]
pub fn config_path(base: &Path) -> PathBuf {
    base.join("config.json")
}
#[must_use]
pub fn edges_path(base: &Path) -> PathBuf {
    graph_dir(base).join("edges.json")
}

#[must_use]
pub fn db_path(base: &Path) -> PathBuf {
    base.join("db")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tankyu_dir_uses_override() {
        let dir = tankyu_dir_from_env(Some("/tmp/test-tankyu"));
        assert_eq!(dir, PathBuf::from("/tmp/test-tankyu"));
    }

    #[test]
    fn tankyu_dir_falls_back_to_home() {
        let dir = tankyu_dir_from_env(None);
        assert_eq!(dir, dirs::home_dir().unwrap().join(".tankyu"));
    }

    #[test]
    fn sub_paths_derive_from_base() {
        let base = PathBuf::from("/tmp/test");
        assert_eq!(topics_dir(&base), base.join("topics"));
        assert_eq!(sources_dir(&base), base.join("sources"));
        assert_eq!(entries_dir(&base), base.join("entries"));
        assert_eq!(insights_dir(&base), base.join("insights"));
        assert_eq!(entities_dir(&base), base.join("entities"));
        assert_eq!(graph_dir(&base), base.join("graph"));
        assert_eq!(config_path(&base), base.join("config.json"));
        assert_eq!(edges_path(&base), base.join("graph").join("edges.json"));
    }
}
