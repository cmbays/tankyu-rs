#![allow(dead_code)] // structs added in Task 3

use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_state_roundtrips() {
        let cases = [
            (SourceState::Active,  "\"active\""),
            (SourceState::Stale,   "\"stale\""),
            (SourceState::Dormant, "\"dormant\""),
            (SourceState::Pruned,  "\"pruned\""),
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
            (SourceType::XAccount,       "\"x-account\""),
            (SourceType::XBookmarks,     "\"x-bookmarks\""),
            (SourceType::GithubRepo,     "\"github-repo\""),
            (SourceType::GithubReleases, "\"github-releases\""),
            (SourceType::GithubUser,     "\"github-user\""),
            (SourceType::Blog,           "\"blog\""),
            (SourceType::RssFeed,        "\"rss-feed\""),
            (SourceType::WebPage,        "\"web-page\""),
            (SourceType::Manual,         "\"manual\""),
            (SourceType::GithubIssues,   "\"github-issues\""),
            (SourceType::AgentReport,    "\"agent-report\""),
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
            (SourceRole::Starred,   "\"starred\""),
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
            (EntryState::New,      "\"new\""),
            (EntryState::Scanned,  "\"scanned\""),
            (EntryState::Triaged,  "\"triaged\""),
            (EntryState::Read,     "\"read\""),
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
            (EntryType::Tweet,       "\"tweet\""),
            (EntryType::Commit,      "\"commit\""),
            (EntryType::Pr,          "\"pr\""),
            (EntryType::Release,     "\"release\""),
            (EntryType::Article,     "\"article\""),
            (EntryType::Page,        "\"page\""),
            (EntryType::Repo,        "\"repo\""),
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
            (Signal::High,   "\"high\""),
            (Signal::Medium, "\"medium\""),
            (Signal::Low,    "\"low\""),
            (Signal::Noise,  "\"noise\""),
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
            (EdgeType::Monitors,      "\"monitors\""),
            (EdgeType::Produced,      "\"produced\""),
            (EdgeType::Yields,        "\"yields\""),
            (EdgeType::RelatesTo,     "\"relates-to\""),
            (EdgeType::Supersedes,    "\"supersedes\""),
            (EdgeType::Contradicts,   "\"contradicts\""),
            (EdgeType::Informs,       "\"informs\""),
            (EdgeType::DiscoveredVia, "\"discovered-via\""),
            (EdgeType::InspiredBy,    "\"inspired-by\""),
            (EdgeType::TaggedWith,    "\"tagged-with\""),
            (EdgeType::Mentions,      "\"mentions\""),
            (EdgeType::Cites,         "\"cites\""),
            (EdgeType::CoOccursWith,  "\"co-occurs-with\""),
            (EdgeType::DerivedFrom,   "\"derived-from\""),
            (EdgeType::Synthesizes,   "\"synthesizes\""),
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
            (NodeType::Topic,   "\"topic\""),
            (NodeType::Source,  "\"source\""),
            (NodeType::Entry,   "\"entry\""),
            (NodeType::Insight, "\"insight\""),
            (NodeType::Project, "\"project\""),
            (NodeType::Entity,  "\"entity\""),
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
            (InsightType::Synthesis,    "\"synthesis\""),
            (InsightType::Briefing,     "\"briefing\""),
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
            (EntityType::Person,       "\"person\""),
            (EntityType::Organization, "\"organization\""),
            (EntityType::Technology,   "\"technology\""),
            (EntityType::Concept,      "\"concept\""),
            (EntityType::Product,      "\"product\""),
            (EntityType::Event,        "\"event\""),
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
            (ClassificationMethod::Keyword,    "\"keyword\""),
            (ClassificationMethod::Llm,        "\"llm\""),
            (ClassificationMethod::Manual,     "\"manual\""),
        ];
        for (variant, expected_json) in cases {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, expected_json, "serialize {variant:?}");
            let restored: ClassificationMethod = serde_json::from_str(&json).unwrap();
            assert_eq!(restored, variant, "deserialize {variant:?}");
        }
    }
}
