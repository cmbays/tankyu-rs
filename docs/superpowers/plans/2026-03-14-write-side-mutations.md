# Write-Side Mutations Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add write-side mutations (topic create, source add/remove/inspect, entry update, entry list --unclassified, health) to tankyu-rs with full BDD/TDD/insta quality loop.

**Architecture:** Approach B — mutation methods added to existing managers in `tankyu-core`; CLI handlers stay thin (parse → call manager → render). All domain logic is unit-tested with stub stores. BDD acceptance tests drive the binary via `TankyuWorld`. Insta snapshots lock rendered output.

**Tech Stack:** Rust, Cargo workspace (tankyu-core + tankyu-cli), tokio, async-trait, clap 4, comfy-table, serde_json, insta (snapshots), assert_cmd, cucumber 0.22 (BDD), cargo-mutants.

**Spec:** `docs/superpowers/specs/2026-03-14-write-side-mutations-design.md`
**TDD Reference:** `TDD_WORKFLOW.md` — read it before starting.

---

## File Map

### Files to create
| File | Purpose |
|------|---------|
| `crates/tankyu-core/src/features/source/url_detect.rs` | `detect_source_type()` + `name_from_url()` pure fns + tests |
| `crates/tankyu-core/src/features/health/mod.rs` | `HealthManager`, `HealthReport`, `HealthWarning`, `HealthThresholds` + tests |
| `crates/tankyu-cli/src/commands/health.rs` | `health::run()` CLI handler |
| `crates/tankyu-cli/tests/cli_health.rs` | Insta snapshot tests for health output |
| `crates/tankyu-cli/tests/acceptance/features/source.feature` | BDD scenarios for source commands |
| `crates/tankyu-cli/tests/acceptance/features/topic.feature` | BDD scenarios for topic commands |
| `crates/tankyu-cli/tests/acceptance/features/health.feature` | BDD scenarios for health command |
| `crates/tankyu-cli/tests/acceptance/steps/source_steps.rs` | BDD step definitions for source |
| `crates/tankyu-cli/tests/acceptance/steps/topic_steps.rs` | BDD step definitions for topic |
| `crates/tankyu-cli/tests/acceptance/steps/health_steps.rs` | BDD step definitions for health |

### Files to modify
| File | Change |
|------|--------|
| `crates/tankyu-core/src/shared/error.rs` | Add `Duplicate` variant to `TankyuError` |
| `crates/tankyu-core/src/features/source/mod.rs` | Add `pub mod url_detect;` |
| `crates/tankyu-core/src/features/source/source_manager.rs` | Add `add()`, `remove()`, `AddSourceInput` |
| `crates/tankyu-core/src/features/topic/topic_manager.rs` | Add `create()`, `CreateTopicInput` |
| `crates/tankyu-core/src/features/entry/entry_manager.rs` | Add `update()` |
| `crates/tankyu-core/src/features/mod.rs` | Add `pub mod health;` |
| `crates/tankyu-core/src/lib.rs` | Re-export `features::health` |
| `crates/tankyu-cli/src/commands/source.rs` | Add `inspect()`, `add()`, `remove()`; polish `list()` |
| `crates/tankyu-cli/src/commands/topic.rs` | Add `create()` |
| `crates/tankyu-cli/src/commands/entry.rs` | Add `update()`; add `--unclassified` to `list()` |
| `crates/tankyu-cli/src/commands/mod.rs` | Add `pub mod health;` |
| `crates/tankyu-cli/src/cli.rs` | Add new clap variants |
| `crates/tankyu-cli/src/main.rs` | Wire new commands |
| `crates/tankyu-cli/src/context.rs` | Add `health_mgr`, expose `graph_store` |
| `crates/tankyu-cli/tests/cli_source.rs` | Extend with new snapshots |
| `crates/tankyu-cli/tests/cli_topic.rs` | Extend with new snapshots |
| `crates/tankyu-cli/tests/cli_entry.rs` | Extend with new snapshots |
| `crates/tankyu-cli/tests/acceptance/steps/mod.rs` | Add new step modules |
| `crates/tankyu-cli/tests/acceptance/features/entry.feature` | Add new scenarios |
| `crates/tankyu-cli/tests/acceptance/steps/entry_steps.rs` | Add new step defs |
| `crates/tankyu-cli/tests/acceptance/world.rs` | Add `write_source()`, `write_topic()`, `write_tagged_with_edge()` helpers |

---

## Chunk 1: Core Infrastructure

Error type extension and URL detection pure functions. No I/O, no async. Fully testable in isolation.

### Task 1: Add `TankyuError::Duplicate`

**Files:**
- Modify: `crates/tankyu-core/src/shared/error.rs`

- [ ] **Step 1.1: Write the failing test first**

Add to the `#[cfg(test)]` block in `error.rs`:

```rust
#[test]
fn duplicate_displays_message() {
    let err = TankyuError::Duplicate {
        kind: "topic".to_string(),
        name: "Rust".to_string(),
    };
    assert_eq!(err.to_string(), "duplicate topic: 'Rust' already exists");
}
```

- [ ] **Step 1.2: Run to confirm it fails**

```bash
cargo test -p tankyu-core duplicate_displays_message 2>&1 | tail -5
```

Expected: compile error — `Duplicate` variant doesn't exist yet.

- [ ] **Step 1.3: Add the variant**

In `error.rs`, add after the `Scan` variant:

```rust
#[error("duplicate {kind}: '{name}' already exists")]
Duplicate { kind: String, name: String },
```

- [ ] **Step 1.4: Run test to confirm pass**

```bash
cargo test -p tankyu-core duplicate_displays_message
```

Expected: `test shared::error::tests::duplicate_displays_message ... ok`

- [ ] **Step 1.5: Lint and commit**

```bash
cargo clippy -p tankyu-core -- -D warnings
git add crates/tankyu-core/src/shared/error.rs
git commit -m "feat(core): add TankyuError::Duplicate variant"
```

---

### Task 2: `url_detect.rs` — pure detection functions

**Files:**
- Create: `crates/tankyu-core/src/features/source/url_detect.rs`
- Modify: `crates/tankyu-core/src/features/source/mod.rs`

- [ ] **Step 2.1: Declare the module first (so tests compile)**

In `crates/tankyu-core/src/features/source/mod.rs`, add:

```rust
pub mod url_detect;
pub use url_detect::{detect_source_type, name_from_url};
```

- [ ] **Step 2.2: Create `url_detect.rs` with tests only (stubs fail to compile)**

```rust
use crate::domain::types::SourceType;

pub fn detect_source_type(_url: &str) -> SourceType {
    todo!()
}

pub fn name_from_url(_url: &str) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::SourceType;

    // ── detect_source_type ─────────────────────────────────────────────────

    #[test]
    fn github_issues_url() {
        assert_eq!(
            detect_source_type("https://github.com/rust-lang/rust/issues"),
            SourceType::GithubIssues
        );
    }

    #[test]
    fn github_releases_url() {
        assert_eq!(
            detect_source_type("https://github.com/rust-lang/rust/releases"),
            SourceType::GithubReleases
        );
    }

    #[test]
    fn github_repo_url() {
        assert_eq!(
            detect_source_type("https://github.com/rust-lang/rust"),
            SourceType::GithubRepo
        );
    }

    /// Mutation killer: issues must be checked BEFORE repo to avoid false GithubRepo match
    #[test]
    fn github_issues_not_matched_as_repo() {
        assert_ne!(
            detect_source_type("https://github.com/rust-lang/rust/issues"),
            SourceType::GithubRepo
        );
    }

    #[test]
    fn github_user_url() {
        assert_eq!(
            detect_source_type("https://github.com/torvalds"),
            SourceType::GithubUser
        );
    }

    #[test]
    fn x_account_url() {
        assert_eq!(
            detect_source_type("https://x.com/karpathy"),
            SourceType::XAccount
        );
    }

    #[test]
    fn twitter_account_url() {
        assert_eq!(
            detect_source_type("https://twitter.com/karpathy"),
            SourceType::XAccount
        );
    }

    #[test]
    fn blog_medium_url() {
        assert_eq!(
            detect_source_type("https://medium.com/some-article"),
            SourceType::Blog
        );
    }

    #[test]
    fn blog_substack_url() {
        assert_eq!(
            detect_source_type("https://example.substack.com/post"),
            SourceType::Blog
        );
    }

    #[test]
    fn rss_feed_url() {
        assert_eq!(
            detect_source_type("https://example.com/feed.xml"),
            SourceType::RssFeed
        );
    }

    #[test]
    fn rss_atom_path() {
        assert_eq!(
            detect_source_type("https://example.com/atom"),
            SourceType::RssFeed
        );
    }

    #[test]
    fn agent_report_file_scheme() {
        assert_eq!(
            detect_source_type("file:///reports/spike.md"),
            SourceType::AgentReport
        );
    }

    #[test]
    fn web_page_fallback() {
        assert_eq!(
            detect_source_type("https://example.com/some/page"),
            SourceType::WebPage
        );
    }

    // ── name_from_url ──────────────────────────────────────────────────────

    #[test]
    fn github_repo_name() {
        assert_eq!(name_from_url("https://github.com/rust-lang/rust"), "rust-lang/rust");
    }

    #[test]
    fn single_path_segment() {
        assert_eq!(name_from_url("https://github.com/torvalds"), "torvalds");
    }

    #[test]
    fn hostname_fallback() {
        assert_eq!(name_from_url("https://example.com"), "example.com");
    }

    #[test]
    fn x_account_name() {
        assert_eq!(name_from_url("https://x.com/karpathy"), "karpathy");
    }
}
```

- [ ] **Step 2.3: Run tests to confirm they fail with `todo!()`**

```bash
cargo test -p tankyu-core url_detect 2>&1 | tail -10
```

Expected: multiple `panicked at 'not yet implemented'` failures.

- [ ] **Step 2.4: Implement `detect_source_type`**

Replace the `todo!()` stub:

```rust
pub fn detect_source_type(url: &str) -> SourceType {
    // Order matters: more-specific GitHub paths before general ones
    if url.contains("github.com") {
        if url.contains("/issues") {
            return SourceType::GithubIssues;
        }
        if url.contains("/releases") {
            return SourceType::GithubReleases;
        }
        // Two path segments → repo; one → user
        let path = url
            .splitn(2, "github.com/")
            .nth(1)
            .unwrap_or("")
            .trim_end_matches('/');
        if path.contains('/') {
            return SourceType::GithubRepo;
        }
        return SourceType::GithubUser;
    }
    if url.contains("x.com/") || url.contains("twitter.com/") {
        return SourceType::XAccount;
    }
    if url.contains("medium.com")
        || url.contains("substack.com")
        || url.contains("dev.to")
        || url.contains(".blog")
        || url.contains("blog.")
    {
        return SourceType::Blog;
    }
    if url.contains("/feed")
        || url.contains("/rss")
        || url.contains("/atom")
        || url.ends_with(".xml")
    {
        return SourceType::RssFeed;
    }
    if url.starts_with("file:") {
        return SourceType::AgentReport;
    }
    SourceType::WebPage
}
```

- [ ] **Step 2.5: Implement `name_from_url`**

```rust
pub fn name_from_url(url: &str) -> String {
    let without_scheme = url.splitn(2, "://").nth(1).unwrap_or(url);
    let without_query = without_scheme.splitn(2, '?').next().unwrap_or(without_scheme);
    let parts: Vec<&str> = without_query
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    // parts[0] = hostname; parts[1..] = path segments
    match parts.get(1) {
        None => parts.first().copied().unwrap_or(url).to_string(),
        Some(seg1) => match parts.get(2) {
            None => seg1.to_string(),
            Some(seg2) => format!("{seg1}/{seg2}"),
        },
    }
}
```

- [ ] **Step 2.6: Run all url_detect tests**

```bash
cargo test -p tankyu-core url_detect
```

Expected: all 14+ tests pass.

- [ ] **Step 2.7: Lint and commit**

```bash
cargo clippy -p tankyu-core -- -D warnings
git add crates/tankyu-core/src/features/source/
git commit -m "feat(core): add url_detect — detect_source_type + name_from_url pure fns"
```

---

## Chunk 2: Core Managers

Write-side methods for `TopicManager`, `SourceManager`, `EntryManager`. Each method follows the same pattern: failing unit test with stub store → minimal implementation → pass.

### Task 3: `TopicManager::create()`

**Files:**
- Modify: `crates/tankyu-core/src/features/topic/topic_manager.rs`

- [ ] **Step 3.1: Add `CreateTopicInput` and the failing test**

At the top of the file (after imports), add:

```rust
use crate::shared::error::TankyuError;
use chrono::Utc;
use uuid::Uuid;

/// Input for creating a new topic.
pub struct CreateTopicInput {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}
```

In the existing `#[cfg(test)]` block, update `StubTopicStore::create` from `unimplemented!()` to a working stub:

```rust
async fn create(&self, _topic: Topic) -> Result<()> {
    Ok(()) // stub — real durability tested in store_compat
}
```

Add these tests:

```rust
#[tokio::test]
async fn test_create_returns_topic_with_correct_fields() {
    let store = Arc::new(StubTopicStore { topics: vec![] });
    let mgr = TopicManager::new(store);
    let result = mgr
        .create(CreateTopicInput {
            name: "Rust Async".to_string(),
            description: "Async Rust patterns".to_string(),
            tags: vec!["rust".to_string(), "async".to_string()],
        })
        .await
        .unwrap();
    assert_eq!(result.name, "Rust Async");
    assert_eq!(result.description, "Async Rust patterns");
    assert_eq!(result.tags, vec!["rust", "async"]);
    assert_eq!(result.scan_count, 0);
    assert!(result.last_scanned_at.is_none());
    assert!(result.projects.is_empty());
}

#[tokio::test]
async fn test_create_duplicate_name_returns_error() {
    let existing = make_topic("Duplicate");
    let store = Arc::new(StubTopicStore { topics: vec![existing] });
    let mgr = TopicManager::new(store);
    let err = mgr
        .create(CreateTopicInput {
            name: "Duplicate".to_string(),
            description: "".to_string(),
            tags: vec![],
        })
        .await
        .unwrap_err();
    // Downcast to TankyuError::Duplicate
    let tankyu_err = err.downcast::<TankyuError>().unwrap();
    assert!(matches!(tankyu_err, TankyuError::Duplicate { .. }));
}
```

_(The `StubTopicStore::create` stub was already updated above in this step.)_

- [ ] **Step 3.2: Run to confirm tests fail**

```bash
cargo test -p tankyu-core test_create -- --nocapture 2>&1 | tail -10
```

Expected: compile error — `create` method not defined on `TopicManager`.

- [ ] **Step 3.3: Implement `TopicManager::create()`**

Add to the `impl TopicManager` block:

```rust
/// Create a new topic. Errors with `TankyuError::Duplicate` if name already exists.
///
/// # Errors
/// Returns `TankyuError::Duplicate` if a topic with this name already exists.
/// Returns an error if the store write fails.
pub async fn create(&self, input: CreateTopicInput) -> Result<Topic> {
    if self.store.get_by_name(&input.name).await?.is_some() {
        return Err(TankyuError::Duplicate {
            kind: "topic".to_string(),
            name: input.name,
        }
        .into());
    }
    let now = Utc::now();
    let topic = Topic {
        id: Uuid::new_v4(),
        name: input.name,
        description: input.description,
        tags: input.tags,
        projects: vec![],
        routing: None,
        created_at: now,
        updated_at: now,
        last_scanned_at: None,
        scan_count: 0,
    };
    self.store.create(topic.clone()).await?;
    Ok(topic)
}
```

- [ ] **Step 3.4: Run tests**

```bash
cargo test -p tankyu-core test_create
```

Expected: both tests pass.

- [ ] **Step 3.5: Lint and commit**

```bash
cargo clippy -p tankyu-core -- -D warnings
git add crates/tankyu-core/src/features/topic/topic_manager.rs
git commit -m "feat(core): TopicManager::create — dedup check, UUID, timestamps"
```

---

### Task 4: `SourceManager::add()`

**Files:**
- Modify: `crates/tankyu-core/src/features/source/source_manager.rs`

- [ ] **Step 4.1: Add `AddSourceInput` struct and failing tests**

Add near the top of `source_manager.rs` (after imports):

```rust
use crate::features::source::url_detect::{detect_source_type, name_from_url};
use crate::shared::error::TankyuError;
use chrono::Utc;
use uuid::Uuid;

/// Input for adding a new source.
pub struct AddSourceInput {
    pub url: String,
    /// Override the auto-detected name.
    pub name: Option<String>,
    /// Override the auto-detected source type.
    pub source_type: Option<SourceType>,
    pub role: Option<SourceRole>,
    /// If provided, a `Monitors` edge is created from this topic to the source.
    pub topic_id: Option<Uuid>,
}
```

Update `StubSourceStore::create` to succeed (change from `unimplemented!()` to `Ok(())`).
Update `StubSourceStore::get_by_url` to search by URL:

```rust
async fn get_by_url(&self, url: &str) -> Result<Option<Source>> {
    Ok(self.sources.iter().find(|s| s.url == url).cloned())
}
```

Update `StubGraphStore::add_edge` to succeed:
```rust
async fn add_edge(&self, _e: Edge) -> Result<()> { Ok(()) }
```

Add tests to the `#[cfg(test)]` block:

```rust
#[tokio::test]
async fn test_add_creates_new_source_for_unknown_url() {
    let store = Arc::new(StubSourceStore { sources: vec![] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = SourceManager::new(store, graph);
    let result = mgr
        .add(AddSourceInput {
            url: "https://github.com/rust-lang/rust".to_string(),
            name: None,
            source_type: None,
            role: None,
            topic_id: None,
        })
        .await
        .unwrap();
    assert_eq!(result.url, "https://github.com/rust-lang/rust");
    assert_eq!(result.name, "rust-lang/rust");
    assert!(matches!(result.state, SourceState::Active));
    assert_eq!(result.check_count, 0);
}

#[tokio::test]
async fn test_add_returns_existing_source_for_duplicate_url() {
    let existing = make_source(Uuid::new_v4(), None);
    let url = existing.url.clone();
    let store = Arc::new(StubSourceStore { sources: vec![existing.clone()] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = SourceManager::new(store, graph);
    let result = mgr
        .add(AddSourceInput {
            url,
            name: None,
            source_type: None,
            role: None,
            topic_id: None,
        })
        .await
        .unwrap();
    assert_eq!(result.id, existing.id);
}

#[tokio::test]
async fn test_add_with_name_override_uses_provided_name() {
    let store = Arc::new(StubSourceStore { sources: vec![] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = SourceManager::new(store, graph);
    let result = mgr
        .add(AddSourceInput {
            url: "https://example.com/page".to_string(),
            name: Some("My Custom Name".to_string()),
            source_type: None,
            role: None,
            topic_id: None,
        })
        .await
        .unwrap();
    assert_eq!(result.name, "My Custom Name");
}

#[tokio::test]
async fn test_add_with_topic_id_creates_monitors_edge() {
    use std::sync::Mutex;
    use crate::domain::types::Edge;

    struct CountingGraphStore {
        edges: Mutex<Vec<Edge>>,
    }
    #[async_trait]
    impl IGraphStore for CountingGraphStore {
        async fn add_edge(&self, e: Edge) -> Result<()> {
            self.edges.lock().unwrap().push(e);
            Ok(())
        }
        async fn get_edges_by_node(&self, _id: Uuid) -> Result<Vec<Edge>> {
            Ok(self.edges.lock().unwrap().clone())
        }
        async fn query(&self, _q: crate::domain::types::GraphQuery) -> Result<Vec<Edge>> {
            Ok(self.edges.lock().unwrap().clone())
        }
    }

    let topic_id = Uuid::new_v4();
    let store = Arc::new(StubSourceStore { sources: vec![] });
    let graph = Arc::new(CountingGraphStore { edges: Mutex::new(vec![]) });
    let mgr = SourceManager::new(store, Arc::clone(&graph) as Arc<dyn IGraphStore>);
    mgr.add(AddSourceInput {
        url: "https://github.com/rust-lang/rust".to_string(),
        name: None,
        source_type: None,
        role: None,
        topic_id: Some(topic_id),
    })
    .await
    .unwrap();
    assert_eq!(graph.edges.lock().unwrap().len(), 1, "expected exactly one Monitors edge");
    let edge = graph.edges.lock().unwrap()[0].clone();
    assert_eq!(edge.from_id, topic_id);
    assert!(matches!(edge.edge_type, EdgeType::Monitors));
}

#[tokio::test]
async fn test_add_duplicate_topic_edge_is_skipped() {
    use std::sync::Mutex;
    use crate::domain::types::Edge;

    struct CountingGraphStore {
        edges: Mutex<Vec<Edge>>,
    }
    #[async_trait]
    impl IGraphStore for CountingGraphStore {
        async fn add_edge(&self, e: Edge) -> Result<()> {
            self.edges.lock().unwrap().push(e);
            Ok(())
        }
        async fn get_edges_by_node(&self, _id: Uuid) -> Result<Vec<Edge>> {
            Ok(self.edges.lock().unwrap().clone())
        }
        async fn query(&self, _q: crate::domain::types::GraphQuery) -> Result<Vec<Edge>> {
            Ok(self.edges.lock().unwrap().clone())
        }
    }

    let topic_id = Uuid::new_v4();
    let url = "https://github.com/rust-lang/rust".to_string();
    // First call creates the source and the edge
    let store = Arc::new(StubSourceStore { sources: vec![] });
    let graph = Arc::new(CountingGraphStore { edges: Mutex::new(vec![]) });
    let mgr = SourceManager::new(store, Arc::clone(&graph) as Arc<dyn IGraphStore>);
    mgr.add(AddSourceInput {
        url: url.clone(),
        name: None,
        source_type: None,
        role: None,
        topic_id: Some(topic_id),
    })
    .await
    .unwrap();
    // Second call (same URL + same topic): edge must NOT be created again
    mgr.add(AddSourceInput {
        url,
        name: None,
        source_type: None,
        role: None,
        topic_id: Some(topic_id),
    })
    .await
    .unwrap();
    assert_eq!(
        graph.edges.lock().unwrap().len(),
        1,
        "dedup guard should prevent a second Monitors edge"
    );
}
```

- [ ] **Step 4.2: Run to confirm failure**

```bash
cargo test -p tankyu-core test_add 2>&1 | tail -5
```

Expected: compile error — `add` not defined.

- [ ] **Step 4.3: Implement `SourceManager::add()`**

Add to `impl SourceManager`:

```rust
/// Add a source. Idempotent by URL: returns existing source if URL already known.
/// If `input.role` differs from existing, updates the role.
/// If `input.topic_id` provided, creates a `Monitors` edge (deduplicated).
///
/// # Errors
/// Returns an error if any store or graph operation fails.
pub async fn add(&self, input: AddSourceInput) -> Result<Source> {
    // Idempotency: return existing source if URL already tracked
    if let Some(existing) = self.store.get_by_url(&input.url).await? {
        // Update role if provided and different
        let source = if let Some(role) = &input.role {
            if existing.role.as_ref() != Some(role) {
                self.store
                    .update(
                        existing.id,
                        SourceUpdate {
                            role: Some(role.clone()),
                            ..Default::default()
                        },
                    )
                    .await?
            } else {
                existing
            }
        } else {
            existing
        };
        // Add monitors edge if topic provided
        if let Some(topic_id) = input.topic_id {
            self.ensure_monitors_edge(source.id, topic_id).await?;
        }
        return Ok(source);
    }

    // New source
    let source_type = input.source_type.unwrap_or_else(|| detect_source_type(&input.url));
    let name = input.name.unwrap_or_else(|| name_from_url(&input.url));
    let source = Source {
        id: Uuid::new_v4(),
        r#type: source_type,
        role: input.role,
        name,
        url: input.url,
        config: None,
        state: SourceState::Active,
        poll_interval_minutes: None,
        discovered_via: None,
        discovery_reason: None,
        last_checked_at: None,
        last_new_content_at: None,
        check_count: 0,
        hit_count: 0,
        miss_count: 0,
        created_at: Utc::now(),
    };
    self.store.create(source.clone()).await?;

    if let Some(topic_id) = input.topic_id {
        self.ensure_monitors_edge(source.id, topic_id).await?;
    }
    Ok(source)
}

/// Create a `Monitors` edge from `topic_id` → `source_id` unless one already exists.
async fn ensure_monitors_edge(&self, source_id: Uuid, topic_id: Uuid) -> Result<()> {
    let edges = self.graph.get_edges_by_node(source_id).await?;
    let already_exists = edges.iter().any(|e| {
        e.from_id == topic_id && e.to_id == source_id && e.edge_type == EdgeType::Monitors
    });
    if already_exists {
        return Ok(());
    }
    let edge = Edge {
        id: Uuid::new_v4(),
        from_id: topic_id,
        from_type: NodeType::Topic,
        to_id: source_id,
        to_type: NodeType::Source,
        edge_type: EdgeType::Monitors,
        reason: format!("Topic monitors source"),
        score: None,
        method: None,
        created_at: Utc::now(),
    };
    self.graph.add_edge(edge).await
}
```

You'll need these additional imports at the top of the file:

```rust
use crate::domain::types::{
    Edge, EdgeType, NodeType, Source, SourceRole, SourceState, SourceUpdate, SourceType,
};
use crate::features::source::url_detect::{detect_source_type, name_from_url};
use crate::shared::error::TankyuError;
use chrono::Utc;
use uuid::Uuid;
```

- [ ] **Step 4.4: Run tests**

```bash
cargo test -p tankyu-core test_add
```

Expected: all 4 tests pass.

- [ ] **Step 4.5: Lint and commit**

```bash
cargo clippy -p tankyu-core -- -D warnings
git add crates/tankyu-core/src/features/source/
git commit -m "feat(core): SourceManager::add — idempotent by URL, type detection, monitors edge"
```

---

### Task 5: `SourceManager::remove()`

**Files:**
- Modify: `crates/tankyu-core/src/features/source/source_manager.rs`

- [ ] **Step 5.1: Write failing tests**

Add to `#[cfg(test)]`:

```rust
#[tokio::test]
async fn test_remove_sets_state_to_pruned() {
    let id = Uuid::new_v4();
    let mut source = make_source(id, None);
    source.name = "to-remove".to_string();
    // StubSourceStore needs a mutable update — use the existing update stub.
    // For this test, we verify the manager calls remove correctly by checking
    // that update is called with Pruned state. Since StubSourceStore::update
    // is unimplemented, we need to implement it for remove tests.
    // Update StubSourceStore::update to return a pruned copy:
    // (see Step 5.2 for stub update)
    let store = Arc::new(StubSourceStoreWithUpdate {
        sources: vec![source.clone()],
    });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = SourceManager::new(store, graph);
    let result = mgr.remove("to-remove").await.unwrap();
    assert!(matches!(result.state, SourceState::Pruned));
}

#[tokio::test]
async fn test_remove_unknown_name_returns_not_found() {
    let store = Arc::new(StubSourceStore { sources: vec![] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = SourceManager::new(store, graph);
    let err = mgr.remove("nonexistent").await.unwrap_err();
    let tankyu_err = err.downcast::<TankyuError>().unwrap();
    assert!(matches!(tankyu_err, TankyuError::NotFound(_)));
}
```

For the `update` stub, add a second stub store type in the tests section:

```rust
struct StubSourceStoreWithUpdate {
    sources: Vec<Source>,
}

#[async_trait]
impl ISourceStore for StubSourceStoreWithUpdate {
    async fn create(&self, _s: Source) -> Result<()> { Ok(()) }
    async fn get(&self, id: Uuid) -> Result<Option<Source>> {
        Ok(self.sources.iter().find(|s| s.id == id).cloned())
    }
    async fn get_by_url(&self, _url: &str) -> Result<Option<Source>> { Ok(None) }
    async fn list(&self) -> Result<Vec<Source>> { Ok(self.sources.clone()) }
    async fn update(&self, id: Uuid, updates: SourceUpdate) -> Result<Source> {
        let mut source = self.sources.iter().find(|s| s.id == id)
            .cloned()
            .ok_or_else(|| TankyuError::NotFound(id.to_string()))?;
        if let Some(state) = updates.state { source.state = state; }
        if let Some(role) = updates.role { source.role = Some(role); }
        Ok(source)
    }
}
```

- [ ] **Step 5.2: Run to confirm failure**

```bash
cargo test -p tankyu-core test_remove 2>&1 | tail -5
```

Expected: compile error — `remove` not defined.

- [ ] **Step 5.3: Implement `remove()`**

```rust
/// Mark a source as pruned by name. Returns the updated source.
///
/// # Errors
/// Returns `TankyuError::NotFound` if no source with this name exists.
pub async fn remove(&self, name: &str) -> Result<Source> {
    let source = self
        .get_by_name(name)
        .await?
        .ok_or_else(|| TankyuError::NotFound(format!("source '{name}'")))?;
    self.store
        .update(
            source.id,
            SourceUpdate {
                state: Some(SourceState::Pruned),
                ..Default::default()
            },
        )
        .await
}
```

- [ ] **Step 5.4: Run tests**

```bash
cargo test -p tankyu-core test_remove
```

Expected: both pass.

- [ ] **Step 5.5: Lint and commit**

```bash
cargo clippy -p tankyu-core -- -D warnings
git add crates/tankyu-core/src/features/source/source_manager.rs
git commit -m "feat(core): SourceManager::remove — set state to Pruned by name"
```

---

### Task 6: `EntryManager::update()`

**Files:**
- Modify: `crates/tankyu-core/src/features/entry/entry_manager.rs`

- [ ] **Step 6.1: Write failing tests**

In the existing `#[cfg(test)]` block, first add to the import block at the top of the test module:

```rust
use crate::shared::error::TankyuError;
```

Then update `StubEntryStore::update` from `unimplemented!()` to a working stub:

```rust
async fn update(&self, id: Uuid, u: EntryUpdate) -> Result<Entry> {
    let mut entry = self.entries.iter().find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| TankyuError::NotFound(id.to_string()))?;
    if let Some(state) = u.state { entry.state = state; }
    if let Some(signal) = u.signal { entry.signal = Some(signal); }
    if let Some(summary) = u.summary { entry.summary = Some(summary); }
    Ok(entry)
}
```

Add tests:

```rust
#[tokio::test]
async fn test_update_state_only() {
    let entry = make_entry("Target", EntryState::New);
    let id = entry.id;
    let store = Arc::new(StubEntryStore { entries: vec![entry] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = EntryManager::new(store, graph);
    let result = mgr
        .update(id, EntryUpdate { state: Some(EntryState::Read), signal: None, summary: None })
        .await
        .unwrap();
    assert!(matches!(result.state, EntryState::Read));
}

#[tokio::test]
async fn test_update_signal_only() {
    let entry = make_entry("Target", EntryState::New);
    let id = entry.id;
    let store = Arc::new(StubEntryStore { entries: vec![entry] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = EntryManager::new(store, graph);
    let result = mgr
        .update(id, EntryUpdate { state: None, signal: Some(Signal::High), summary: None })
        .await
        .unwrap();
    assert_eq!(result.signal, Some(Signal::High));
}

#[tokio::test]
async fn test_update_state_and_signal() {
    let entry = make_entry("Target", EntryState::New);
    let id = entry.id;
    let store = Arc::new(StubEntryStore { entries: vec![entry] });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = EntryManager::new(store, graph);
    let result = mgr
        .update(id, EntryUpdate { state: Some(EntryState::Triaged), signal: Some(Signal::Medium), summary: None })
        .await
        .unwrap();
    assert!(matches!(result.state, EntryState::Triaged));
    assert_eq!(result.signal, Some(Signal::Medium));
}
```

- [ ] **Step 6.2: Run to confirm failure**

```bash
cargo test -p tankyu-core test_update 2>&1 | tail -5
```

Expected: compile error — `update` not defined on `EntryManager`.

- [ ] **Step 6.3: Implement `update()`**

```rust
/// Apply a partial update to an entry. Returns the updated entry.
///
/// # Errors
/// Returns an error if the entry is not found or the store write fails.
pub async fn update(&self, id: Uuid, patch: EntryUpdate) -> Result<Entry> {
    self.store.update(id, patch).await
}
```

- [ ] **Step 6.4: Run tests**

```bash
cargo test -p tankyu-core test_update
```

Expected: all 3 pass.

- [ ] **Step 6.5: Lint and run full core test suite**

```bash
cargo clippy -p tankyu-core -- -D warnings
cargo test -p tankyu-core
```

Expected: all tests pass.

- [ ] **Step 6.6: Commit**

```bash
git add crates/tankyu-core/src/features/entry/entry_manager.rs
git commit -m "feat(core): EntryManager::update — delegates to store.update"
```

---

## Chunk 3: HealthManager

New `features/health` module. Pure domain logic over stores — no async I/O beyond store calls. Fully unit-tested with stub stores.

### Task 7: `HealthManager`

**Files:**
- Create: `crates/tankyu-core/src/features/health/mod.rs`
- Modify: `crates/tankyu-core/src/features/mod.rs`
- Modify: `crates/tankyu-core/src/lib.rs`

- [ ] **Step 7.1: Declare the module**

In `crates/tankyu-core/src/features/mod.rs`, add:

```rust
pub mod health;
```

In `crates/tankyu-core/src/lib.rs`, verify `features` is re-exported (it should already be `pub mod features`). Add if needed:

```rust
pub use features::health::{HealthManager, HealthReport, HealthThresholds, HealthWarning, HealthWarningKind};
```

- [ ] **Step 7.2: Create `health/mod.rs` with types and failing tests**

```rust
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
        Self { source_store, entry_store }
    }

    /// Run health checks and return a report.
    ///
    /// # Errors
    /// Returns an error if any store read fails.
    pub async fn health(&self, thresholds: HealthThresholds) -> Result<HealthReport> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        ports::{IEntryStore, ISourceStore},
        types::{
            Entry, EntryState, EntryType, EntryUpdate, Source, SourceRole, SourceState,
            SourceType, SourceUpdate,
        },
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::{Duration, Utc};
    use std::sync::Arc;
    use uuid::Uuid;

    struct StubSourceStore { sources: Vec<Source> }
    struct StubEntryStore { entries: Vec<Entry> }

    #[async_trait]
    impl ISourceStore for StubSourceStore {
        async fn create(&self, _s: Source) -> Result<()> { Ok(()) }
        async fn get(&self, id: Uuid) -> Result<Option<Source>> {
            Ok(self.sources.iter().find(|s| s.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Source>> { Ok(None) }
        async fn list(&self) -> Result<Vec<Source>> { Ok(self.sources.clone()) }
        async fn update(&self, _id: Uuid, _u: SourceUpdate) -> Result<Source> { unimplemented!() }
    }

    #[async_trait]
    impl IEntryStore for StubEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> { Ok(()) }
        async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
            Ok(self.entries.iter().find(|e| e.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Entry>> { Ok(None) }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> { Ok(None) }
        async fn list_by_source(&self, sid: Uuid) -> Result<Vec<Entry>> {
            Ok(self.entries.iter().filter(|e| e.source_id == sid).cloned().collect())
        }
        async fn list(&self) -> Result<Vec<Entry>> { Ok(self.entries.clone()) }
        async fn update(&self, _id: Uuid, _u: EntryUpdate) -> Result<Entry> { unimplemented!() }
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
        HealthThresholds { stale_days: 7, dormant_days: 30 }
    }

    // ── Tests ─────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_recently_checked_source_produces_no_staleness_warning() {
        let source = make_source("fresh", Some(1)); // checked 1 day ago
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![entry] });
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
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![entry] });
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
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![entry] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Dormant);
        // Must NOT be Stale
        assert!(!report.warnings.iter().any(|w| w.kind == HealthWarningKind::Stale));
    }

    #[tokio::test]
    async fn test_never_checked_source_produces_stale_warning() {
        // NULL last_checked_at → Stale (not Dormant)
        let source = make_source("never-checked", None);
        let id = source.id;
        let entry = make_entry(id);
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![entry] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Stale);
        assert!(report.warnings[0].detail.contains("never checked"));
    }

    #[tokio::test]
    async fn test_source_with_no_entries_produces_empty_warning() {
        let source = make_source("empty", Some(1)); // fresh but no entries
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![] }); // no entries
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
        assert_eq!(report.warnings[0].kind, HealthWarningKind::Empty);
    }

    #[tokio::test]
    async fn test_source_can_have_both_stale_and_empty_warnings() {
        let source = make_source("stale-and-empty", Some(10)); // stale + no entries
        let store = Arc::new(StubSourceStore { sources: vec![source] });
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
        let store = Arc::new(StubSourceStore { sources: vec![source] });
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
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![entry] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(report.ok);
    }

    #[tokio::test]
    async fn test_ok_false_when_any_warning() {
        let source = make_source("unhealthy", None);
        let store = Arc::new(StubSourceStore { sources: vec![source] });
        let entries = Arc::new(StubEntryStore { entries: vec![] });
        let mgr = HealthManager::new(store, entries);
        let report = mgr.health(thresholds()).await.unwrap();
        assert!(!report.ok);
    }
}
```

- [ ] **Step 7.3: Run to confirm tests fail with `todo!()`**

```bash
cargo test -p tankyu-core health 2>&1 | tail -10
```

Expected: panics from `todo!()`.

- [ ] **Step 7.4: Implement `health()`**

Replace `todo!()`:

```rust
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
                let age_days = (now - last_checked).num_days() as u32;
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
```

- [ ] **Step 7.5: Run health tests**

```bash
cargo test -p tankyu-core health
```

Expected: all 9 tests pass.

- [ ] **Step 7.6: Run full core suite + lint**

```bash
cargo test -p tankyu-core && cargo clippy -p tankyu-core -- -D warnings
```

- [ ] **Step 7.7: Commit**

```bash
git add crates/tankyu-core/src/features/health/ crates/tankyu-core/src/features/mod.rs crates/tankyu-core/src/lib.rs
git commit -m "feat(core): HealthManager — config-driven stale/dormant/empty checks"
```

---

## Chunk 4: CLI Layer

Wire all new commands. Start with scaffolding (`cli.rs`, `context.rs`, `mod.rs`), then implement each handler, then wire `main.rs`.

### Task 8: AppContext + scaffolding

**Files:**
- Modify: `crates/tankyu-cli/src/context.rs`
- Modify: `crates/tankyu-cli/src/commands/mod.rs`

- [ ] **Step 8.1: Update `context.rs` to add `health_mgr` and `graph_store`**

Add to imports:
```rust
use tankyu_core::features::health::HealthManager;
```

Add to `AppContext` struct:
```rust
pub health_mgr: HealthManager,
pub graph_store: Arc<dyn IGraphStore>,
```

In `AppContext::new()`, clone the arcs BEFORE passing them to managers so `HealthManager` can share them without creating duplicate store instances pointing at the same files:

```rust
let topic_store: Arc<dyn ITopicStore> =
    Arc::new(TopicStore::new(constants::topics_dir(&base)));
let source_store: Arc<dyn ISourceStore> =
    Arc::new(SourceStore::new(constants::sources_dir(&base)));
let graph_store: Arc<dyn IGraphStore> =
    Arc::new(JsonGraphStore::new(constants::edges_path(&base)));
let entry_store: Arc<dyn IEntryStore> =
    Arc::new(EntryStore::new(constants::entries_dir(&base)));

// Clone arcs for HealthManager before they are moved into managers
let health_source_store = Arc::clone(&source_store);
let health_entry_store = Arc::clone(&entry_store);

Ok(Self {
    topic_mgr: TopicManager::new(topic_store),
    source_mgr: SourceManager::new(source_store, Arc::clone(&graph_store)),
    entry_mgr: EntryManager::new(entry_store, Arc::clone(&graph_store)),
    health_mgr: HealthManager::new(health_source_store, health_entry_store),
    graph_store: Arc::clone(&graph_store),
    config,
    output: OutputMode::detect(json),
    data_dir: base,
})
```

- [ ] **Step 8.2: Add `pub mod health` to commands**

In `crates/tankyu-cli/src/commands/mod.rs`, add:

```rust
pub mod health;
```

- [ ] **Step 8.3: Create empty `commands/health.rs`**

```rust
use anyhow::Result;
use crate::context::AppContext;

pub async fn run(_ctx: &AppContext) -> Result<()> {
    todo!()
}
```

- [ ] **Step 8.4: Verify it compiles**

```bash
cargo build -p tankyu-cli 2>&1 | head -20
```

---

### Task 9: New clap variants

**Files:**
- Modify: `crates/tankyu-cli/src/cli.rs`

- [ ] **Step 9.1: Add all new variants**

Replace the entire `cli.rs` content:

```rust
use std::path::PathBuf;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "tankyu", version, about = "Research intelligence graph")]
pub struct Cli {
    #[arg(long, global = true, help = "Override data directory")]
    pub tankyu_dir: Option<PathBuf>,
    #[arg(long, global = true, help = "Output as JSON")]
    pub json: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Dashboard with source, topic, and entry counts
    Status,
    /// Topic management
    Topic {
        #[command(subcommand)]
        command: TopicCommands,
    },
    /// Source management
    Source {
        #[command(subcommand)]
        command: SourceCommands,
    },
    /// Entry management
    Entry {
        #[command(subcommand)]
        command: EntryCommands,
    },
    /// Configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Run diagnostics on the data directory
    Doctor,
    /// Check source health — stale, dormant, and empty sources
    Health,
}

#[derive(Subcommand)]
pub enum TopicCommands {
    /// List all topics
    List,
    /// Inspect a topic by name
    Inspect { name: String },
    /// Create a new research topic
    Create {
        name: String,
        /// Topic description
        #[arg(long, default_value = "")]
        description: String,
        /// Comma-separated tags (e.g. rust,async)
        #[arg(long, default_value = "")]
        tags: String,
    },
}

#[derive(Subcommand)]
pub enum SourceCommands {
    /// List sources, optionally filtered by topic or role
    List {
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        role: Option<String>,
    },
    /// Show full details for a source
    Inspect { name: String },
    /// Add a source (auto-detects type from URL)
    Add {
        url: String,
        /// Override the auto-detected name
        #[arg(long)]
        name: Option<String>,
        /// Link to a topic (creates Monitors edge)
        #[arg(long)]
        topic: Option<String>,
        /// Set role: starred, role-model, reference
        #[arg(long)]
        role: Option<String>,
        /// Override source type
        #[arg(long, value_name = "TYPE")]
        source_type: Option<String>,
    },
    /// Mark a source as pruned
    Remove { name: String },
}

#[derive(Subcommand)]
pub enum EntryCommands {
    /// List entries, optionally filtered
    List {
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        signal: Option<String>,
        #[arg(long)]
        source: Option<String>,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
        /// Show only entries with no topic classification (no tagged-with edges)
        #[arg(long)]
        unclassified: bool,
    },
    /// Inspect a single entry by UUID
    Inspect { id: String },
    /// Update entry fields (state and/or signal)
    Update {
        id: String,
        #[arg(long)]
        state: Option<String>,
        #[arg(long)]
        signal: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Print the current configuration
    Show,
}
```

- [ ] **Step 9.2: Verify it compiles (main.rs will have unmatched arms — fix next)**

```bash
cargo build -p tankyu-cli 2>&1 | grep "error" | head -20
```

---

### Task 10: Source handlers

**Files:**
- Modify: `crates/tankyu-cli/src/commands/source.rs`

- [ ] **Step 10.1: Implement `inspect`, `add`, `remove`, and polish `list`**

Replace `source.rs` content with:

```rust
use anyhow::Result;
use tankyu_core::{
    domain::types::{SourceRole, SourceState, SourceType},
    features::source::{
        source_manager::AddSourceInput,
        url_detect::detect_source_type,
    },
};
use crate::context::AppContext;

fn parse_role(s: &str) -> Result<SourceRole> {
    match s {
        "starred" => Ok(SourceRole::Starred),
        "role-model" => Ok(SourceRole::RoleModel),
        "reference" => Ok(SourceRole::Reference),
        _ => Err(anyhow::anyhow!(
            "Invalid role '{s}'. Valid: starred, role-model, reference"
        )),
    }
}

fn parse_source_type(s: &str) -> Result<SourceType> {
    match s {
        "x-account" => Ok(SourceType::XAccount),
        "x-bookmarks" => Ok(SourceType::XBookmarks),
        "github-repo" => Ok(SourceType::GithubRepo),
        "github-releases" => Ok(SourceType::GithubReleases),
        "github-user" => Ok(SourceType::GithubUser),
        "blog" => Ok(SourceType::Blog),
        "rss-feed" => Ok(SourceType::RssFeed),
        "web-page" => Ok(SourceType::WebPage),
        "manual" => Ok(SourceType::Manual),
        "github-issues" => Ok(SourceType::GithubIssues),
        "agent-report" => Ok(SourceType::AgentReport),
        _ => Err(anyhow::anyhow!(
            "Invalid type '{s}'. Valid: x-account, x-bookmarks, github-repo, github-releases, \
             github-user, blog, rss-feed, web-page, manual, github-issues, agent-report"
        )),
    }
}

const fn role_str(r: Option<&SourceRole>) -> &'static str {
    match r {
        None => "—",
        Some(SourceRole::Starred) => "starred",
        Some(SourceRole::RoleModel) => "role-model",
        Some(SourceRole::Reference) => "reference",
    }
}

const fn type_str(t: &SourceType) -> &'static str {
    match t {
        SourceType::XAccount => "x-account",
        SourceType::XBookmarks => "x-bookmarks",
        SourceType::GithubRepo => "github-repo",
        SourceType::GithubReleases => "github-releases",
        SourceType::GithubUser => "github-user",
        SourceType::Blog => "blog",
        SourceType::RssFeed => "rss-feed",
        SourceType::WebPage => "web-page",
        SourceType::Manual => "manual",
        SourceType::GithubIssues => "github-issues",
        SourceType::AgentReport => "agent-report",
    }
}

const fn state_str(s: &SourceState) -> &'static str {
    match s {
        SourceState::Active => "active",
        SourceState::Stale => "stale",
        SourceState::Dormant => "dormant",
        SourceState::Pruned => "pruned",
    }
}

pub async fn list(ctx: &AppContext, topic: Option<&str>, role: Option<&str>) -> Result<()> {
    let sources = match (topic, role) {
        (Some(t), _) => {
            let topic = ctx
                .topic_mgr
                .get_by_name(t)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Topic '{t}' not found"))?;
            ctx.source_mgr.list_by_topic(topic.id).await?
        }
        (None, Some(r)) => ctx.source_mgr.list_by_role(parse_role(r)?).await?,
        (None, None) => ctx.source_mgr.list_all().await?,
    };
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&sources)?);
        return Ok(());
    }
    if sources.is_empty() {
        let hint = match (topic, role) {
            (Some(t), _) => format!(
                "No sources for \"{t}\". Add one with: tankyu source add <url> --topic {t}"
            ),
            (None, Some(r)) => format!("No {r} sources."),
            (None, None) => "No sources yet. Add one with: tankyu source add <url>".to_string(),
        };
        println!("{hint}");
        return Ok(());
    }
    let mut table = comfy_table::Table::new();
    table.set_header(["Name", "Type", "State", "Role"]);
    for s in &sources {
        table.add_row([
            s.name.as_str(),
            type_str(&s.r#type),
            state_str(&s.state),
            role_str(s.role.as_ref()),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub async fn inspect(ctx: &AppContext, name: &str) -> Result<()> {
    let s = ctx
        .source_mgr
        .get_by_name(name)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Source '{name}' not found"))?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&s)?);
        return Ok(());
    }
    println!("Name:         {}", s.name);
    println!("Type:         {}", type_str(&s.r#type));
    println!("State:        {}", state_str(&s.state));
    println!("Role:         {}", role_str(s.role.as_ref()));
    println!("URL:          {}", s.url);
    println!("Check count:  {}", s.check_count);
    println!("Hit count:    {}", s.hit_count);
    println!("Miss count:   {}", s.miss_count);
    println!(
        "Last checked: {}",
        s.last_checked_at
            .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "never".to_string())
    );
    println!("Created:      {}", s.created_at.format("%Y-%m-%d"));
    Ok(())
}

pub async fn add(
    ctx: &AppContext,
    url: &str,
    name: Option<&str>,
    topic: Option<&str>,
    role: Option<&str>,
    source_type: Option<&str>,
) -> Result<()> {
    let role = role.map(parse_role).transpose()?;
    let source_type = source_type.map(parse_source_type).transpose()?;
    let topic_id = if let Some(t) = topic {
        Some(
            ctx.topic_mgr
                .get_by_name(t)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Topic '{t}' not found"))?
                .id,
        )
    } else {
        None
    };
    let source = ctx
        .source_mgr
        .add(AddSourceInput {
            url: url.to_string(),
            name: name.map(str::to_string),
            source_type,
            role,
            topic_id,
        })
        .await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&source)?);
        return Ok(());
    }
    println!("Added source: {} ({})", source.name, type_str(&source.r#type));
    println!("  URL: {}", source.url);
    println!("  ID:  {}", source.id);
    if let Some(t) = topic {
        println!("  Linked to topic: {t}");
    }
    Ok(())
}

pub async fn remove(ctx: &AppContext, name: &str) -> Result<()> {
    let source = ctx.source_mgr.remove(name).await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&source)?);
        return Ok(());
    }
    println!("Removed source: {} (marked as pruned)", source.name);
    Ok(())
}
```

---

### Task 11: Topic `create` handler

**Files:**
- Modify: `crates/tankyu-cli/src/commands/topic.rs`

- [ ] **Step 11.1: Add `create()` to topic.rs**

Add after the existing `inspect()` function:

```rust
pub async fn create(
    ctx: &AppContext,
    name: &str,
    description: &str,
    tags_csv: &str,
) -> Result<()> {
    use tankyu_core::features::topic::topic_manager::CreateTopicInput;
    let tags: Vec<String> = tags_csv
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    let topic = ctx
        .topic_mgr
        .create(CreateTopicInput {
            name: name.to_string(),
            description: description.to_string(),
            tags: tags.clone(),
        })
        .await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&topic)?);
        return Ok(());
    }
    println!("Created topic: {} ({})", topic.name, topic.id);
    if !tags.is_empty() {
        println!("  Tags: {}", tags.join(", "));
    }
    Ok(())
}
```

---

### Task 12: Entry `update` and `--unclassified`

**Files:**
- Modify: `crates/tankyu-cli/src/commands/entry.rs`

- [ ] **Step 12.1: Add `update()` function to entry.rs**

Add after `inspect()`:

```rust
pub async fn update(
    ctx: &AppContext,
    id: &str,
    state: Option<&str>,
    signal: Option<&str>,
) -> Result<()> {
    use tankyu_core::domain::types::EntryUpdate;
    if state.is_none() && signal.is_none() {
        anyhow::bail!("At least one of --state or --signal must be provided");
    }
    let uuid = Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {id}"))?;
    let state_val = state.map(parse_state).transpose()?;
    let signal_val = signal.map(parse_signal).transpose()?;
    let entry = ctx
        .entry_mgr
        .update(uuid, EntryUpdate { state: state_val, signal: signal_val, summary: None })
        .await?;
    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&entry)?);
        return Ok(());
    }
    println!("Updated entry: {}", entry.title);
    if let Some(s) = state {
        println!("  State: {s}");
    }
    if let Some(s) = signal {
        println!("  Signal: {s}");
    }
    Ok(())
}
```

- [ ] **Step 12.2: Add `--unclassified` path to `list()`**

Update the `list()` function signature:

```rust
pub async fn list(
    ctx: &AppContext,
    state: Option<&str>,
    signal: Option<&str>,
    source: Option<&str>,
    topic: Option<&str>,
    limit: Option<usize>,
    unclassified: bool,      // ← new parameter
) -> Result<()> {
```

Add mutual exclusion check at the top of `list()`:

```rust
if unclassified && (source.is_some() || topic.is_some()) {
    anyhow::bail!("--unclassified is mutually exclusive with --topic and --source");
}
```

Add unclassified branch in the match:

```rust
let mut entries = if unclassified {
    use tankyu_core::domain::types::{EdgeType, GraphQuery, NodeType};
    let classified_ids: std::collections::HashSet<_> = ctx
        .graph_store
        .query(GraphQuery {
            edge_type: Some(EdgeType::TaggedWith),
            from_type: Some(NodeType::Entry),
            ..Default::default()
        })
        .await?
        .into_iter()
        .map(|e| e.from_id)
        .collect();
    let all = ctx.entry_mgr.list_all().await?;
    all.into_iter()
        .filter(|e| !classified_ids.contains(&e.id))
        .collect::<Vec<_>>()
} else {
    match (source, topic) {
        (Some(name), None) => {
            let src = ctx
                .source_mgr
                .get_by_name(name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Source '{name}' not found"))?;
            ctx.entry_mgr.list_by_source(src.id).await?
        }
        (None, Some(name)) => {
            let t = ctx
                .topic_mgr
                .get_by_name(name)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Topic '{name}' not found"))?;
            ctx.entry_mgr.list_by_topic(t.id).await?
        }
        (None, None) => ctx.entry_mgr.list_all().await?,
        (Some(_), Some(_)) => unreachable!("mutually exclusive guard above"),
    }
};
```

---

### Task 13: Health command handler

**Files:**
- Modify: `crates/tankyu-cli/src/commands/health.rs`

- [ ] **Step 13.1: Implement `health::run()`**

```rust
use anyhow::Result;
use tankyu_core::{
    domain::types::SourceType,
    features::health::{HealthThresholds, HealthWarningKind},
};

use crate::context::AppContext;

/// Maps `SourceType` to its kebab-case CLI string.
/// Duplicated from source.rs (YAGNI — don't abstract prematurely).
const fn source_type_str(t: &SourceType) -> &'static str {
    match t {
        SourceType::XAccount => "x-account",
        SourceType::XBookmarks => "x-bookmarks",
        SourceType::GithubRepo => "github-repo",
        SourceType::GithubReleases => "github-releases",
        SourceType::GithubUser => "github-user",
        SourceType::Blog => "blog",
        SourceType::RssFeed => "rss-feed",
        SourceType::WebPage => "web-page",
        SourceType::Manual => "manual",
        SourceType::GithubIssues => "github-issues",
        SourceType::AgentReport => "agent-report",
    }
}

pub async fn run(ctx: &AppContext) -> Result<()> {
    let thresholds = HealthThresholds {
        stale_days: ctx.config.stale_days,
        dormant_days: ctx.config.dormant_days,
    };
    let report = ctx.health_mgr.health(thresholds).await?;
    let healthy = report.ok;

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&report)?);
    } else if healthy {
        println!("All sources healthy");
    } else {
        let mut table = comfy_table::Table::new();
        table.set_header(["Kind", "Source", "Type", "Detail"]);
        for w in &report.warnings {
            let kind = match w.kind {
                HealthWarningKind::Stale => "stale",
                HealthWarningKind::Dormant => "dormant",
                HealthWarningKind::Empty => "empty",
            };
            table.add_row([kind, &w.source_name, source_type_str(&w.source_type), &w.detail]);
        }
        println!("{table}");
    }

    if !healthy {
        anyhow::bail!("health check failed: {} warning(s)", report.warnings.len());
    }
    Ok(())
}
```

---

### Task 14: Wire `main.rs` and fix build

**Files:**
- Modify: `crates/tankyu-cli/src/main.rs`

- [ ] **Step 14.1: Update imports and add all match arms**

**Important:** Add the new arms to the existing `match command` block. Do NOT remove the existing `Commands::Status`, `Commands::Config`, and `Commands::Doctor` arms — preserve them exactly.

```rust
use cli::{
    Cli, Commands, ConfigCommands, EntryCommands, SourceCommands, TopicCommands,
};

// In the match:
Commands::Health => commands::health::run(&ctx).await,

Commands::Topic { command } => match command {
    TopicCommands::List => commands::topic::list(&ctx).await,
    TopicCommands::Inspect { name } => commands::topic::inspect(&ctx, &name).await,
    TopicCommands::Create { name, description, tags } => {
        commands::topic::create(&ctx, &name, &description, &tags).await
    }
},

Commands::Source { command } => match command {
    SourceCommands::List { topic, role } => {
        commands::source::list(&ctx, topic.as_deref(), role.as_deref()).await
    }
    SourceCommands::Inspect { name } => commands::source::inspect(&ctx, &name).await,
    SourceCommands::Add { url, name, topic, role, source_type } => {
        commands::source::add(
            &ctx,
            &url,
            name.as_deref(),
            topic.as_deref(),
            role.as_deref(),
            source_type.as_deref(),
        )
        .await
    }
    SourceCommands::Remove { name } => commands::source::remove(&ctx, &name).await,
},

Commands::Entry { command } => match command {
    EntryCommands::List { state, signal, source, topic, limit, unclassified } => {
        commands::entry::list(
            &ctx,
            state.as_deref(),
            signal.as_deref(),
            source.as_deref(),
            topic.as_deref(),
            limit,
            unclassified,
        )
        .await
    }
    EntryCommands::Inspect { id } => commands::entry::inspect(&ctx, &id).await,
    EntryCommands::Update { id, state, signal } => {
        commands::entry::update(&ctx, &id, state.as_deref(), signal.as_deref()).await
    }
},
```

- [ ] **Step 14.2: Build and fix all errors**

```bash
cargo build -p tankyu-cli 2>&1
```

Fix any remaining compile errors before proceeding.

- [ ] **Step 14.3: Run full test suite**

```bash
cargo test --all
```

Expected: all existing tests pass; no regressions.

- [ ] **Step 14.4: Commit**

```bash
git add crates/tankyu-cli/src/
git commit -m "feat(cli): wire all new commands — source inspect/add/remove, topic create, entry update/unclassified, health"
```

---

## Chunk 5: BDD Acceptance Tests + Insta Snapshots

Full BDD coverage for all new commands. Step definitions build on the existing `TankyuWorld` pattern.

### Task 15: Extend `TankyuWorld` helpers

**Files:**
- Modify: `crates/tankyu-cli/tests/acceptance/world.rs`

- [ ] **Step 15.1: Add `write_source`, `write_topic`, `write_tagged_with_edge`**

```rust
pub fn write_source(
    &self,
    id: &str,
    name: &str,
    url: &str,
    state: &str,
    last_checked_days_ago: Option<i64>,
) {
    use chrono::{Duration, Utc};
    let last_checked = last_checked_days_ago
        .map(|d| (Utc::now() - Duration::days(d)).to_rfc3339())
        .map(serde_json::Value::String)
        .unwrap_or(serde_json::Value::Null);
    write_json(
        self.data_dir.path().join(format!("sources/{id}.json")),
        &serde_json::json!({
            "id": id,
            "type": "github-repo",
            "name": name,
            "url": url,
            "state": state,
            "config": null,
            "pollIntervalMinutes": null,
            "discoveredVia": null,
            "discoveryReason": null,
            "lastCheckedAt": last_checked,
            "lastNewContentAt": null,
            "checkCount": 0,
            "hitCount": 0,
            "missCount": 0,
            "createdAt": "2025-01-01T00:00:00Z"
        }),
    );
}

pub fn write_topic(&self, id: &str, name: &str) {
    write_json(
        self.data_dir.path().join(format!("topics/{id}.json")),
        &serde_json::json!({
            "id": id,
            "name": name,
            "description": "",
            "tags": [],
            "projects": [],
            "createdAt": "2025-01-01T00:00:00Z",
            "updatedAt": "2025-01-01T00:00:00Z",
            "lastScannedAt": null,
            "scanCount": 0
        }),
    );
}

pub fn write_tagged_with_edge(&self, entry_id: &str, topic_id: &str) {
    use uuid::Uuid;
    let edge_id = Uuid::new_v4().to_string();
    // Read current edges, append, write back
    let edges_path = self.data_dir.path().join("graph/edges.json");
    let current: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&edges_path).unwrap()).unwrap();
    let mut edges = current["edges"].as_array().cloned().unwrap_or_default();
    edges.push(serde_json::json!({
        "id": edge_id,
        "fromId": entry_id,
        "fromType": "entry",
        "toId": topic_id,
        "toType": "topic",
        "edgeType": "tagged-with",
        "reason": "test classification",
        "createdAt": "2025-01-01T00:00:00Z"
    }));
    write_json(
        &edges_path,
        &serde_json::json!({ "version": 1, "edges": edges }),
    );
}
```

- [ ] **Step 15.2: Verify helpers compile**

```bash
cargo build -p tankyu-cli 2>&1 | head -10
```

Expected: no errors (warnings are OK at this stage).

- [ ] **Step 15.3: Commit**

```bash
git add crates/tankyu-cli/tests/acceptance/world.rs
git commit -m "test(bdd): extend TankyuWorld — write_source, write_topic, write_tagged_with_edge helpers"
```

---

### Task 16: `source.feature` + step definitions

**Files:**
- Create: `crates/tankyu-cli/tests/acceptance/features/source.feature`
- Create: `crates/tankyu-cli/tests/acceptance/steps/source_steps.rs`
- Modify: `crates/tankyu-cli/tests/acceptance/steps/mod.rs`
- Modify: `crates/tankyu-cli/tests/acceptance/main.rs`

- [ ] **Step 16.1: Write `source.feature`**

```gherkin
Feature: Source management
  As a researcher
  I want to add, inspect, and remove sources
  So that I can manage the information sources I track

  Scenario: Inspect an existing source
    Given a source exists with name "rust-lang/rust" and URL "https://github.com/rust-lang/rust"
    When I run "source inspect rust-lang/rust"
    Then the command exits successfully
    And stdout contains "rust-lang/rust"
    And stdout contains "github-repo"
    And stdout contains "active"

  Scenario: Inspect a non-existent source fails
    When I run "source inspect does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"

  Scenario: Add a new GitHub repo source
    When I run "source add https://github.com/tokio-rs/tokio"
    Then the command exits successfully
    And stdout contains "tokio-rs/tokio"
    And stdout contains "github-repo"

  Scenario: Remove a source marks it as pruned
    Given a source exists with name "old-source" and URL "https://example.com/old"
    When I run "source remove old-source"
    Then the command exits successfully
    And stdout contains "marked as pruned"

  Scenario: Remove a non-existent source fails
    When I run "source remove does-not-exist"
    Then the command exits with failure
    And stderr contains "not found"

  Scenario: List sources shows empty hint when no sources exist
    When I run "source list"
    Then the command exits successfully
    And stdout contains "No sources yet"

  Scenario: List sources with topic filter shows empty hint
    Given a topic exists with name "Async Rust"
    When I run "source list --topic Async Rust"
    Then the command exits successfully
    And stdout contains "No sources for"

  Scenario: Add a duplicate source URL returns existing source
    Given a source exists with name "existing" and URL "https://github.com/rust-lang/rust"
    When I run "source add https://github.com/rust-lang/rust"
    Then the command exits successfully
    And stdout contains "Added source"

  Scenario: Add a source linked to a topic creates monitors edge
    Given a topic exists with name "Async Rust"
    When I run "source add https://github.com/tokio-rs/tokio --topic Async Rust"
    Then the command exits successfully
    And stdout contains "tokio-rs/tokio"
    And stdout contains "Linked to topic: Async Rust"
```

- [ ] **Step 16.2: Write `source_steps.rs`**

```rust
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::TankyuWorld;
use cucumber::{given, when};
use uuid::Uuid;

#[given(expr = "a source exists with name {string} and URL {string}")]
fn given_source(world: &mut TankyuWorld, name: String, url: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &url, "active", Some(1));
}

#[given(expr = "a topic exists with name {string}")]
fn given_topic(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_topic(&id, &name);
}
```

(Note: `when I run`, `then` steps are already defined in `entry_steps.rs` — reuse them.)

- [ ] **Step 16.3: Register step module**

In `steps/mod.rs`, add:
```rust
pub mod source_steps;
```

In `main.rs`, add to the `World::cucumber()` builder — cucumber auto-discovers steps from all registered modules.

- [ ] **Step 16.4: Run source BDD scenarios**

```bash
cargo test -p tankyu-cli --test acceptance 2>&1 | grep -E "(PASS|FAIL|scenario)"
```

Expected: all source scenarios pass.

- [ ] **Step 16.5: Commit**

```bash
git add crates/tankyu-cli/tests/acceptance/
git commit -m "test(bdd): source.feature — inspect, add, remove, list-empty scenarios"
```

---

### Task 17: `topic.feature` + step definitions

**Files:**
- Create: `crates/tankyu-cli/tests/acceptance/features/topic.feature`
- Create: `crates/tankyu-cli/tests/acceptance/steps/topic_steps.rs`

- [ ] **Step 17.1: Write `topic.feature`**

```gherkin
Feature: Topic management
  As a researcher
  I want to create and list research topics
  So that I can organize my research focus areas

  Scenario: Create a topic
    When I run "topic create Rust Async"
    Then the command exits successfully
    And stdout contains "Created topic: Rust Async"

  Scenario: Create a topic with tags
    When I run "topic create Systems Programming --tags rust,c,cpp"
    Then the command exits successfully
    And stdout contains "Tags: rust, c, cpp"

  Scenario: Create a duplicate topic fails
    Given a topic exists with name "Existing Topic"
    When I run "topic create Existing Topic"
    Then the command exits with failure
    And stderr contains "already exists"

  Scenario: List topics shows empty hint when no topics exist
    When I run "topic list"
    Then the command exits successfully
    And stdout does not contain "error"
```

- [ ] **Step 17.2: Register `topic_steps.rs`**

The `given_topic` step is already in `source_steps.rs`. Create `topic_steps.rs` as an empty module (or with topic-specific steps if needed later).

- [ ] **Step 17.3: Run topic BDD**

```bash
cargo test -p tankyu-cli --test acceptance 2>&1 | grep -E "topic"
```

- [ ] **Step 17.4: Commit**

```bash
git add crates/tankyu-cli/tests/acceptance/
git commit -m "test(bdd): topic.feature — create, create-with-tags, duplicate, list-empty"
```

---

### Task 18: `health.feature` + step definitions

**Files:**
- Create: `crates/tankyu-cli/tests/acceptance/features/health.feature`
- Create: `crates/tankyu-cli/tests/acceptance/steps/health_steps.rs`

- [ ] **Step 18.1: Write `health.feature`**

```gherkin
Feature: Source health checking
  As a researcher
  I want to know which sources are stale, dormant, or empty
  So that I can maintain the quality of my research pipeline

  Scenario: All sources healthy exits 0
    Given a source exists with name "fresh-source" checked 1 day ago with entries
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Never-checked source produces stale warning
    Given a source exists with name "never-checked" that has never been checked
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
    And stdout contains "never checked"

  Scenario: Stale source produces warning and exits 1
    Given a source exists with name "stale-source" last checked 10 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "stale"
    And stdout contains "stale-source"

  Scenario: Dormant source produces warning and exits 1
    Given a source exists with name "dormant-source" last checked 35 days ago
    When I run "health"
    Then the command exits with failure
    And stdout contains "dormant"
    And stdout contains "dormant-source"

  Scenario: Empty source produces warning and exits 1
    Given a source exists with name "empty-source" that has no entries
    When I run "health"
    Then the command exits with failure
    And stdout contains "empty"

  Scenario: Pruned source is ignored
    Given a pruned source exists with name "pruned-source"
    When I run "health"
    Then the command exits successfully
    And stdout contains "All sources healthy"

  Scenario: Health report as JSON
    Given a source exists with name "fresh-source" checked 1 day ago with entries
    When I run "health --json"
    Then the command exits successfully
    And stdout contains "\"ok\":"
```

- [ ] **Step 18.2: Add `write_entry_for_source` helper to `world.rs`**

In `world.rs`, add:

```rust
/// Write an entry whose sourceId matches `source_id` exactly.
/// Used by health steps where the "has entries" check depends on source linkage.
pub fn write_entry_for_source(&self, source_id: &str) {
    let id = uuid::Uuid::new_v4().to_string();
    write_json(
        self.data_dir.path().join(format!("entries/{id}.json")),
        &serde_json::json!({
            "id": id,
            "sourceId": source_id,
            "type": "article",
            "title": "test entry",
            "url": format!("https://example.com/{id}"),
            "summary": null,
            "contentHash": null,
            "state": "new",
            "signal": null,
            "scannedAt": "2025-01-15T10:00:00Z",
            "metadata": null,
            "createdAt": "2025-01-15T10:00:00Z"
        }),
    );
}
```

- [ ] **Step 18.3: Write `health_steps.rs`**

```rust
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::TankyuWorld;
use cucumber::given;
use uuid::Uuid;

#[given(expr = "a source exists with name {string} checked {int} day(s) ago with entries")]
fn given_source_with_entries(world: &mut TankyuWorld, name: String, days: i64) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &format!("https://example.com/{name}"), "active", Some(days));
    world.write_entry_for_source(&id);
}

#[given(expr = "a source exists with name {string} that has never been checked")]
fn given_never_checked_source(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &format!("https://example.com/{name}"), "active", None);
}

#[given(expr = "a source exists with name {string} last checked {int} days ago")]
fn given_stale_source(world: &mut TankyuWorld, name: String, days: i64) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &format!("https://example.com/{name}"), "active", Some(days));
}

#[given(expr = "a source exists with name {string} that has no entries")]
fn given_empty_source(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &format!("https://example.com/{name}"), "active", Some(1));
    // No entries written — source is empty by design
}

#[given(expr = "a pruned source exists with name {string}")]
fn given_pruned_source(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &format!("https://example.com/{name}"), "pruned", None);
}
```

- [ ] **Step 18.3: Run health BDD**

```bash
cargo test -p tankyu-cli --test acceptance 2>&1 | grep -E "health"
```

- [ ] **Step 18.4: Register step module and run health BDD**

In `steps/mod.rs`, add:
```rust
pub mod health_steps;
```

Then run:
```bash
cargo test -p tankyu-cli --test acceptance 2>&1 | grep -E "health"
```

Expected: all health scenarios pass.

- [ ] **Step 18.5: Commit**

```bash
git add crates/tankyu-cli/tests/acceptance/
git commit -m "test(bdd): health.feature — healthy/stale/dormant/never-checked/empty/pruned/json scenarios"
```

---

### Task 19: Entry BDD extensions

**Files:**
- Modify: `crates/tankyu-cli/tests/acceptance/features/entry.feature`
- Modify: `crates/tankyu-cli/tests/acceptance/steps/entry_steps.rs`

- [ ] **Step 19.1: Add scenarios to `entry.feature`**

```gherkin
  Scenario: Update entry state
    Given the data directory contains 3 entries with mixed state
    When I run "entry update aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa --state read"
    Then the command exits successfully
    And stdout contains "Updated entry"
    And stdout contains "read"

  Scenario: Update entry signal
    Given the data directory contains 3 entries with mixed state
    When I run "entry update aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa --signal high"
    Then the command exits successfully
    And stdout contains "high"

  Scenario: Update non-existent entry fails
    When I run "entry update 00000000-0000-0000-0000-000000000000 --state read"
    Then the command exits with failure
    And stderr contains "not found"

  Scenario: Update without flags fails
    When I run "entry update aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa"
    Then the command exits with failure
    And stderr contains "at least one"

  Scenario: List unclassified entries excludes classified entries
    Given the data directory contains 3 entries with mixed state
    And entry "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa" is classified under topic "11111111-1111-1111-1111-111111111111"
    When I run "entry list --unclassified"
    Then the command exits successfully
    And stdout does not contain "Alpha entry"

  Scenario: List unclassified returns all entries when none classified
    Given the data directory contains 3 entries with mixed state
    When I run "entry list --unclassified"
    Then the command exits successfully
    And stdout contains "Alpha entry"
    And stdout contains "Beta entry"
```

- [ ] **Step 19.2: Add step definitions to `entry_steps.rs`**

```rust
#[given(expr = "entry {string} is classified under topic {string}")]
fn given_entry_classified(world: &mut TankyuWorld, entry_id: String, topic_id: String) {
    // Write the topic file too, so the topic exists
    world.write_topic(&topic_id, "Test Topic");
    world.write_tagged_with_edge(&entry_id, &topic_id);
}
```

- [ ] **Step 19.3: Run full BDD suite**

```bash
cargo test -p tankyu-cli --test acceptance
```

Expected: all scenarios pass.

- [ ] **Step 19.4: Commit**

```bash
git add crates/tankyu-cli/tests/acceptance/
git commit -m "test(bdd): extend entry.feature — update, unclassified scenarios"
```

---

### Task 20: Insta snapshots

**Files:**
- Modify: `crates/tankyu-cli/tests/cli_source.rs`
- Modify: `crates/tankyu-cli/tests/cli_topic.rs`
- Modify: `crates/tankyu-cli/tests/cli_entry.rs`
- Create: `crates/tankyu-cli/tests/cli_health.rs`

- [ ] **Step 20.1: Add source snapshots to `cli_source.rs`**

```rust
#[test]
fn source_inspect_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "inspect", "rust-lang/rust"])
        .output()
        .unwrap();
    assert!(out.status.success(), "source inspect failed: {}", String::from_utf8_lossy(&out.stderr));
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn source_inspect_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "source", "inspect", "rust-lang/rust"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["name"], "rust-lang/rust");
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn source_add_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "add", "https://github.com/tokio-rs/tokio"])
        .output()
        .unwrap();
    assert!(out.status.success(), "source add failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("tokio-rs/tokio"), "expected source name in output: {stdout}");
    // Dynamic ID — strip the ID line before snapshotting
    let stable: String = stdout.lines()
        .filter(|l| !l.trim_start().starts_with("ID:"))
        .collect::<Vec<_>>()
        .join("\n") + "\n";
    insta::assert_snapshot!(stable);
}

#[test]
fn source_list_empty_plain() {
    use tempfile::TempDir;
    use common::write_json;
    let dir = TempDir::new().unwrap();
    let b = dir.path();
    std::fs::create_dir_all(b.join("topics")).unwrap();
    std::fs::create_dir_all(b.join("sources")).unwrap();
    std::fs::create_dir_all(b.join("entries")).unwrap();
    std::fs::create_dir_all(b.join("graph")).unwrap();
    write_json(b.join("config.json"), &serde_json::json!({
        "version": 1, "defaultScanLimit": 20, "staleDays": 7,
        "dormantDays": 30, "llmClassify": false, "localRepoPaths": {}
    }));
    write_json(b.join("graph/edges.json"), &serde_json::json!({ "version": 1, "edges": [] }));
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
```

- [ ] **Step 20.2: Add topic snapshots to `cli_topic.rs`**

```rust
#[test]
fn topic_create_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["topic", "create", "New Topic"])
        .output()
        .unwrap();
    assert!(out.status.success(), "topic create failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Created topic: New Topic"));
    // Strip dynamic UUID from snapshot
    let stable: String = stdout.lines()
        .map(|l| {
            if l.contains("Created topic:") {
                l.split('(').next().unwrap_or(l).trim_end().to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n") + "\n";
    insta::assert_snapshot!(stable);
}

#[test]
fn topic_create_with_tags_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["topic", "create", "Systems", "--tags", "rust,c,cpp"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Tags: rust, c, cpp"));
    insta::assert_snapshot!(stdout.lines()
        .filter(|l| !l.contains('('))  // strip UUID line
        .collect::<Vec<_>>()
        .join("\n") + "\n");
}

#[test]
fn topic_create_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "topic", "create", "JSON Topic"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["name"], "JSON Topic");
    assert_eq!(v["scanCount"], 0);
    // Snapshot the structural shape (id is dynamic — use assert_eq on fields only)
    insta::assert_snapshot!(format!("name={} tags=[] scanCount=0", v["name"]));
}
```

- [ ] **Step 20.3: Add entry snapshots to `cli_entry.rs`**

```rust
#[test]
fn entry_update_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "update", ENTRY_ID, "--state", "read"])
        .output()
        .unwrap();
    assert!(out.status.success(), "entry update failed: {}", String::from_utf8_lossy(&out.stderr));
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_unclassified_plain() {
    // Fixture has 1 entry with no tagged-with edges → it is unclassified
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--unclassified"])
        .output()
        .unwrap();
    assert!(out.status.success(), "entry list --unclassified failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("feat: add entry management"), "fixture entry must be unclassified: {stdout}");
    insta::assert_snapshot!(stdout);
}
```

- [ ] **Step 20.4: Create `cli_health.rs`**

```rust
mod common;
use common::{cmd, create_fixture, write_json, SOURCE_ID};

/// Patch the fixture's source to have a recent `lastCheckedAt` so health passes.
/// Uses a fixed recent-enough date string (not dynamic) for snapshot stability.
fn patch_source_recently_checked(dir: &tempfile::TempDir) {
    write_json(
        dir.path().join(format!("sources/{SOURCE_ID}.json")),
        &serde_json::json!({
            "id": SOURCE_ID, "type": "github-repo", "name": "rust-lang/rust",
            "url": "https://github.com/rust-lang/rust", "state": "active",
            "discoveredVia": null, "discoveryReason": null,
            // Use a date 1 day before the fixed "now" in the fixture (2025-01-15).
            // The fixture uses 2025-01-15 for entry scannedAt — use 2025-01-14 here.
            "lastCheckedAt": "2025-01-14T10:00:00Z", "lastNewContentAt": null,
            "checkCount": 1, "hitCount": 0, "missCount": 0,
            "createdAt": "2025-01-01T00:00:00Z"
        }),
    );
}

#[test]
fn health_all_healthy_plain() {
    // Note: this test is sensitive to the system clock because "N days ago"
    // is computed at runtime. Instead of snapshotting the full output, we
    // verify the key assertion and snapshot the stable parts.
    let dir = create_fixture();
    patch_source_recently_checked(&dir);
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["health"])
        .output()
        .unwrap();
    assert!(out.status.success(), "health should exit 0 when healthy: {}", String::from_utf8_lossy(&out.stderr));
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn health_never_checked_stale_plain() {
    // The default fixture source has lastCheckedAt: null → stale (never checked).
    // "never checked" detail is stable (not time-dependent).
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["health"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "health should exit 1 when there are stale sources");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("stale"), "expected stale warning: {stdout}");
    assert!(stdout.contains("never checked"), "expected never-checked detail: {stdout}");
    insta::assert_snapshot!(stdout);
}

#[test]
fn health_json() {
    let dir = create_fixture();
    patch_source_recently_checked(&dir);
    let out = cmd(&dir)
        .args(["--json", "health"])
        .output()
        .unwrap();
    assert!(out.status.success(), "health --json should exit 0 when healthy");
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["warnings"].as_array().unwrap().is_empty());
    // Snapshot the JSON (checkedAt is dynamic — strip it before snapshotting)
    let mut stable = v.clone();
    stable["checkedAt"] = serde_json::Value::String("<dynamic>".to_string());
    insta::assert_snapshot!(serde_json::to_string_pretty(&stable).unwrap());
}
```

- [ ] **Step 20.5: Run and accept snapshots**

```bash
cargo test -p tankyu-cli 2>&1 | tail -20
cargo insta review
```

Review each snapshot and accept. Re-run to confirm all pass:

```bash
cargo test -p tankyu-cli
```

- [ ] **Step 20.6: Commit snapshots**

```bash
git add crates/tankyu-cli/tests/
git commit -m "test(snapshots): insta snapshots for source/topic/entry/health commands"
```

---

### Task 21: Final verification

- [ ] **Step 21.1: Run full test suite**

```bash
cargo test --all
```

Expected: all tests pass, 0 failures.

- [ ] **Step 21.2: Run clippy**

```bash
cargo clippy -- -D warnings
```

Expected: no warnings.

- [ ] **Step 21.3: Run fmt check**

```bash
cargo fmt --check
```

If any formatting issues: `cargo fmt && git add -A && git commit -m "style: cargo fmt"`

- [ ] **Step 21.4: Verify coverage gate (optional, local)**

```bash
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
cargo llvm-cov report --lcov lcov.info 2>&1 | grep "TOTAL"
```

Expected: ≥80% line coverage.

- [ ] **Step 21.5: Invoke finishing skill**

Use `superpowers:finishing-a-development-branch` to create the PR.
