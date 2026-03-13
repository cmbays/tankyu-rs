use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// ── Source enums ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceState {
    Active,
    Stale,
    Dormant,
    Pruned,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceType {
    XAccount,
    XBookmarks,
    GithubRepo,
    GithubReleases,
    GithubUser,
    Blog,
    RssFeed,
    WebPage,
    Manual,
    GithubIssues,
    AgentReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceRole {
    Starred,
    RoleModel,
    Reference,
}

// ── Entry enums ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntryState {
    New,
    Scanned,
    Triaged,
    Read,
    Archived,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntryType {
    Tweet,
    Commit,
    Pr,
    Release,
    Article,
    Page,
    Repo,
    GithubIssue,
    SpikeReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Signal {
    High,
    Medium,
    Low,
    Noise,
}

// ── Graph enums ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EdgeType {
    Monitors,
    Produced,
    Yields,
    RelatesTo,
    Supersedes,
    Contradicts,
    Informs,
    DiscoveredVia,
    InspiredBy,
    TaggedWith,
    Mentions,
    Cites,
    CoOccursWith,
    DerivedFrom,
    Synthesizes,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeType {
    Topic,
    Source,
    Entry,
    Insight,
    Project,
    Entity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassificationMethod {
    SourceRule,
    Keyword,
    Llm,
    Manual,
}

// ── Insight / Entity enums ────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InsightType {
    ResearchNote,
    Synthesis,
    Briefing,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntityType {
    Person,
    Organization,
    Technology,
    Concept,
    Product,
    Event,
}

// ── Structs ───────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopicRouting {
    pub keywords: Vec<String>,
    pub min_score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Topic {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub projects: Vec<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing: Option<TopicRouting>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_scanned_at: Option<DateTime<Utc>>,
    pub scan_count: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: Uuid,
    pub r#type: SourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<SourceRole>,
    pub name: String,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<HashMap<String, String>>,
    pub state: SourceState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll_interval_minutes: Option<u32>,
    // Nullable fields: always present in JSON (null when absent), use #[serde(default)]
    #[serde(default)]
    pub discovered_via: Option<Uuid>,
    #[serde(default)]
    pub discovery_reason: Option<String>,
    #[serde(default)]
    pub last_checked_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub last_new_content_at: Option<DateTime<Utc>>,
    pub check_count: u32,
    pub hit_count: u32,
    pub miss_count: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: Uuid,
    pub source_id: Uuid,
    pub r#type: EntryType,
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub content_hash: Option<String>,
    pub state: EntryState,
    #[serde(default)]
    pub signal: Option<Signal>,
    pub scanned_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Edge {
    pub id: Uuid,
    pub from_id: Uuid,
    pub from_type: NodeType,
    pub to_id: Uuid,
    pub to_type: NodeType,
    pub edge_type: EdgeType,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<ClassificationMethod>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphIndex {
    pub version: u32,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TankyuConfig {
    pub version: u32,
    pub default_scan_limit: u32,
    pub stale_days: u32,
    pub dormant_days: u32,
    pub llm_classify: bool,
    pub local_repo_paths: HashMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registry_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Insight {
    pub id: Uuid,
    pub r#type: InsightType,
    pub title: String,
    pub body: String,
    pub key_points: Vec<String>,
    pub citations: Vec<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: Uuid,
    pub r#type: EntityType,
    pub name: String,
    pub aliases: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ── Update types (partial structs for store updates) ──────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct TopicUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub routing: Option<TopicRouting>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_scanned_at: Option<DateTime<Utc>>,
    pub scan_count: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct SourceUpdate {
    pub role: Option<SourceRole>,
    pub state: Option<SourceState>,
    pub poll_interval_minutes: Option<u32>,
    pub last_checked_at: Option<DateTime<Utc>>,
    pub last_new_content_at: Option<DateTime<Utc>>,
    pub check_count: Option<u32>,
    pub hit_count: Option<u32>,
    pub miss_count: Option<u32>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct EntryUpdate {
    pub state: Option<EntryState>,
    pub signal: Option<Signal>,
    pub summary: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct InsightUpdate {
    pub title: Option<String>,
    pub body: Option<String>,
    pub key_points: Option<Vec<String>>,
    pub citations: Option<Vec<Uuid>>,
    pub updated_at: Option<DateTime<Utc>>,
}

// ── Graph query ───────────────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct GraphQuery {
    pub from_type: Option<NodeType>,
    pub to_type: Option<NodeType>,
    pub edge_type: Option<EdgeType>,
    pub from_id: Option<Uuid>,
    pub to_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_state_roundtrips() {
        let cases = [
            (SourceState::Active, "\"active\""),
            (SourceState::Stale, "\"stale\""),
            (SourceState::Dormant, "\"dormant\""),
            (SourceState::Pruned, "\"pruned\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: SourceState = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn source_type_roundtrips() {
        let cases = [
            (SourceType::XAccount, "\"x-account\""),
            (SourceType::XBookmarks, "\"x-bookmarks\""),
            (SourceType::GithubRepo, "\"github-repo\""),
            (SourceType::GithubReleases, "\"github-releases\""),
            (SourceType::GithubUser, "\"github-user\""),
            (SourceType::Blog, "\"blog\""),
            (SourceType::RssFeed, "\"rss-feed\""),
            (SourceType::WebPage, "\"web-page\""),
            (SourceType::Manual, "\"manual\""),
            (SourceType::GithubIssues, "\"github-issues\""),
            (SourceType::AgentReport, "\"agent-report\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: SourceType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn source_role_roundtrips() {
        let cases = [
            (SourceRole::Starred, "\"starred\""),
            (SourceRole::RoleModel, "\"role-model\""),
            (SourceRole::Reference, "\"reference\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: SourceRole = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn entry_state_roundtrips() {
        let cases = [
            (EntryState::New, "\"new\""),
            (EntryState::Scanned, "\"scanned\""),
            (EntryState::Triaged, "\"triaged\""),
            (EntryState::Read, "\"read\""),
            (EntryState::Archived, "\"archived\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: EntryState = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn entry_type_roundtrips() {
        let cases = [
            (EntryType::Tweet, "\"tweet\""),
            (EntryType::Commit, "\"commit\""),
            (EntryType::Pr, "\"pr\""),
            (EntryType::Release, "\"release\""),
            (EntryType::Article, "\"article\""),
            (EntryType::Page, "\"page\""),
            (EntryType::Repo, "\"repo\""),
            (EntryType::GithubIssue, "\"github-issue\""),
            (EntryType::SpikeReport, "\"spike-report\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: EntryType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn signal_roundtrips() {
        let cases = [
            (Signal::High, "\"high\""),
            (Signal::Medium, "\"medium\""),
            (Signal::Low, "\"low\""),
            (Signal::Noise, "\"noise\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: Signal = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn edge_type_roundtrips() {
        let cases = [
            (EdgeType::Monitors, "\"monitors\""),
            (EdgeType::Produced, "\"produced\""),
            (EdgeType::Yields, "\"yields\""),
            (EdgeType::RelatesTo, "\"relates-to\""),
            (EdgeType::Supersedes, "\"supersedes\""),
            (EdgeType::Contradicts, "\"contradicts\""),
            (EdgeType::Informs, "\"informs\""),
            (EdgeType::DiscoveredVia, "\"discovered-via\""),
            (EdgeType::InspiredBy, "\"inspired-by\""),
            (EdgeType::TaggedWith, "\"tagged-with\""),
            (EdgeType::Mentions, "\"mentions\""),
            (EdgeType::Cites, "\"cites\""),
            (EdgeType::CoOccursWith, "\"co-occurs-with\""),
            (EdgeType::DerivedFrom, "\"derived-from\""),
            (EdgeType::Synthesizes, "\"synthesizes\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: EdgeType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn node_type_roundtrips() {
        let cases = [
            (NodeType::Topic, "\"topic\""),
            (NodeType::Source, "\"source\""),
            (NodeType::Entry, "\"entry\""),
            (NodeType::Insight, "\"insight\""),
            (NodeType::Project, "\"project\""),
            (NodeType::Entity, "\"entity\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: NodeType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn insight_type_roundtrips() {
        let cases = [
            (InsightType::ResearchNote, "\"research-note\""),
            (InsightType::Synthesis, "\"synthesis\""),
            (InsightType::Briefing, "\"briefing\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: InsightType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn entity_type_roundtrips() {
        let cases = [
            (EntityType::Person, "\"person\""),
            (EntityType::Organization, "\"organization\""),
            (EntityType::Technology, "\"technology\""),
            (EntityType::Concept, "\"concept\""),
            (EntityType::Product, "\"product\""),
            (EntityType::Event, "\"event\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: EntityType = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }

    #[test]
    fn classification_method_roundtrips() {
        let cases = [
            (ClassificationMethod::SourceRule, "\"source-rule\""),
            (ClassificationMethod::Keyword, "\"keyword\""),
            (ClassificationMethod::Llm, "\"llm\""),
            (ClassificationMethod::Manual, "\"manual\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: ClassificationMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }
}
