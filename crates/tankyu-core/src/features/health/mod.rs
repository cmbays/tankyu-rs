use std::sync::Arc;

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    ports::{IEntryStore, ISourceStore},
    types::{SourceState, SourceType},
};

/// Thresholds for health checks, derived from `TankyuConfig`.
/// Config never enters `tankyu-core` — the CLI layer constructs this.
pub struct HealthThresholds {
    pub stale_days: u32,
    pub dormant_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HealthWarningKind {
    Stale,
    Dormant,
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthWarning {
    pub source_id: Uuid,
    pub source_name: String,
    pub source_type: SourceType,
    pub kind: HealthWarningKind,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthReport {
    pub ok: bool,
    pub warnings: Vec<HealthWarning>,
    pub checked_at: DateTime<Utc>,
}

/// Checks source health against config-driven thresholds.
pub struct HealthManager {
    source_store: Arc<dyn ISourceStore>,
    entry_store: Arc<dyn IEntryStore>,
}

impl HealthManager {
    #[must_use]
    pub fn new(source_store: Arc<dyn ISourceStore>, entry_store: Arc<dyn IEntryStore>) -> Self {
        Self {
            source_store,
            entry_store,
        }
    }

    /// Run health checks and return a report.
    ///
    /// # Errors
    /// Returns an error if any store read fails.
    pub async fn health(&self, thresholds: HealthThresholds) -> Result<HealthReport> {
        let sources = self.source_store.list().await?;
        let entries = self.entry_store.list().await?;

        // Build set of source IDs that have at least one entry
        let sources_with_entries: std::collections::HashSet<Uuid> =
            entries.iter().map(|e| e.source_id).collect();

        let now = Utc::now();
        let mut warnings = Vec::new();

        for source in &sources {
            if source.state == SourceState::Pruned {
                continue;
            }

            // Staleness check
            match source.last_checked_at {
                None => {
                    // Never checked → always Stale
                    warnings.push(HealthWarning {
                        source_id: source.id,
                        source_name: source.name.clone(),
                        source_type: source.r#type.clone(),
                        kind: HealthWarningKind::Stale,
                        detail: "never checked".to_string(),
                    });
                }
                Some(last_checked) => {
                    let age_days = u32::try_from((now - last_checked).num_days()).unwrap_or(0);
                    if age_days > thresholds.dormant_days {
                        warnings.push(HealthWarning {
                            source_id: source.id,
                            source_name: source.name.clone(),
                            source_type: source.r#type.clone(),
                            kind: HealthWarningKind::Dormant,
                            detail: format!("last checked {age_days} days ago"),
                        });
                    } else if age_days > thresholds.stale_days {
                        warnings.push(HealthWarning {
                            source_id: source.id,
                            source_name: source.name.clone(),
                            source_type: source.r#type.clone(),
                            kind: HealthWarningKind::Stale,
                            detail: format!("last checked {age_days} days ago"),
                        });
                    }
                }
            }

            // Empty check (independent of staleness)
            if !sources_with_entries.contains(&source.id) {
                warnings.push(HealthWarning {
                    source_id: source.id,
                    source_name: source.name.clone(),
                    source_type: source.r#type.clone(),
                    kind: HealthWarningKind::Empty,
                    detail: "no entries ingested from this source".to_string(),
                });
            }
        }

        Ok(HealthReport {
            ok: warnings.is_empty(),
            warnings,
            checked_at: now,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports::{IEntryStore, ISourceStore},
        types::{
            Entry, EntryState, EntryType, EntryUpdate, Source, SourceState, SourceType,
            SourceUpdate,
        },
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use uuid::Uuid;

    struct StubSourceStore {
        sources: Vec<Source>,
    }
    struct StubEntryStore {
        entries: Vec<Entry>,
    }

    #[async_trait]
    impl ISourceStore for StubSourceStore {
        async fn create(&self, _s: Source) -> Result<()> {
            Ok(())
        }
        async fn get(&self, id: Uuid) -> Result<Option<Source>> {
            Ok(self.sources.iter().find(|s| s.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Source>> {
            Ok(None)
        }
        async fn list(&self) -> Result<Vec<Source>> {
            Ok(self.sources.clone())
        }
        async fn update(&self, _id: Uuid, _u: SourceUpdate) -> Result<Source> {
            unimplemented!()
        }
    }

    #[async_trait]
    impl IEntryStore for StubEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> {
            Ok(())
        }
        async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
            Ok(self.entries.iter().find(|e| e.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Entry>> {
            Ok(None)
        }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> {
            Ok(None)
        }
        async fn list_by_source(&self, sid: Uuid) -> Result<Vec<Entry>> {
            Ok(self
                .entries
                .iter()
                .filter(|e| e.source_id == sid)
                .cloned()
                .collect())
        }
        async fn list(&self) -> Result<Vec<Entry>> {
            Ok(self.entries.clone())
        }
        async fn update(&self, _id: Uuid, _u: EntryUpdate) -> Result<Entry> {
            unimplemented!()
        }
    }

    fn make_source(name: &str, last_checked_days_ago: Option<i64>) -> Source {
        Source {
            id: Uuid::new_v4(),
            r#type: SourceType::GithubRepo,
            role: None,
            name: name.to_string(),
            url: format!("https://example.com/{name}"),
            config: None,
            state: SourceState::Active,
            poll_interval_minutes: None,
            discovered_via: None,
            discovery_reason: None,
            last_checked_at: last_checked_days_ago.map(|d| Utc::now() - Duration::days(d)),
            last_new_content_at: None,
            check_count: 0,
            hit_count: 0,
            miss_count: 0,
            created_at: Utc::now(),
        }
    }

    fn make_entry(source_id: Uuid) -> Entry {
        Entry {
            id: Uuid::new_v4(),
            source_id,
            r#type: EntryType::Article,
            title: "test".to_string(),
            url: "https://example.com".to_string(),
            summary: None,
            content_hash: None,
            state: EntryState::New,
            signal: None,
            scanned_at: Utc::now(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    fn thresholds() -> HealthThresholds {
        HealthThresholds {
            stale_days: 7,
            dormant_days: 30,
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_recently_checked_source_produces_no_staleness_warning() {
        let source = make_source("fresh", Some(1)); // checked 1 day ago
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(report.ok);
        assert!(report.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_source_checked_beyond_stale_threshold_produces_stale_warning() {
        let source = make_source("stale", Some(10)); // 10 days > stale_days=7
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings.len(), 1);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Stale);
    }

    #[tokio::test]
    async fn test_source_checked_beyond_dormant_threshold_produces_dormant_not_stale() {
        // Mutation killer: dormant_days=30, stale_days=7 — 35 days must produce Dormant, not Stale
        let source = make_source("dormant", Some(35));
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Dormant);
        // Must NOT be Stale
        assert!(!report
            .warnings
            .iter()
            .any(|w| w.kind == HealthWarningKind::Stale));
    }

    #[tokio::test]
    async fn test_never_checked_source_produces_stale_warning() {
        // NULL last_checked_at → Stale (not Dormant)
        let source = make_source("never-checked", None);
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Stale);
        assert!(report.warnings[0].detail.contains("never checked"));
    }

    #[tokio::test]
    async fn test_source_with_no_entries_produces_empty_warning() {
        let source = make_source("empty", Some(1)); // fresh but no entries
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore { entries: vec![] }); // no entries
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Empty);
    }

    #[tokio::test]
    async fn test_source_can_have_both_stale_and_empty_warnings() {
        let source = make_source("stale-and-empty", Some(10)); // stale + no entries
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore { entries: vec![] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert_eq!(report.warnings.len(), 2);
    }

    #[tokio::test]
    async fn test_pruned_source_is_skipped() {
        // Mutation killer: != Pruned skip logic
        let mut source = make_source("pruned", None);
        source.state = SourceState::Pruned;
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore { entries: vec![] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(report.ok);
        assert!(report.warnings.is_empty());
    }

    #[tokio::test]
    async fn test_ok_true_when_no_warnings() {
        let source = make_source("healthy", Some(1));
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(report.ok);
    }

    #[tokio::test]
    async fn test_ok_false_when_any_warning() {
        let source = make_source("unhealthy", None);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore { entries: vec![] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
    }

    #[tokio::test]
    async fn test_source_at_exact_stale_boundary_is_not_stale() {
        // Mutation killer: age_days == stale_days should NOT produce stale warning
        let source = make_source("boundary-stale", Some(7)); // exactly stale_days
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(
            report.ok,
            "source at exact stale boundary should be healthy"
        );
    }

    #[tokio::test]
    async fn test_source_at_exact_dormant_boundary_is_not_dormant() {
        // Mutation killer: age_days == dormant_days should NOT produce dormant warning
        let source = make_source("boundary-dormant", Some(30)); // exactly dormant_days
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore {
            sources: vec![source],
        });
        let entries = Arc::new(StubEntryStore {
            entries: vec![entry],
        });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        // At exactly dormant_days, should not be Dormant
        assert!(
            !report
                .warnings
                .iter()
                .any(|w| w.kind == HealthWarningKind::Dormant),
            "source at exact dormant boundary should not produce Dormant warning"
        );
    }
}
