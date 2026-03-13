//! Data compatibility test: reads TypeScript-format fixture files and asserts
//! they parse successfully with zero errors. Guards against TypeScript/Rust
//! JSON format drift.

use std::path::PathBuf;
use tankyu_core::{
    domain::{
        ports::{IEntityStore, IEntryStore, IGraphStore, IInsightStore, ISourceStore, ITopicStore},
        types::TankyuConfig,
    },
    infrastructure::{
        graph::JsonGraphStore,
        stores::{EntryStore, EntityStore, InsightStore, SourceStore, TopicStore},
    },
};

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
}

#[tokio::test]
async fn config_parses() {
    let path = fixtures_dir().join("config.json");
    let bytes = tokio::fs::read(&path).await
        .expect("fixtures/config.json must exist");
    let config: TankyuConfig = serde_json::from_slice(&bytes)
        .expect("config.json must parse as TankyuConfig");
    assert_eq!(config.version, 1);
    assert_eq!(config.stale_days, 7);
}

#[tokio::test]
async fn topic_fixture_parses() {
    let store = TopicStore::new(fixtures_dir().join("topics"));
    let topics = store.list().await.expect("topics must list without error");
    assert_eq!(topics.len(), 1, "expected 1 fixture topic");
    assert_eq!(topics[0].name, "Rust tooling");
}

#[tokio::test]
async fn source_fixture_parses() {
    let store = SourceStore::new(fixtures_dir().join("sources"));
    let sources = store.list().await.expect("sources must list without error");
    assert_eq!(sources.len(), 1, "expected 1 fixture source");
    assert_eq!(sources[0].check_count, 5);
    // All four nullable fields must be present (as null) in TypeScript data
    assert!(sources[0].discovered_via.is_none());
    assert!(sources[0].discovery_reason.is_none());
    assert!(sources[0].last_checked_at.is_none());
    assert!(sources[0].last_new_content_at.is_none());
}

#[tokio::test]
async fn entry_fixture_parses() {
    let store = EntryStore::new(fixtures_dir().join("entries"));
    let entries = store.list().await.expect("entries must list without error");
    assert_eq!(entries.len(), 1, "expected 1 fixture entry");
    assert_eq!(entries[0].title, "Stabilize feature X");
}

#[tokio::test]
async fn graph_fixture_parses() {
    let store = JsonGraphStore::new(fixtures_dir().join("graph").join("edges.json"));
    let edges = store.list().await.expect("edges.json must parse without error");
    assert_eq!(edges.len(), 1, "expected 1 fixture edge");
}

#[tokio::test]
async fn insight_fixture_parses() {
    let store = InsightStore::new(fixtures_dir().join("insights"));
    let insights = store.list().await.expect("insights must list without error");
    assert_eq!(insights.len(), 1, "expected 1 fixture insight");
    assert_eq!(insights[0].title, "Rust async patterns");
}

#[tokio::test]
async fn entity_fixture_parses() {
    let store = EntityStore::new(fixtures_dir().join("entities"));
    let entities = store.list().await.expect("entities must list without error");
    assert_eq!(entities.len(), 1, "expected 1 fixture entity");
    assert_eq!(entities[0].name, "Tokio");
}
