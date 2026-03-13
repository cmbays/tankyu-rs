use chrono::Utc;
use proptest::prelude::*;
use tankyu_core::domain::types::*;
use uuid::Uuid;

// Helper strategies
fn source_state_strategy() -> impl Strategy<Value = SourceState> {
    prop_oneof![
        Just(SourceState::Active),
        Just(SourceState::Stale),
        Just(SourceState::Dormant),
        Just(SourceState::Pruned),
    ]
}

fn source_type_strategy() -> impl Strategy<Value = SourceType> {
    prop_oneof![
        Just(SourceType::XAccount),
        Just(SourceType::XBookmarks),
        Just(SourceType::GithubRepo),
        Just(SourceType::GithubReleases),
        Just(SourceType::GithubUser),
        Just(SourceType::Blog),
        Just(SourceType::RssFeed),
        Just(SourceType::WebPage),
        Just(SourceType::Manual),
        Just(SourceType::GithubIssues),
        Just(SourceType::AgentReport),
    ]
}

fn entry_type_strategy() -> impl Strategy<Value = EntryType> {
    prop_oneof![
        Just(EntryType::Tweet),
        Just(EntryType::Commit),
        Just(EntryType::Pr),
        Just(EntryType::Release),
        Just(EntryType::Article),
        Just(EntryType::Page),
        Just(EntryType::Repo),
        Just(EntryType::GithubIssue),
        Just(EntryType::SpikeReport),
    ]
}

fn edge_type_strategy() -> impl Strategy<Value = EdgeType> {
    prop_oneof![
        Just(EdgeType::Monitors),
        Just(EdgeType::Produced),
        Just(EdgeType::Yields),
        Just(EdgeType::RelatesTo),
        Just(EdgeType::Supersedes),
        Just(EdgeType::Contradicts),
        Just(EdgeType::Informs),
        Just(EdgeType::DiscoveredVia),
        Just(EdgeType::InspiredBy),
        Just(EdgeType::TaggedWith),
        Just(EdgeType::Mentions),
        Just(EdgeType::Cites),
        Just(EdgeType::CoOccursWith),
        Just(EdgeType::DerivedFrom),
        Just(EdgeType::Synthesizes),
    ]
}

fn node_type_strategy() -> impl Strategy<Value = NodeType> {
    prop_oneof![
        Just(NodeType::Topic),
        Just(NodeType::Source),
        Just(NodeType::Entry),
        Just(NodeType::Insight),
        Just(NodeType::Project),
        Just(NodeType::Entity),
    ]
}

proptest! {
    #[test]
    fn topic_roundtrip(
        name in "[a-zA-Z][a-zA-Z0-9 ]{0,50}",
        description in "[a-zA-Z0-9 ]{0,200}",
    ) {
        let topic = Topic {
            id: Uuid::new_v4(),
            name,
            description,
            tags: vec![],
            projects: vec![],
            routing: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_scanned_at: None,
            scan_count: 0,
        };
        let json = serde_json::to_string(&topic).unwrap();
        let restored: Topic = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(topic.id, restored.id);
        prop_assert_eq!(topic.name, restored.name);
        prop_assert_eq!(topic.description, restored.description);
    }

    #[test]
    fn source_roundtrip(
        state in source_state_strategy(),
        src_type in source_type_strategy(),
        name in "[a-zA-Z][a-zA-Z0-9 ]{0,50}",
        check_count in 0u32..1000u32,
        hit_count in 0u32..1000u32,
        miss_count in 0u32..1000u32,
    ) {
        let source = Source {
            id: Uuid::new_v4(),
            r#type: src_type,
            role: None,
            name,
            url: "https://example.com".to_string(),
            config: None,
            state,
            poll_interval_minutes: None,
            discovered_via: None,
            discovery_reason: None,
            last_checked_at: None,
            last_new_content_at: None,
            check_count,
            hit_count,
            miss_count,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&source).unwrap();
        let restored: Source = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(source.id, restored.id);
        prop_assert_eq!(source.check_count, restored.check_count);
        prop_assert_eq!(source.hit_count, restored.hit_count);
        prop_assert_eq!(source.miss_count, restored.miss_count);
    }

    #[test]
    fn entry_roundtrip(
        entry_type in entry_type_strategy(),
        title in "[a-zA-Z][a-zA-Z0-9 ]{0,100}",
    ) {
        let entry = Entry {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            r#type: entry_type,
            title,
            url: "https://example.com".to_string(),
            summary: None,
            content_hash: None,
            state: EntryState::New,
            signal: None,
            scanned_at: Utc::now(),
            metadata: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: Entry = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(entry.id, restored.id);
        prop_assert_eq!(entry.title, restored.title);
    }

    #[test]
    fn edge_roundtrip(
        edge_type in edge_type_strategy(),
        from_type in node_type_strategy(),
        to_type in node_type_strategy(),
        reason in "[a-zA-Z0-9 ]{1,100}",
    ) {
        let edge = Edge {
            id: Uuid::new_v4(),
            from_id: Uuid::new_v4(),
            from_type,
            to_id: Uuid::new_v4(),
            to_type,
            edge_type,
            reason,
            score: None,
            method: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&edge).unwrap();
        let restored: Edge = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(edge.id, restored.id);
        prop_assert_eq!(edge.reason, restored.reason);
    }

    #[test]
    fn graph_index_roundtrip(n_edges in 0usize..10usize) {
        let edges: Vec<Edge> = (0..n_edges).map(|_| Edge {
            id: Uuid::new_v4(),
            from_id: Uuid::new_v4(),
            from_type: NodeType::Topic,
            to_id: Uuid::new_v4(),
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        }).collect();
        let index = GraphIndex { version: 1, edges };
        let json = serde_json::to_string(&index).unwrap();
        let restored: GraphIndex = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(index.version, restored.version);
        prop_assert_eq!(index.edges.len(), restored.edges.len());
    }

    #[test]
    fn config_roundtrip(
        stale_days in 1u32..30u32,
        dormant_days in 30u32..180u32,
    ) {
        let config = TankyuConfig {
            version: 1,
            default_scan_limit: 20,
            stale_days,
            dormant_days,
            llm_classify: false,
            local_repo_paths: std::collections::HashMap::new(),
            registry_path: None,
        };
        let json = serde_json::to_string(&config).unwrap();
        let restored: TankyuConfig = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(config.version, restored.version);
        prop_assert_eq!(config.stale_days, restored.stale_days);
        prop_assert_eq!(config.dormant_days, restored.dormant_days);
    }

    #[test]
    fn source_nullable_fields_roundtrip(
        check_count in 0u32..100u32,
    ) {
        // Nullable fields (.nullable() in TS) must always be present in JSON
        // as null when None — they must NOT use skip_serializing_if.
        // serde serializes Option::None as null by default (without skip_serializing_if).
        let source = Source {
            id: Uuid::new_v4(),
            r#type: SourceType::GithubRepo,
            role: None,
            name: "test".to_string(),
            url: "https://github.com/test/repo".to_string(),
            config: None,
            state: SourceState::Active,
            poll_interval_minutes: None,
            discovered_via: None,
            discovery_reason: None,
            last_checked_at: None,
            last_new_content_at: None,
            check_count,
            hit_count: 0,
            miss_count: 0,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&source).unwrap();
        // Nullable fields must be present in JSON as null (not absent)
        prop_assert!(json.contains("\"discoveredVia\":null"));
        prop_assert!(json.contains("\"discoveryReason\":null"));
        prop_assert!(json.contains("\"lastCheckedAt\":null"));
        prop_assert!(json.contains("\"lastNewContentAt\":null"));
        let restored: Source = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(source.check_count, restored.check_count);
    }

}

// Stand-alone (non-proptest) test: no generated values needed.
#[test]
fn entry_nullable_fields_roundtrip() {
    // Entry nullable fields: summary, contentHash, signal (.nullable() in TS)
    // must always serialize as null when None — not absent from JSON.
    let entry = Entry {
        id: Uuid::new_v4(),
        source_id: Uuid::new_v4(),
        r#type: EntryType::Article,
        title: "test entry".to_string(),
        url: "https://example.com".to_string(),
        summary: None,
        content_hash: None,
        state: EntryState::New,
        signal: None,
        scanned_at: Utc::now(),
        metadata: None,
        created_at: Utc::now(),
    };
    let json = serde_json::to_string(&entry).unwrap();
    // Nullable fields must be present as null (not absent)
    assert!(
        json.contains("\"summary\":null"),
        "summary must be null, got: {json}"
    );
    assert!(
        json.contains("\"contentHash\":null"),
        "contentHash must be null, got: {json}"
    );
    assert!(
        json.contains("\"signal\":null"),
        "signal must be null, got: {json}"
    );
    let restored: Entry = serde_json::from_str(&json).unwrap();
    assert_eq!(entry.id, restored.id);
}
