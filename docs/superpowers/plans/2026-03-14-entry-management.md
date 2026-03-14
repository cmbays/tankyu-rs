# Entry Management Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `entry list` and `entry inspect` CLI commands backed by a new `EntryManager` feature class, with full BDD/TDD test coverage and three bundled bug fixes.

**Architecture:** Hexagonal (ports & adapters). `EntryManager` wraps `IEntryStore` + `IGraphStore` — same pattern as `SourceManager`. All filtering is in-memory at the manager layer. The CLI commands are thin async handlers that call the manager and format output.

**Tech Stack:** Rust, Cargo workspace, tokio, clap 4, comfy-table, insta (snapshots), assert_cmd (e2e), cucumber 0.22 (BDD), cargo-mutants (mutation testing).

**Spec:** `docs/superpowers/specs/2026-03-14-entry-management-design.md`
**TDD Reference:** `TDD_WORKFLOW.md` — read it before starting.

---

## File Map

### Files to create
| File | Purpose |
|------|---------|
| `crates/tankyu-core/src/features/entry/mod.rs` | Re-exports `EntryManager` |
| `crates/tankyu-core/src/features/entry/entry_manager.rs` | `EntryManager` struct + unit tests |
| `crates/tankyu-cli/src/commands/entry.rs` | `list` and `inspect` CLI handlers |
| `crates/tankyu-cli/tests/cli_entry.rs` | Insta snapshot tests |
| `crates/tankyu-cli/tests/acceptance/main.rs` | Cucumber test binary entry point |
| `crates/tankyu-cli/tests/acceptance/world.rs` | `TankyuWorld` shared state |
| `crates/tankyu-cli/tests/acceptance/steps/mod.rs` | Step definition module |
| `crates/tankyu-cli/tests/acceptance/steps/entry_steps.rs` | BDD step definitions |
| `crates/tankyu-cli/tests/acceptance/features/entry.feature` | Gherkin scenarios |

### Files to modify
| File | Change |
|------|--------|
| `crates/tankyu-core/src/features/mod.rs` | Add `pub mod entry;` |
| `crates/tankyu-core/src/features/source/source_manager.rs` | Add `get_by_name()` method + test |
| `crates/tankyu-core/src/features/topic/topic_manager.rs` | Rename `list()` → `list_all()` |
| `crates/tankyu-cli/src/context.rs` | Fix `graph_store` clone; add `entry_mgr: EntryManager` |
| `crates/tankyu-cli/src/commands/mod.rs` | Add `pub mod entry;` |
| `crates/tankyu-cli/src/commands/status.rs` | Call `ctx.entry_mgr.list_all()` |
| `crates/tankyu-cli/src/commands/topic.rs` | Call `ctx.topic_mgr.list_all()` |
| `crates/tankyu-cli/src/commands/doctor.rs` | Replace `ctx.entry_store.list()` with `ctx.entry_mgr.list_all()` |
| `crates/tankyu-cli/src/cli.rs` | Add `EntryCommands` enum + `Entry` variant |
| `crates/tankyu-cli/src/main.rs` | Add `EntryCommands` import + match arm |
| `crates/tankyu-cli/tests/common/mod.rs` | Add `ENTRY_ID` constant + entry fixture JSON |
| `crates/tankyu-cli/tests/cmd_tests.rs` | Add `entry_list_*` and `entry_inspect_*` tests |
| `Cargo.toml` (workspace) | Add `cucumber = "0.22"` to workspace deps |
| `crates/tankyu-cli/Cargo.toml` | Add cucumber dev-dep + `[[test]]` acceptance binary |

### Pre-existing fixtures (already committed — do not recreate)
| File | Notes |
|------|-------|
| `crates/tankyu-core/tests/fixtures/entries/00000000-0000-0000-0000-000000000003.json` | Entry fixture used by `entry_fixture_parses` in `store_compat.rs`; title = `"Stabilize feature X"`. Verify it passes before starting Task 4. |

---

## Chunk 1: Infrastructure Fixes

These three fixes are prerequisites for everything else. They are pure modifications — no new files. Each fix is a separate commit.

### Task 1: Fix `graph_store` Arc clone in `AppContext`

**Files:**
- Modify: `crates/tankyu-cli/src/context.rs`

**Background:** `graph_store` is currently moved into `SourceManager::new()`, making it unavailable for `EntryManager`. Fix it now so the later `EntryManager` wiring step has what it needs.

- [ ] **Step 1.1: Write a compile-time proof test**

In `crates/tankyu-cli/src/context.rs`, in the `AppContext::new` body, temporarily add a line after the `SourceManager::new(...)` line:

```rust
let _ = Arc::clone(&graph_store); // will fail to compile — proves graph_store was moved
```

- [ ] **Step 1.2: Run the build to confirm the error**

```bash
cargo build -p tankyu-cli 2>&1 | head -20
```

Expected: compile error — `use of moved value: graph_store`

- [ ] **Step 1.3: Remove the proof line, apply the fix**

In `crates/tankyu-cli/src/context.rs`, change line 49 from:

```rust
source_mgr: SourceManager::new(source_store, graph_store),
```

to:

```rust
source_mgr: SourceManager::new(source_store, Arc::clone(&graph_store)),
```

Leave `graph_store` in scope (do not drop it — it will be used by `EntryManager` in Task 6).

- [ ] **Step 1.4: Verify it compiles**

```bash
cargo build -p tankyu-cli
```

Expected: success (no errors, no warnings about unused `graph_store` — it'll be used in Task 6; if clippy warns about the unused binding, prefix it with an underscore: `let _graph_store = graph_store;` temporarily, and remove the underscore in Task 6 when it is consumed by `EntryManager::new`).

- [ ] **Step 1.5: Run tests to confirm nothing broke**

```bash
cargo test --all
```

Expected: all tests pass (same count as before).

- [ ] **Step 1.6: Commit**

```bash
git add crates/tankyu-cli/src/context.rs
git commit -m "fix(context): Arc::clone graph_store instead of moving into SourceManager"
```

---

### Task 2: Rename `TopicManager::list()` → `list_all()`

**Files:**
- Modify: `crates/tankyu-core/src/features/topic/topic_manager.rs` (3 occurrences)
- Modify: `crates/tankyu-cli/src/commands/topic.rs` (1 occurrence)
- Modify: `crates/tankyu-cli/src/commands/status.rs` (1 occurrence)

**Background:** `TopicManager::list()` conflicts with the naming convention used by `SourceManager::list_all()`. Unify to `list_all()` before adding `EntryManager::list_all()`.

- [ ] **Step 2.1: Search for all call sites**

```bash
cargo grep "topic_mgr\.list\(\)" 2>/dev/null || grep -r "topic_mgr\.list()" crates/
```

Expected output shows hits in `commands/topic.rs` and `commands/status.rs`.

```bash
grep -n "\.list(" crates/tankyu-core/src/features/topic/topic_manager.rs
```

Expected: hits in the `pub async fn list` signature and in the test assertions.

- [ ] **Step 2.2: Rename the method signature in `topic_manager.rs`**

In `crates/tankyu-core/src/features/topic/topic_manager.rs`, change:

```rust
    pub async fn list(&self) -> Result<Vec<Topic>> {
```

to:

```rust
    pub async fn list_all(&self) -> Result<Vec<Topic>> {
```

- [ ] **Step 2.3: Update the test assertion in `topic_manager.rs`**

In the `#[cfg(test)]` module of the same file, find:

```rust
        assert_eq!(mgr.list().await.unwrap().len(), 2);
```

Change to:

```rust
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
```

- [ ] **Step 2.4: Update `commands/topic.rs`**

In `crates/tankyu-cli/src/commands/topic.rs`, change line 6:

```rust
    let topics = ctx.topic_mgr.list().await?;
```

to:

```rust
    let topics = ctx.topic_mgr.list_all().await?;
```

- [ ] **Step 2.5: Update `commands/status.rs`**

In `crates/tankyu-cli/src/commands/status.rs`, change line 7:

```rust
    let topics = ctx.topic_mgr.list().await?;
```

to:

```rust
    let topics = ctx.topic_mgr.list_all().await?;
```

- [ ] **Step 2.6: Verify no remaining `list()` call sites**

```bash
grep -rn "topic_mgr\.list()" crates/
```

Expected: no output (all sites updated).

- [ ] **Step 2.7: Run tests**

```bash
cargo test --all
```

Expected: all tests pass.

- [ ] **Step 2.8: Commit**

```bash
git add crates/tankyu-core/src/features/topic/topic_manager.rs \
        crates/tankyu-cli/src/commands/topic.rs \
        crates/tankyu-cli/src/commands/status.rs
git commit -m "refactor(topic): rename TopicManager::list() to list_all() for consistency"
```

---

### Task 3: Add `SourceManager::get_by_name()`

**Files:**
- Modify: `crates/tankyu-core/src/features/source/source_manager.rs`

**Background:** `--source <name>` in `entry list` needs to resolve a source name to a UUID. `ISourceStore` only has `get_by_url()`, so name lookup lives at the manager layer as a linear scan.

- [ ] **Step 3.1: Write the failing unit test first**

In `crates/tankyu-core/src/features/source/source_manager.rs`, inside the `#[cfg(test)] mod tests` block (after the last test), add:

```rust
    #[tokio::test]
    async fn test_get_by_name_found() {
        let target_id = Uuid::new_v4();
        let mut target = make_source(target_id, None);
        target.name = "rust-lang/rust".to_string();
        let store = Arc::new(StubSourceStore {
            sources: vec![target, make_source(Uuid::new_v4(), None)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.get_by_name("rust-lang/rust").await.unwrap();
        assert_eq!(result.unwrap().id, target_id);
    }

    #[tokio::test]
    async fn test_get_by_name_not_found() {
        let store = Arc::new(StubSourceStore { sources: vec![] });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = SourceManager::new(store, graph);
        let result = mgr.get_by_name("nonexistent").await.unwrap();
        assert!(result.is_none());
    }
```

- [ ] **Step 3.2: Run the tests to confirm RED**

```bash
cargo test -p tankyu-core test_get_by_name
```

Expected: compile error — `no method named 'get_by_name' found for struct 'SourceManager'`

- [ ] **Step 3.3: Implement `get_by_name` in `SourceManager`**

In `crates/tankyu-core/src/features/source/source_manager.rs`, after the `list_by_topic` method (before the closing `}` of the `impl` block), add:

```rust
    /// Find a source by its name (case-sensitive, first match).
    ///
    /// Returns `None` if no source with that name exists.
    /// Note: name uniqueness is not enforced; this returns the first match.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn get_by_name(&self, name: &str) -> Result<Option<Source>> {
        let all = self.store.list().await?;
        Ok(all.into_iter().find(|s| s.name == name))
    }
```

- [ ] **Step 3.4: Run the tests to confirm GREEN**

```bash
cargo test -p tankyu-core test_get_by_name
```

Expected: 2 tests pass.

- [ ] **Step 3.5: Run full suite**

```bash
cargo test --all
```

Expected: all tests pass.

- [ ] **Step 3.6: Commit**

```bash
git add crates/tankyu-core/src/features/source/source_manager.rs
git commit -m "feat(source): add SourceManager::get_by_name() for --source filter resolution"
```

---

## Chunk 2: EntryManager Core

**Prerequisite check:** Before starting this chunk, verify the core entry fixture test passes:

```bash
cargo test -p tankyu-core entry_fixture_parses
```

Expected: 1 test passes. If it fails, the fixture file at `crates/tankyu-core/tests/fixtures/entries/00000000-0000-0000-0000-000000000003.json` is missing or malformed — inspect it before proceeding.

### Task 4: Create `EntryManager` with unit tests

**Files:**
- Create: `crates/tankyu-core/src/features/entry/entry_manager.rs`
- Create: `crates/tankyu-core/src/features/entry/mod.rs`

**Background:** `EntryManager` is the domain service that wraps `IEntryStore` + `IGraphStore`. Follow the `SourceManager` pattern exactly. Write ALL unit tests before writing ANY implementation code.

- [ ] **Step 4.1: Create the module file and wire it into the crate**

Create `crates/tankyu-core/src/features/entry/mod.rs` with:

```rust
pub mod entry_manager;
pub use entry_manager::EntryManager;
```

Then open `crates/tankyu-core/src/features/mod.rs` and add `pub mod entry;` (alphabetically before `source`):

```rust
pub mod entry;
pub mod source;
pub mod topic;
```

**Why now:** Rust only compiles modules that are reachable from the crate root. Without this line, `entry_manager.rs` is an orphan — `cargo test -p tankyu-core entry_manager` will find zero tests, not the compile error expected in Step 4.3. Task 5 (which previously added this line) is now removed since the work is done here.

Verify the crate builds before writing any tests:

```bash
cargo build -p tankyu-core
```

Expected: success (the new empty module compiles cleanly).

- [ ] **Step 4.2: Write the failing tests first**

Create `crates/tankyu-core/src/features/entry/entry_manager.rs` with ONLY the test module (no `EntryManager` struct yet):

```rust
#[cfg(test)]
mod tests {
    use crate::domain::{
        ports::{IEntryStore, IGraphStore},
        types::{
            Edge, EdgeType, Entry, EntryState, EntryType, EntryUpdate, GraphQuery, NodeType,
            Signal,
        },
    };
    use anyhow::Result;
    use async_trait::async_trait;
    use chrono::Utc;
    use std::sync::Arc;
    use uuid::Uuid;

    // ── Stubs ─────────────────────────────────────────────────────────────────

    struct StubEntryStore {
        entries: Vec<Entry>,
    }

    #[async_trait]
    impl IEntryStore for StubEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> {
            unimplemented!()
        }
        async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
            Ok(self.entries.iter().find(|e| e.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Entry>> {
            unimplemented!()
        }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> {
            unimplemented!()
        }
        async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>> {
            Ok(self
                .entries
                .iter()
                .filter(|e| e.source_id == source_id)
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

    struct StubGraphStore {
        edges: Vec<Edge>,
    }

    #[async_trait]
    impl IGraphStore for StubGraphStore {
        async fn add_edge(&self, _e: Edge) -> Result<()> {
            unimplemented!()
        }
        async fn remove_edge(&self, _id: Uuid) -> Result<()> {
            unimplemented!()
        }
        async fn get_edges_by_node(&self, node_id: Uuid) -> Result<Vec<Edge>> {
            Ok(self
                .edges
                .iter()
                .filter(|e| e.from_id == node_id || e.to_id == node_id)
                .cloned()
                .collect())
        }
        async fn get_neighbors(
            &self,
            _id: Uuid,
            _et: Option<EdgeType>,
        ) -> Result<Vec<Edge>> {
            unimplemented!()
        }
        async fn query(&self, _opts: GraphQuery) -> Result<Vec<Edge>> {
            unimplemented!()
        }
        async fn list(&self) -> Result<Vec<Edge>> {
            Ok(self.edges.clone())
        }
    }

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn make_entry(title: &str, state: EntryState) -> Entry {
        Entry {
            id: Uuid::new_v4(),
            source_id: Uuid::new_v4(),
            r#type: EntryType::Article,
            title: title.to_string(),
            url: format!("https://example.com/{title}"),
            summary: None,
            content_hash: None,
            state,
            signal: None,
            scanned_at: Utc::now(),
            metadata: None,
            created_at: Utc::now(),
        }
    }

    fn make_entry_with_signal(title: &str, signal: Signal) -> Entry {
        let mut e = make_entry(title, EntryState::New);
        e.signal = Some(signal);
        e
    }

    fn make_monitors_edge(topic_id: Uuid, source_id: Uuid) -> Edge {
        Edge {
            id: Uuid::new_v4(),
            from_id: topic_id,
            from_type: NodeType::Topic,
            to_id: source_id,
            to_type: NodeType::Source,
            edge_type: EdgeType::Monitors,
            reason: "test".to_string(),
            score: None,
            method: None,
            created_at: Utc::now(),
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    use super::EntryManager;

    #[tokio::test]
    async fn test_list_all_returns_all_entries() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry("Alpha", EntryState::New),
                make_entry("Beta", EntryState::Read),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        assert_eq!(mgr.list_all().await.unwrap().len(), 2);
    }

    #[tokio::test]
    async fn test_list_all_empty_returns_empty() {
        let store = Arc::new(StubEntryStore { entries: vec![] });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        assert!(mgr.list_all().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_list_by_state_returns_only_matching() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry("New one", EntryState::New),
                make_entry("Read one", EntryState::Read),
                make_entry("Also new", EntryState::New),
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_state(EntryState::New).await.unwrap();
        assert_eq!(result.len(), 2);
        assert!(result.iter().all(|e| e.state == EntryState::New));
    }

    #[tokio::test]
    async fn test_list_by_state_excludes_none_matching() {
        let store = Arc::new(StubEntryStore {
            entries: vec![make_entry("Read one", EntryState::Read)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_state(EntryState::New).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_signal_returns_only_matching() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry_with_signal("High one", Signal::High),
                make_entry_with_signal("Low one", Signal::Low),
                make_entry("No signal", EntryState::New), // signal == None
            ],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_signal(Signal::High).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].title, "High one");
    }

    #[tokio::test]
    async fn test_list_by_signal_excludes_none_signal() {
        let store = Arc::new(StubEntryStore {
            entries: vec![make_entry("No signal", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_signal(Signal::High).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_list_by_source_returns_only_matching() {
        let source_id = Uuid::new_v4();
        let mut entry_for_source = make_entry("For source", EntryState::New);
        entry_for_source.source_id = source_id;
        let store = Arc::new(StubEntryStore {
            entries: vec![entry_for_source.clone(), make_entry("Other", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_source(source_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, entry_for_source.id);
    }

    #[tokio::test]
    async fn test_list_by_topic_returns_entries_from_monitored_sources() {
        let topic_id = Uuid::new_v4();
        let source_id = Uuid::new_v4();
        let unrelated_source_id = Uuid::new_v4();

        let mut e1 = make_entry("Monitored entry", EntryState::New);
        e1.source_id = source_id;
        let mut e2 = make_entry("Unrelated entry", EntryState::New);
        e2.source_id = unrelated_source_id;

        let store = Arc::new(StubEntryStore {
            entries: vec![e1.clone(), e2],
        });
        let graph = Arc::new(StubGraphStore {
            edges: vec![make_monitors_edge(topic_id, source_id)],
        });
        let mgr = EntryManager::new(store, graph);

        let result = mgr.list_by_topic(topic_id).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, e1.id);
    }

    #[tokio::test]
    async fn test_list_by_topic_empty_graph_returns_empty() {
        let store = Arc::new(StubEntryStore {
            entries: vec![make_entry("Some entry", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.list_by_topic(Uuid::new_v4()).await.unwrap();
        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_returns_entry_by_id() {
        let entry = make_entry("Target", EntryState::New);
        let entry_id = entry.id;
        let store = Arc::new(StubEntryStore {
            entries: vec![entry, make_entry("Other", EntryState::New)],
        });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.get(entry_id).await.unwrap();
        assert_eq!(result.unwrap().id, entry_id);
    }

    #[tokio::test]
    async fn test_get_returns_none_for_missing_id() {
        let store = Arc::new(StubEntryStore { entries: vec![] });
        let graph = Arc::new(StubGraphStore { edges: vec![] });
        let mgr = EntryManager::new(store, graph);
        let result = mgr.get(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }
}
```

- [ ] **Step 4.3: Run the tests to confirm RED**

```bash
cargo test -p tankyu-core entry_manager
```

Expected: compile error — `unresolved import 'super::EntryManager'` (the struct doesn't exist yet). That is the correct RED state.

- [ ] **Step 4.4: Add the `EntryManager` struct and `use` declarations above the test module**

At the top of `entry_manager.rs` (before the `#[cfg(test)]` block), add:

```rust
use std::sync::Arc;

use anyhow::Result;
use uuid::Uuid;

use crate::domain::{
    ports::{IEntryStore, IGraphStore},
    types::{EdgeType, Entry, EntryState, Signal},
};

/// Coordinates entry read operations.
pub struct EntryManager {
    store: Arc<dyn IEntryStore>,
    graph: Arc<dyn IGraphStore>,
}

impl EntryManager {
    /// Create an `EntryManager` backed by `store` and `graph`.
    #[must_use]
    pub fn new(store: Arc<dyn IEntryStore>, graph: Arc<dyn IGraphStore>) -> Self {
        Self { store, graph }
    }

    /// Return all entries.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_all(&self) -> Result<Vec<Entry>> {
        self.store.list().await
    }

    /// Return entries filtered by lifecycle state.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_by_state(&self, state: EntryState) -> Result<Vec<Entry>> {
        let all = self.store.list().await?;
        Ok(all.into_iter().filter(|e| e.state == state).collect())
    }

    /// Return entries with the given signal strength.
    /// Entries with `signal == None` are excluded.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_by_signal(&self, signal: Signal) -> Result<Vec<Entry>> {
        let all = self.store.list().await?;
        Ok(all
            .into_iter()
            .filter(|e| e.signal.as_ref() == Some(&signal))
            .collect())
    }

    /// Return entries belonging to the given source.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>> {
        self.store.list_by_source(source_id).await
    }

    /// Return entries from sources monitored by the given topic.
    ///
    /// Walks `Monitors` edges from the topic node, then performs a single
    /// `store.list()` and filters by source ID membership. This is a single
    /// scan regardless of the number of monitored sources.
    ///
    /// # Errors
    /// Returns an error if the graph store or entry store fails.
    pub async fn list_by_topic(&self, topic_id: Uuid) -> Result<Vec<Entry>> {
        let edges = self.graph.get_edges_by_node(topic_id).await?;
        let source_ids: std::collections::HashSet<Uuid> = edges
            .into_iter()
            .filter(|e| e.from_id == topic_id && e.edge_type == EdgeType::Monitors)
            .map(|e| e.to_id)
            .collect();
        if source_ids.is_empty() {
            return Ok(vec![]);
        }
        let all = self.store.list().await?;
        Ok(all
            .into_iter()
            .filter(|e| source_ids.contains(&e.source_id))
            .collect())
    }

    /// Retrieve a single entry by UUID. Returns `None` if not found.
    ///
    /// # Errors
    /// Returns an error if the store fails.
    pub async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
        self.store.get(id).await
    }
}
```

- [ ] **Step 4.5: Run the tests to confirm GREEN**

```bash
cargo test -p tankyu-core entry_manager
```

Expected: 11 tests pass, 0 fail.

- [ ] **Step 4.6: Run clippy**

```bash
cargo clippy -p tankyu-core -- -D warnings
```

Expected: no warnings.

- [ ] **Step 4.7: Commit**

```bash
git add crates/tankyu-core/src/features/entry/ \
        crates/tankyu-core/src/features/mod.rs
git commit -m "feat(core): EntryManager — list_all, list_by_state, list_by_signal, list_by_source, list_by_topic, get"
```

---

### Task 5: ~~Export `EntryManager` from `features/mod.rs`~~ (subsumed into Task 4)

> **Note:** Step 4.1 already adds `pub mod entry;` to `features/mod.rs` as a prerequisite for the RED test in Step 4.3. There is no separate work here. Skip directly to Task 6.

The commit for `features/mod.rs` is included in the Task 4 commit (Step 4.7 stages `crates/tankyu-core/src/features/entry/` which includes both the new files and the modified `features/mod.rs`).

---

### Task 6: Wire `EntryManager` into `AppContext`

**Files:**
- Modify: `crates/tankyu-cli/src/context.rs`
- Modify: `crates/tankyu-cli/src/commands/status.rs`
- Modify: `crates/tankyu-cli/src/commands/doctor.rs`

**Background:** Replace the leaking `entry_store: Arc<EntryStore>` (concrete type) with `entry_mgr: EntryManager` (domain abstraction). Update the two commands that currently use `entry_store` directly.

- [ ] **Step 6.1: Write a test that will fail until wiring is complete**

In `crates/tankyu-cli/tests/cmd_tests.rs`, add after the last test:

```rust
#[test]
fn status_json_counts_entries_via_mgr() {
    // This test verifies AppContext wires entry_mgr (not raw entry_store)
    // by confirming status still works after the refactor
    let dir = create_fixture();
    let output = cmd(&dir).args(["--json", "status"]).output().unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["entries"], 0); // fixture has no entries yet
}
```

- [ ] **Step 6.2: Run to confirm it currently passes (GREEN before refactor)**

```bash
cargo test -p tankyu-cli status_json_counts_entries_via_mgr
```

Expected: passes (confirms baseline before refactor).

- [ ] **Step 6.3: Rewrite `context.rs`**

Replace the entire content of `crates/tankyu-cli/src/context.rs` with:

```rust
use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use tankyu_core::{
    domain::{
        ports::{IEntryStore, IGraphStore, ISourceStore, ITopicStore},
        types::TankyuConfig,
    },
    features::{entry::EntryManager, source::SourceManager, topic::TopicManager},
    infrastructure::{
        graph::JsonGraphStore,
        stores::{EntryStore, SourceStore, TopicStore},
    },
    shared::constants,
};

use crate::output::OutputMode;

pub struct AppContext {
    pub topic_mgr: TopicManager,
    pub source_mgr: SourceManager,
    pub entry_mgr: EntryManager,
    pub config: TankyuConfig,
    pub output: OutputMode,
    pub data_dir: PathBuf,
}

impl AppContext {
    pub async fn new(tankyu_dir_arg: Option<PathBuf>, json: bool) -> Result<Self> {
        let base = tankyu_dir_arg.unwrap_or_else(constants::tankyu_dir);

        let config_path = constants::config_path(&base);
        let config_bytes = tokio::fs::read(&config_path)
            .await
            .with_context(|| format!("Cannot read config: {}", config_path.display()))?;
        let config: TankyuConfig = serde_json::from_slice(&config_bytes)
            .with_context(|| format!("Cannot parse config: {}", config_path.display()))?;

        let topic_store: Arc<dyn ITopicStore> =
            Arc::new(TopicStore::new(constants::topics_dir(&base)));
        let source_store: Arc<dyn ISourceStore> =
            Arc::new(SourceStore::new(constants::sources_dir(&base)));
        let graph_store: Arc<dyn IGraphStore> =
            Arc::new(JsonGraphStore::new(constants::edges_path(&base)));
        let entry_store: Arc<dyn IEntryStore> =
            Arc::new(EntryStore::new(constants::entries_dir(&base)));

        Ok(Self {
            topic_mgr: TopicManager::new(topic_store),
            source_mgr: SourceManager::new(source_store, Arc::clone(&graph_store)),
            entry_mgr: EntryManager::new(entry_store, graph_store),
            config,
            output: OutputMode::detect(json),
            data_dir: base,
        })
    }
}
```

- [ ] **Step 6.4: Update `commands/status.rs`**

Replace the content of `crates/tankyu-cli/src/commands/status.rs` with:

```rust
use anyhow::Result;

use crate::context::AppContext;

pub async fn run(ctx: &AppContext) -> Result<()> {
    let topics = ctx.topic_mgr.list_all().await?;
    let sources = ctx.source_mgr.list_all().await?;
    let entries = ctx.entry_mgr.list_all().await?;

    if ctx.output.is_json() {
        println!(
            "{}",
            serde_json::json!({
                "topics": topics.len(),
                "sources": sources.len(),
                "entries": entries.len()
            })
        );
        return Ok(());
    }

    let mut table = comfy_table::Table::new();
    table.set_header(["Metric", "Count"]);
    table.add_row(["Topics", &topics.len().to_string()]);
    table.add_row(["Sources", &sources.len().to_string()]);
    table.add_row(["Entries", &entries.len().to_string()]);
    println!("{table}");
    Ok(())
}
```

- [ ] **Step 6.5: Update `commands/doctor.rs`**

Replace the `IEntryStore` import and `entry_store` usage. Open `crates/tankyu-cli/src/commands/doctor.rs`. Change the import from:

```rust
use tankyu_core::domain::ports::IEntryStore;
```

Remove that import entirely (no longer needed — `entry_mgr` is used instead).

Change:

```rust
    let entry_count = ctx.entry_store.list().await?.len();
```

to:

```rust
    let entry_count = ctx.entry_mgr.list_all().await?.len();
```

- [ ] **Step 6.6: Build and run tests**

```bash
cargo build -p tankyu-cli && cargo test --all
```

Expected: clean build, all tests pass (including `status_json_counts_entries_via_mgr`).

- [ ] **Step 6.7: Run clippy**

```bash
cargo clippy --all -- -D warnings
```

Expected: no warnings.

- [ ] **Step 6.8: Commit**

```bash
git add crates/tankyu-cli/src/context.rs \
        crates/tankyu-cli/src/commands/status.rs \
        crates/tankyu-cli/src/commands/doctor.rs \
        crates/tankyu-cli/tests/cmd_tests.rs
git commit -m "refactor(context): replace entry_store with entry_mgr, fix abstraction leak"
```

---

## Chunk 3: Test Infrastructure

### Task 7: Add entry fixture to CLI tests

**Files:**
- Modify: `crates/tankyu-cli/tests/common/mod.rs`

**Background:** The CLI integration tests use `create_fixture()` to produce a temp data directory. Currently it has no entry JSON. Add one entry belonging to the existing `SOURCE_ID` so `entry list` tests have data to work with.

- [ ] **Step 7.1: Add the `ENTRY_ID` constant and entry fixture**

Open `crates/tankyu-cli/tests/common/mod.rs`. After the `SOURCE_ID` constant, add:

```rust
pub const ENTRY_ID: &str = "33333333-3333-3333-3333-333333333333";
```

Inside `create_fixture()`, after the `write_json(b.join(format!("sources/{SOURCE_ID}.json")), ...)` block and before the `write_json(b.join("graph/edges.json"), ...)` line, add:

```rust
    write_json(
        b.join(format!("entries/{ENTRY_ID}.json")),
        &serde_json::json!({
            "id": ENTRY_ID,
            "sourceId": SOURCE_ID,
            "type": "commit",
            "title": "feat: add entry management",
            "url": "https://github.com/rust-lang/rust/commit/abc123",
            "summary": null,
            "contentHash": null,
            "state": "new",
            "signal": "high",
            "scannedAt": "2025-01-15T10:00:00Z",
            "metadata": null,
            "createdAt": "2025-01-15T10:00:00Z"
        }),
    );
```

Also add a helper function (after `cmd`) for tests that need multiple entries:

```rust
/// Write an additional entry fixture to an existing fixture dir.
pub fn write_entry(
    dir: &TempDir,
    id: &str,
    title: &str,
    state: &str,
    signal: Option<&str>,
) {
    write_json(
        dir.path().join(format!("entries/{id}.json")),
        &serde_json::json!({
            "id": id,
            "sourceId": SOURCE_ID,
            "type": "article",
            "title": title,
            "url": format!("https://example.com/{id}"),
            "summary": null,
            "contentHash": null,
            "state": state,
            "signal": signal,
            "scannedAt": "2025-01-15T10:00:00Z",
            "metadata": null,
            "createdAt": "2025-01-15T10:00:00Z"
        }),
    );
}
```

- [ ] **Step 7.2: Update `cmd_tests.rs` status count**

In `crates/tankyu-cli/tests/cmd_tests.rs`, find `status_json_has_counts` and update the entries assertion:

```rust
    assert_eq!(v["entries"], 1); // was 0, now 1 after adding entry fixture
```

Also update `status_json_counts_entries_via_mgr` (added in Step 6.1) to reflect the new entry count:

```rust
    assert_eq!(v["entries"], 1);
```

- [ ] **Step 7.3: Run tests**

```bash
cargo test --all
```

Expected: all tests pass (status count assertions reflect the new entry).

- [ ] **Step 7.4: Commit**

```bash
git add crates/tankyu-cli/tests/common/mod.rs \
        crates/tankyu-cli/tests/cmd_tests.rs
git commit -m "test(fixtures): add entry fixture to CLI test data directory"
```

---

### Task 8: Add cucumber + scaffold acceptance test binary

**Files:**
- Modify: `Cargo.toml` (workspace)
- Modify: `crates/tankyu-cli/Cargo.toml`
- Create: `crates/tankyu-cli/tests/acceptance/main.rs`
- Create: `crates/tankyu-cli/tests/acceptance/world.rs`
- Create: `crates/tankyu-cli/tests/acceptance/steps/mod.rs`

**Background:** cucumber replaces the default test harness for BDD tests. Each cucumber test binary requires `harness = false` in its `[[test]]` block.

- [ ] **Step 8.1: Add cucumber to workspace dependencies**

In `Cargo.toml` (workspace root), in `[workspace.dependencies]`, add:

```toml
cucumber = { version = "0.22", features = ["macros"] }
```

- [ ] **Step 8.2: Add cucumber to CLI dev-dependencies and register the test binary**

In `crates/tankyu-cli/Cargo.toml`:

In `[dev-dependencies]`, add:

```toml
cucumber = { workspace = true }
serde_json = { workspace = true }
```

After the `[dev-dependencies]` block, add:

```toml
[[test]]
name = "acceptance"
path = "tests/acceptance/main.rs"
harness = false
```

- [ ] **Step 8.3: Create the acceptance test entry point**

Create `crates/tankyu-cli/tests/acceptance/main.rs`:

```rust
mod steps;
mod world;

use world::TankyuWorld;

#[tokio::main]
async fn main() {
    TankyuWorld::run("tests/acceptance/features").await;
}
```

- [ ] **Step 8.4: Create the World struct**

Create `crates/tankyu-cli/tests/acceptance/world.rs`:

```rust
use assert_cmd::Command;
use cucumber::World;
use std::path::Path;
use tempfile::TempDir;

fn write_json(path: impl AsRef<Path>, value: &serde_json::Value) {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

/// Shared state carried through all steps of one cucumber scenario.
#[derive(Debug, World)]
#[world(init = Self::new)]
pub struct TankyuWorld {
    pub data_dir: TempDir,
    pub last_stdout: String,
    pub last_stderr: String,
    pub last_exit_code: Option<i32>,
}

impl TankyuWorld {
    async fn new() -> Self {
        let dir = TempDir::new().unwrap();
        let b = dir.path();
        for sub in &["topics", "sources", "entries", "graph"] {
            std::fs::create_dir_all(b.join(sub)).unwrap();
        }
        write_json(
            b.join("config.json"),
            &serde_json::json!({
                "version": 1, "defaultScanLimit": 20, "staleDays": 7,
                "dormantDays": 30, "llmClassify": false, "localRepoPaths": {}
            }),
        );
        write_json(
            b.join("graph/edges.json"),
            &serde_json::json!({ "version": 1, "edges": [] }),
        );
        Self {
            data_dir: dir,
            last_stdout: String::new(),
            last_stderr: String::new(),
            last_exit_code: None,
        }
    }

    pub fn run_cmd(&mut self, args: &[&str]) {
        let mut c = Command::cargo_bin("tankyu").unwrap();
        c.env("TANKYU_DIR", self.data_dir.path());
        for arg in args {
            c.arg(arg);
        }
        let out = c.output().unwrap();
        self.last_stdout = String::from_utf8_lossy(&out.stdout).to_string();
        self.last_stderr = String::from_utf8_lossy(&out.stderr).to_string();
        self.last_exit_code = out.status.code();
    }

    pub fn write_entry(&self, id: &str, title: &str, state: &str, signal: Option<&str>) {
        write_json(
            self.data_dir.path().join(format!("entries/{id}.json")),
            &serde_json::json!({
                "id": id,
                "sourceId": "22222222-2222-2222-2222-222222222222",
                "type": "article",
                "title": title,
                "url": format!("https://example.com/{id}"),
                "summary": null,
                "contentHash": null,
                "state": state,
                "signal": signal,
                "scannedAt": "2025-01-15T10:00:00Z",
                "metadata": null,
                "createdAt": "2025-01-15T10:00:00Z"
            }),
        );
    }
}
```

- [ ] **Step 8.5: Create the steps module stub**

Create `crates/tankyu-cli/tests/acceptance/steps/mod.rs`:

```rust
pub mod entry_steps;
```

Create `crates/tankyu-cli/tests/acceptance/steps/entry_steps.rs` as an empty stub for now:

```rust
// Step definitions added in Task 9
```

- [ ] **Step 8.6: Verify the acceptance binary compiles**

```bash
cargo test --test acceptance 2>&1 | head -30
```

Expected: compiles but fails immediately because there are no `.feature` files yet. That is correct — the binary ran with no scenarios.

- [ ] **Step 8.7: Commit**

```bash
git add Cargo.toml \
        crates/tankyu-cli/Cargo.toml \
        crates/tankyu-cli/tests/acceptance/
git commit -m "test(bdd): scaffold cucumber acceptance test binary and World struct"
```

---

### Task 9: Write the `.feature` file and step definitions

**Files:**
- Create: `crates/tankyu-cli/tests/acceptance/features/entry.feature`
- Modify: `crates/tankyu-cli/tests/acceptance/steps/entry_steps.rs`

**Background:** In BDD/ATDD, the `.feature` file is the spec. It must exist and compile before step definitions are written. Step definitions come before implementation.

- [ ] **Step 9.1: Create the feature file**

Create `crates/tankyu-cli/tests/acceptance/features/entry.feature`:

```gherkin
Feature: Entry management
  As a researcher
  I want to list and inspect my collected entries
  So that I can review what has been gathered and focus on what needs attention

  Scenario: List entries in table format when entries exist
    Given the data directory contains 3 entries with mixed state
    When I run "entry list"
    Then the command exits successfully
    And stdout contains "new"

  Scenario: List all entries as JSON
    Given the data directory contains 3 entries with mixed state
    When I run "entry list --json"
    Then the command exits successfully
    And stdout is a JSON array of length 3

  Scenario: Filter entries by state new
    Given the data directory contains 3 entries with mixed state
    When I run "entry list --state new"
    Then the command exits successfully
    And stdout contains "Alpha entry"
    And stdout does not contain "Beta entry"

  Scenario: Invalid state flag is rejected
    When I run "entry list --state garbage"
    Then the command exits with failure
    And stderr contains "Invalid state"

  Scenario: topic and source flags are mutually exclusive
    When I run "entry list --topic foo --source bar"
    Then the command exits with failure
    And stderr contains "mutually exclusive"

  Scenario: No entries shows empty table
    When I run "entry list"
    Then the command exits successfully

  Scenario: Inspect a non-existent entry fails
    When I run "entry inspect 00000000-0000-0000-0000-000000000000"
    Then the command exits with failure
    And stderr contains "not found"
```

- [ ] **Step 9.2: Run to confirm step definitions are needed (RED)**

```bash
cargo test --test acceptance 2>&1 | head -40
```

Expected: scenarios run but fail with "step is not defined" errors for each step. That is the correct RED.

- [ ] **Step 9.3: Write the step definitions**

Replace `crates/tankyu-cli/tests/acceptance/steps/entry_steps.rs` with:

```rust
use crate::world::TankyuWorld;
use cucumber::{given, then, when};

#[given("the data directory contains 3 entries with mixed state")]
async fn given_three_entries(world: &mut TankyuWorld) {
    world.write_entry(
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "Alpha entry",
        "new",
        Some("high"),
    );
    world.write_entry(
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "Beta entry",
        "read",
        Some("low"),
    );
    world.write_entry(
        "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "Gamma entry",
        "triaged",
        None,
    );
}

#[when(expr = "I run {string}")]
async fn when_run(world: &mut TankyuWorld, cmd_str: String) {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    world.run_cmd(&parts);
}

#[then("the command exits successfully")]
async fn then_exits_success(world: &mut TankyuWorld) {
    assert_eq!(
        world.last_exit_code,
        Some(0),
        "Expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
        world.last_exit_code,
        world.last_stdout,
        world.last_stderr
    );
}

#[then("the command exits with failure")]
async fn then_exits_failure(world: &mut TankyuWorld) {
    assert_ne!(
        world.last_exit_code,
        Some(0),
        "Expected non-zero exit\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stdout contains {string}")]
async fn then_stdout_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        world.last_stdout.contains(&needle),
        "stdout did not contain {needle:?}\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stdout does not contain {string}")]
async fn then_stdout_not_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        !world.last_stdout.contains(&needle),
        "stdout should NOT contain {needle:?}\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stderr contains {string}")]
async fn then_stderr_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        world.last_stderr.to_lowercase().contains(&needle.to_lowercase()),
        "stderr did not contain {needle:?}\nstderr: {}",
        world.last_stderr
    );
}

#[then(expr = "stdout is a JSON array of length {int}")]
async fn then_json_array_length(world: &mut TankyuWorld, len: i64) {
    let v: serde_json::Value = serde_json::from_str(&world.last_stdout)
        .expect("stdout is not valid JSON");
    let arr = v.as_array().expect("stdout is not a JSON array");
    assert_eq!(arr.len(), len as usize, "expected {len} items, got {}", arr.len());
}
```

- [ ] **Step 9.4: Run the acceptance tests — expect RED (entry commands not yet implemented)**

```bash
cargo test --test acceptance 2>&1 | tail -20
```

Expected: step definitions compile and match, but scenarios fail because `tankyu entry list` is not a valid subcommand yet. This is the correct RED before CLI implementation.

- [ ] **Step 9.5: Commit the feature file and step definitions**

```bash
git add crates/tankyu-cli/tests/acceptance/
git commit -m "test(bdd): entry.feature scenarios and step definitions — RED before CLI implementation"
```

---

## Chunk 4: CLI Commands

### Task 10: Add `EntryCommands` to the CLI definition

**Files:**
- Modify: `crates/tankyu-cli/src/cli.rs`
- Modify: `crates/tankyu-cli/src/commands/mod.rs`
- Modify: `crates/tankyu-cli/src/main.rs`

- [ ] **Step 10.1: Write a failing CLI test first**

In `crates/tankyu-cli/tests/cmd_tests.rs`, add:

```rust
#[test]
fn entry_list_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["entry", "list"]).assert().success();
}

#[test]
fn entry_list_json_is_array() {
    let dir = create_fixture();
    let output = cmd(&dir)
        .args(["--json", "entry", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v.is_array());
}

#[test]
fn entry_inspect_missing_fails() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "inspect", "00000000-0000-0000-0000-000000000000"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not found"),
        "expected 'not found' in stderr, got: {stderr}"
    );
}

#[test]
fn entry_list_invalid_state_fails() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "list", "--state", "garbage"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("Invalid state"),
        "expected 'Invalid state' in stderr, got: {stderr}"
    );
}

#[test]
fn entry_list_source_and_topic_mutual_exclusion() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "list", "--source", "foo", "--topic", "bar"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("mutually exclusive"),
        "expected 'mutually exclusive' in stderr, got: {stderr}"
    );
}
```

- [ ] **Step 10.2: Run the tests to confirm RED**

```bash
cargo test -p tankyu-cli entry_list
```

Expected: compile error — `error[E0308]: mismatched types` or similar because `entry` is not a valid subcommand. That is RED.

- [ ] **Step 10.3: Add `EntryCommands` to `cli.rs`**

Open `crates/tankyu-cli/src/cli.rs`. After the `SourceCommands` enum, add:

```rust
#[derive(Subcommand)]
pub enum EntryCommands {
    /// List entries, optionally filtered by state, signal, source, or topic
    List {
        /// Filter by state: new, scanned, triaged, read, archived
        #[arg(long)]
        state: Option<String>,
        /// Filter by signal: high, medium, low, noise
        #[arg(long)]
        signal: Option<String>,
        /// Filter by source name
        #[arg(long)]
        source: Option<String>,
        /// Filter by topic name (resolves to sources monitored by that topic)
        #[arg(long)]
        topic: Option<String>,
        /// Limit number of results (applied after all filters)
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Inspect a single entry by UUID
    Inspect {
        /// Entry UUID
        id: String,
    },
}
```

Add `Entry` to the `Commands` enum (after `Source`):

```rust
    /// Entry management
    Entry {
        #[command(subcommand)]
        command: EntryCommands,
    },
```

- [ ] **Step 10.4: Add `pub mod entry` to `commands/mod.rs`**

Open `crates/tankyu-cli/src/commands/mod.rs`. Add:

```rust
pub mod entry;
```

- [ ] **Step 10.5: Create a stub `commands/entry.rs` that compiles but panics**

Create `crates/tankyu-cli/src/commands/entry.rs`:

```rust
use anyhow::Result;

use crate::context::AppContext;

pub async fn list(
    _ctx: &AppContext,
    _state: Option<&str>,
    _signal: Option<&str>,
    _source: Option<&str>,
    _topic: Option<&str>,
    _limit: Option<usize>,
) -> Result<()> {
    todo!("entry list not yet implemented")
}

pub async fn inspect(_ctx: &AppContext, _id: &str) -> Result<()> {
    todo!("entry inspect not yet implemented")
}
```

- [ ] **Step 10.6: Wire the match arm in `main.rs`**

Open `crates/tankyu-cli/src/main.rs`. Change the import on line 12 from:

```rust
use cli::{Cli, Commands, ConfigCommands, SourceCommands, TopicCommands};
```

to:

```rust
use cli::{Cli, Commands, ConfigCommands, EntryCommands, SourceCommands, TopicCommands};
```

Add the match arm after the `Source` block (before `Config`):

```rust
        Commands::Entry { command } => match command {
            EntryCommands::List { state, signal, source, topic, limit } => {
                commands::entry::list(
                    &ctx,
                    state.as_deref(),
                    signal.as_deref(),
                    source.as_deref(),
                    topic.as_deref(),
                    limit,
                )
                .await
            }
            EntryCommands::Inspect { id } => commands::entry::inspect(&ctx, &id).await,
        },
```

- [ ] **Step 10.7: Run tests to confirm they compile but fail (still RED)**

```bash
cargo test -p tankyu-cli entry_list_exits_success 2>&1
```

Expected: test compiles but fails — `entry list` runs and panics with `todo!`. That's expected RED.

- [ ] **Step 10.8: Commit the stub**

```bash
git add crates/tankyu-cli/src/cli.rs \
        crates/tankyu-cli/src/commands/mod.rs \
        crates/tankyu-cli/src/commands/entry.rs \
        crates/tankyu-cli/src/main.rs \
        crates/tankyu-cli/tests/cmd_tests.rs
git commit -m "feat(cli): add EntryCommands stub — list and inspect wired, todo!() bodies"
```

---

### Task 11: Implement `commands/entry.rs`

**Files:**
- Modify: `crates/tankyu-cli/src/commands/entry.rs`

- [ ] **Step 11.1: Replace the stub with the full implementation**

Replace the content of `crates/tankyu-cli/src/commands/entry.rs` with:

```rust
use anyhow::Result;
use uuid::Uuid;

use tankyu_core::domain::types::{EntryState, EntryType, Signal};

use crate::context::AppContext;

fn parse_state(s: &str) -> Result<EntryState> {
    match s {
        "new" => Ok(EntryState::New),
        "scanned" => Ok(EntryState::Scanned),
        "triaged" => Ok(EntryState::Triaged),
        "read" => Ok(EntryState::Read),
        "archived" => Ok(EntryState::Archived),
        _ => Err(anyhow::anyhow!(
            "Invalid state '{s}'. Valid: new, scanned, triaged, read, archived"
        )),
    }
}

fn parse_signal(s: &str) -> Result<Signal> {
    match s {
        "high" => Ok(Signal::High),
        "medium" => Ok(Signal::Medium),
        "low" => Ok(Signal::Low),
        "noise" => Ok(Signal::Noise),
        _ => Err(anyhow::anyhow!(
            "Invalid signal '{s}'. Valid: high, medium, low, noise"
        )),
    }
}

const fn state_str(s: &EntryState) -> &'static str {
    match s {
        EntryState::New => "new",
        EntryState::Scanned => "scanned",
        EntryState::Triaged => "triaged",
        EntryState::Read => "read",
        EntryState::Archived => "archived",
    }
}

const fn signal_str(s: Option<&Signal>) -> &'static str {
    match s {
        None => "—",
        Some(Signal::High) => "high",
        Some(Signal::Medium) => "medium",
        Some(Signal::Low) => "low",
        Some(Signal::Noise) => "noise",
    }
}

const fn type_str(t: &EntryType) -> &'static str {
    match t {
        EntryType::Tweet => "tweet",
        EntryType::Commit => "commit",
        EntryType::Pr => "pr",
        EntryType::Release => "release",
        EntryType::Article => "article",
        EntryType::Page => "page",
        EntryType::Repo => "repo",
        EntryType::GithubIssue => "github-issue",
        EntryType::SpikeReport => "spike-report",
    }
}

pub async fn list(
    ctx: &AppContext,
    state: Option<&str>,
    signal: Option<&str>,
    source: Option<&str>,
    topic: Option<&str>,
    limit: Option<usize>,
) -> Result<()> {
    if source.is_some() && topic.is_some() {
        anyhow::bail!("--topic and --source are mutually exclusive");
    }

    let mut entries = match (source, topic) {
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
        _ => ctx.entry_mgr.list_all().await?,
    };

    if let Some(s) = state {
        let filter = parse_state(s)?;
        entries.retain(|e| e.state == filter);
    }

    if let Some(s) = signal {
        let filter = parse_signal(s)?;
        entries.retain(|e| e.signal.as_ref() == Some(&filter));
    }

    entries.sort_by(|a, b| b.scanned_at.cmp(&a.scanned_at));

    if let Some(n) = limit {
        entries.truncate(n);
    }

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&entries)?);
        return Ok(());
    }

    let mut table = comfy_table::Table::new();
    table.set_header(["ID", "Type", "State", "Signal", "Title", "Scanned"]);
    for e in &entries {
        let id_short = &e.id.to_string()[..8];
        let title: String = if e.title.chars().count() > 60 {
            format!("{}…", e.title.chars().take(59).collect::<String>())
        } else {
            e.title.clone()
        };
        table.add_row([
            id_short,
            type_str(&e.r#type),
            state_str(&e.state),
            signal_str(e.signal.as_ref()),
            &title,
            &e.scanned_at.format("%Y-%m-%d").to_string(),
        ]);
    }
    println!("{table}");
    Ok(())
}

pub async fn inspect(ctx: &AppContext, id: &str) -> Result<()> {
    let uuid = Uuid::parse_str(id).map_err(|_| anyhow::anyhow!("Invalid UUID: {id}"))?;
    let e = ctx
        .entry_mgr
        .get(uuid)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Entry '{id}' not found"))?;

    if ctx.output.is_json() {
        println!("{}", serde_json::to_string(&e)?);
        return Ok(());
    }

    println!("ID:        {}", e.id);
    println!("Type:      {}", type_str(&e.r#type));
    println!("State:     {}", state_str(&e.state));
    println!("Signal:    {}", signal_str(e.signal.as_ref()));
    println!("Title:     {}", e.title);
    println!("URL:       {}", e.url);
    println!("Source:    {}", e.source_id);
    println!("Summary:   {}", e.summary.as_deref().unwrap_or("—"));
    println!("Scanned:   {}", e.scanned_at);
    println!("Created:   {}", e.created_at.format("%Y-%m-%d"));
    Ok(())
}
```

- [ ] **Step 11.2: Run the assert_cmd tests**

```bash
cargo test -p tankyu-cli entry_list entry_inspect
```

Expected: `entry_list_exits_success`, `entry_list_json_is_array`, `entry_inspect_missing_fails`, `entry_list_invalid_state_fails`, `entry_list_source_and_topic_mutual_exclusion` all pass.

- [ ] **Step 11.3: Run the acceptance tests**

```bash
cargo test --test acceptance
```

Expected: all BDD scenarios pass (GREEN).

- [ ] **Step 11.4: Run clippy**

```bash
cargo clippy --all -- -D warnings
```

Expected: no warnings.

- [ ] **Step 11.5: Run full suite**

```bash
cargo test --all
```

Expected: all tests pass.

- [ ] **Step 11.6: Commit**

```bash
git add crates/tankyu-cli/src/commands/entry.rs
git commit -m "feat(cli): implement entry list and entry inspect commands"
```

---

### Task 12: Add insta snapshot tests

**Files:**
- Create: `crates/tankyu-cli/tests/cli_entry.rs`

**Background:** Snapshot tests lock the rendered output format. They use `insta::assert_snapshot!` — on first run, the snapshot is written (call `cargo insta review` to accept). Subsequent runs fail if the output changes unexpectedly.

Use `NO_COLOR=1` for plain table output (disables terminal color codes that would make snapshots non-deterministic).

- [ ] **Step 12.1: Create `cli_entry.rs`**

Create `crates/tankyu-cli/tests/cli_entry.rs`:

```rust
mod common;
use common::{cmd, create_fixture, write_entry, ENTRY_ID};

#[test]
fn entry_list_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list"])
        .output()
        .unwrap();
    assert!(out.status.success(), "entry list failed: {}", String::from_utf8_lossy(&out.stderr));
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "entry", "list"])
        .output()
        .unwrap();
    // The fixture uses fixed ISO timestamps so the snapshot is deterministic.
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], ENTRY_ID);
    assert_eq!(arr[0]["state"], "new");
    assert_eq!(arr[0]["signal"], "high");
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_filtered_by_state() {
    let dir = create_fixture();
    // Add a second entry with different state
    write_entry(&dir, "44444444-4444-4444-4444-444444444444", "A read entry", "read", None);
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--state", "new"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("feat: add entry management"));
    assert!(!stdout.contains("A read entry"));
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_list_filtered_by_signal() {
    let dir = create_fixture();
    write_entry(&dir, "55555555-5555-5555-5555-555555555555", "Low signal entry", "new", Some("low"));
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--signal", "high"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("feat: add entry management"));
    assert!(!stdout.contains("Low signal entry"));
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_list_filtered_by_source() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--source", "rust-lang/rust"])
        .output()
        .unwrap();
    assert!(out.status.success(), "entry list --source failed: {}", String::from_utf8_lossy(&out.stderr));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("feat: add entry management"), "expected fixture entry in output: {stdout}");
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_inspect_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "inspect", ENTRY_ID])
        .output()
        .unwrap();
    assert!(out.status.success(), "entry inspect failed: {}", String::from_utf8_lossy(&out.stderr));
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_inspect_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "entry", "inspect", ENTRY_ID])
        .output()
        .unwrap();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], ENTRY_ID);
    assert_eq!(v["state"], "new");
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_inspect_invalid_uuid_fails() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "inspect", "not-a-uuid"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Invalid UUID"));
}
```

- [ ] **Step 12.2: Run the snapshot tests (first run writes snapshots)**

```bash
cargo test -p tankyu-cli cli_entry
```

Expected: tests run and new snapshots are written to `crates/tankyu-cli/tests/snapshots/`. Tests show as "PENDING" (not failed — insta creates the snapshot file but marks it for review).

- [ ] **Step 12.3: Review and accept the snapshots**

```bash
cargo insta review
```

Press `a` (accept) for each snapshot. Verify the output looks correct before accepting:
- `entry_list_plain`: should show a table with 1 row (the fixture entry)
- `entry_list_json`: should show a JSON array with 1 object
- `entry_inspect_plain`: should show key/value pairs with the fixture entry's data
- `entry_inspect_json`: should show the full Entry JSON object

- [ ] **Step 12.4: Run tests again to confirm GREEN**

```bash
cargo test -p tankyu-cli cli_entry
```

Expected: all 8 tests pass.

- [ ] **Step 12.5: Run full suite**

```bash
cargo test --all
```

Expected: all tests pass.

- [ ] **Step 12.6: Commit**

```bash
git add crates/tankyu-cli/tests/cli_entry.rs \
        crates/tankyu-cli/tests/snapshots/
git commit -m "test(snapshots): entry list and inspect — plain + JSON + filtered variants"
```

---

## Chunk 5: Quality Gates

### Task 13: Mutation testing gate

**Background:** Mutation testing verifies that tests are actually asserting meaningful behavior. Install `cargo-mutants` once (if not already installed), then run it on `entry_manager.rs`. Target: 80% kill rate.

- [ ] **Step 13.1: Install cargo-mutants (if needed)**

```bash
which cargo-mutants || cargo install cargo-mutants
```

- [ ] **Step 13.2: Run mutation testing on `entry_manager.rs`**

```bash
cargo mutants -p tankyu-core \
  --file crates/tankyu-core/src/features/entry/entry_manager.rs \
  2>&1 | tee /tmp/mutants-entry.log
```

Expected runtime: 1–3 minutes (each mutant recompiles and runs the test suite).

- [ ] **Step 13.3: Check the result**

```bash
tail -20 /tmp/mutants-entry.log
```

Look for the summary line like:
```
8 mutants tested: 7 caught, 1 missed (87% caught)
```

- [ ] **Step 13.4: If score < 80% — find and kill surviving mutants**

Run to see which mutants survived:

```bash
cat /tmp/mutants-entry.log | grep "^MISSED"
```

A typical survivor looks like:
```
MISSED: ...entry_manager.rs:45:13: replace `==` with `!=` in filter
```

This means the `list_by_state` filter equality check is not being tested with a case where the wrong state would pass. Add a targeted test in `entry_manager.rs #[cfg(test)]`:

```rust
#[tokio::test]
async fn test_list_by_state_wrong_state_returns_empty() {
    let store = Arc::new(StubEntryStore {
        entries: vec![make_entry("Read", EntryState::Read)],
    });
    let graph = Arc::new(StubGraphStore { edges: vec![] });
    let mgr = EntryManager::new(store, graph);
    // Asking for New when only Read exists must return empty — kills the == → != mutant
    assert!(mgr.list_by_state(EntryState::New).await.unwrap().is_empty());
}
```

Re-run mutants until score ≥ 80%.

- [ ] **Step 13.5: Also run on `source_manager.rs` for the new `get_by_name` method**

```bash
cargo mutants -p tankyu-core \
  --file crates/tankyu-core/src/features/source/source_manager.rs \
  2>&1 | grep -E "(MISSED|caught|tested)"
```

Expected: `get_by_name` mutations are caught (the existing `test_get_by_name_found` and `test_get_by_name_not_found` tests should kill them).

- [ ] **Step 13.6: Commit any new tests added to kill survivors**

Stage only the files that actually changed (add each file only if you added tests to it):

```bash
# Stage only the files you modified:
git add crates/tankyu-core/src/features/entry/entry_manager.rs   # if new tests were added here
git add crates/tankyu-core/src/features/source/source_manager.rs  # only if new tests were added here too
git commit -m "test(mutation): add targeted tests to reach 80% mutation kill rate"
```

*(Skip this entire commit if neither file needed new tests.)*

---

### Task 14: Final gate — full validation

- [ ] **Step 14.1: Run the complete test suite**

```bash
cargo test --all
```

Expected: all tests pass, zero failures.

- [ ] **Step 14.2: Run the acceptance tests explicitly**

```bash
cargo test --test acceptance
```

Expected: all BDD scenarios pass.

- [ ] **Step 14.3: Run clippy (zero warnings)**

```bash
cargo clippy --all -- -D warnings
```

Expected: no warnings.

- [ ] **Step 14.4: Run formatter check**

```bash
cargo fmt --check
```

Expected: no formatting violations. If there are: run `cargo fmt` and check the diff before committing.

- [ ] **Step 14.5: Confirm no unreviewed snapshots**

```bash
cargo insta review
```

Expected: "No snapshots are pending review." If any are pending, review and accept them now.

- [ ] **Step 14.6: Final commit if `cargo fmt` made changes**

```bash
cargo fmt
git add -p  # stage only formatting changes
git commit -m "style: cargo fmt"
```

- [ ] **Step 14.7: Verify the full commit history for this feature**

```bash
git log --oneline main..HEAD
```

Expected output should show roughly these commits (in order from oldest):
```
fix(context): Arc::clone graph_store instead of moving into SourceManager
refactor(topic): rename TopicManager::list() to list_all() for consistency
feat(source): add SourceManager::get_by_name() for --source filter resolution
feat(core): EntryManager — list_all, list_by_state, list_by_signal, list_by_source, list_by_topic, get
feat(core): export EntryManager from features module
refactor(context): replace entry_store with entry_mgr, fix abstraction leak
test(fixtures): add entry fixture to CLI test data directory
test(bdd): scaffold cucumber acceptance test binary and World struct
test(bdd): entry.feature scenarios and step definitions — RED before CLI implementation
feat(cli): add EntryCommands stub — list and inspect wired, todo!() bodies
feat(cli): implement entry list and entry inspect commands
test(snapshots): entry list and inspect — plain + JSON + filtered variants
test(mutation): add targeted tests to reach 80% mutation kill rate (if needed)
```

---

## Quick Reference

```bash
# Run everything
cargo test --all

# Run only entry manager unit tests
cargo test -p tankyu-core entry_manager

# Run only CLI integration tests
cargo test -p tankyu-cli

# Run BDD acceptance tests
cargo test --test acceptance

# Review snapshot changes
cargo insta review

# Mutation testing — entry manager
cargo mutants -p tankyu-core --file crates/tankyu-core/src/features/entry/entry_manager.rs

# Lint
cargo clippy --all -- -D warnings

# Format
cargo fmt
```
