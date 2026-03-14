# Entry Management — Design Spec

**Date:** 2026-03-14
**Status:** Approved
**Cycle:** Small batch — one session

---

## 1. Problem Statement

A researcher using tankyu accumulates entries — scanned artifacts from monitored sources (commits, articles, PRs, releases, tweets, etc.). The CLI currently exposes `status` (shows total entry counts) and `source list` (shows sources), but there is no way to see, filter, or browse the actual entries. The only option is opening raw JSON files.

**The gap:** "I know my scanner has collected 400 entries. Which ones are high-signal and unread? Which ones came from this source? I can't tell without leaving the terminal."

---

## 2. Appetite

**Small batch — one cycle.** The infrastructure is already fully built: `JsonEntryStore` exists, `IEntryStore` is defined, and both `Arc<dyn IEntryStore>` and `Arc<dyn IGraphStore>` flow through `AppContext`. The missing pieces are:
- `EntryManager` (thin feature class, ~60 lines + tests)
- CLI entry commands (2 subcommands)
- Three bug fixes bundled into the same PR (see §6)

No schema changes. No new stores. No new port methods.

---

## 3. In Scope

1. `entry list` with filters: `--state`, `--signal`, `--source <name>`, `--topic <name>`, `--limit <n>`
2. `entry inspect <id>` — full field display for one entry
3. JSON output mode for both (`--json` global flag, already exists)
4. `EntryManager` feature class with methods: `list_all`, `list_by_state`, `list_by_signal`, `list_by_source`, `list_by_topic`, `get`
5. `SourceManager::get_by_name()` — thin addition needed for `--source <name>` resolution
6. `AppContext` wired to `entry_mgr: EntryManager` (replaces the leaking `entry_store: Arc<EntryStore>`)
7. Full test coverage per the TDD contract in `TDD_WORKFLOW.md`

## 4. Out of Scope

| Feature | Reason |
|---|---|
| Write operations (entry state/signal mutation) | Preserves read-only CLI contract; `EntryUpdate` is `#[allow(dead_code)]` by design |
| `--topic`/`--source` combination | Unclear semantics; these are mutually exclusive source filters |
| Full-text search of title/summary | No store-level support; would require loading everything + string scan |
| Pagination / `--offset` | Consistent with current `topic list` / `source list` (no pagination) |
| Sorting flags | Fixed default (`scanned_at` descending); sort key combinations are a rabbit hole |
| Insight / entity commands | Separate cycle |

---

## 5. Architecture Decisions

### ADR-001: Filtering — in-memory at manager layer

**Decision:** All filters except `list_by_source` are applied after a single `store.list()` call. No new `IEntryStore` methods.

**Why:** The storage model is one JSON file per record. Any scan requires `read_all()` as the baseline. Adding `list_by_state(state)` or `list_by_signal(signal)` to the store port would issue the same syscall count with zero I/O savings. The only exception, `list_by_source`, already exists on the port because it reflects a structural property of the data (`source_id` field) and was added in the TypeScript original.

**`list_by_topic` uses single scan, not N calls:** Collects source IDs into a `HashSet<Uuid>` via graph edge traversal, then calls `store.list()` once and filters by set membership. This is strictly better than the N-call pattern in `SourceManager::list_by_topic` (which is a known issue, not replicated here).

**Rejected:** Store-level `list_by_state`, `list_by_signal` — duplicated `read_all()` + iterator logic in every store adapter and stub, for no gain.

---

### ADR-002: `EntryManager` dependencies

**Decision:** `EntryManager::new(store: Arc<dyn IEntryStore>, graph: Arc<dyn IGraphStore>)`.

**Why:** `list_by_topic` requires graph traversal (topic → `Monitors` edges → source IDs → entries). Taking `IGraphStore` directly mirrors the existing `SourceManager` pattern. No manager-to-manager dependencies — this keeps test stubs simple and avoids coupling.

**`AppContext` fix required (two parts):**
1. `graph_store` is currently *moved* into `SourceManager::new()`. Change to `Arc::clone(&graph_store)` so both `SourceManager` and `EntryManager` can receive it.
2. `entry_store` is currently typed as `Arc<EntryStore>` (concrete). Re-type it as `Arc<dyn IEntryStore>` at construction: `let entry_store: Arc<dyn IEntryStore> = Arc::new(EntryStore::new(...));`. This typed binding then flows as `Arc::clone(&entry_store)` into `EntryManager::new(...)`. `status.rs` is updated to call `ctx.entry_mgr.list_all()` instead of reaching through the raw store.

---

### ADR-003: `--source` resolves by name, not UUID

**Decision:** `--source <name>` accepts a human-readable source name. Resolution is a linear scan via `SourceManager::get_by_name()` (which calls `store.list()` + filter — no new port method needed on `ISourceStore`).

**Why:** Consistent UX with `--topic <name>`. UUIDs are not human-friendly. Sources are not pre-registered in a config file — the store is the single source of truth, and the `name` field is the natural handle.

**Note:** Name uniqueness is not enforced by the store. Duplicate names return the first match (linear scan). Document as a known behavior.

---

### ADR-004: No new `ISourceStore` or `IEntryStore` port methods

**Decision:** The existing port surfaces are sufficient. `SourceManager::get_by_name()` is added as a thin manager-level method (delegates to `store.list()` + find), not as a new port method.

**Why:** Adding to a port forces every implementor (concrete stores + all test stubs) to implement the method. A manager-level linear scan achieves the same result without widening the interface.

**Asymmetry note:** `ITopicStore` already has `get_by_name()` as a port method (mirroring the TypeScript original). `ISourceStore` does not — it has `get_by_url()` as its identity lookup. Rather than widen `ISourceStore` just for the CLI's `--source <name>` flag, the lookup is kept at the manager layer. This is a deliberate divergence from the `TopicManager` pattern; the gap exists in the TypeScript source and is not worth closing in the port.

---

### ADR-005: Bundled bug fixes

These three bugs are fixed in the same PR as entry management (they would be blocking anyway):

1. **`AppContext` abstraction leak:** `entry_store: Arc<EntryStore>` (concrete type) → replaced by `entry_mgr: EntryManager`. `status.rs` updated to call `ctx.entry_mgr.list_all()`.
2. **`graph_store` move:** `Arc::clone(&graph_store)` passed to both `SourceManager` and `EntryManager`.
3. **Naming inconsistency:** `TopicManager::list()` renamed to `list_all()`. Three call sites: `commands/topic.rs`, `commands/status.rs`, and the unit tests inside `topic_manager.rs` itself.

---

## 6. `EntryManager` Public Interface

```rust
// crates/tankyu-core/src/features/entry/entry_manager.rs

pub struct EntryManager {
    store: Arc<dyn IEntryStore>,
    graph: Arc<dyn IGraphStore>,
}

impl EntryManager {
    pub fn new(store: Arc<dyn IEntryStore>, graph: Arc<dyn IGraphStore>) -> Self;

    /// Return all entries across all sources. Sorted by callers.
    pub async fn list_all(&self) -> Result<Vec<Entry>>;

    /// Return entries matching the given lifecycle state.
    pub async fn list_by_state(&self, state: EntryState) -> Result<Vec<Entry>>;

    /// Return entries with the given signal strength. Entries with signal == None excluded.
    pub async fn list_by_signal(&self, signal: Signal) -> Result<Vec<Entry>>;

    /// Return entries belonging to a specific source (delegates to store).
    pub async fn list_by_source(&self, source_id: Uuid) -> Result<Vec<Entry>>;

    /// Return entries from sources monitored by the given topic.
    /// Walks Monitors edges from topic node, then single store.list() + set filter.
    pub async fn list_by_topic(&self, topic_id: Uuid) -> Result<Vec<Entry>>;

    /// Retrieve a single entry by UUID. Returns None if not found.
    pub async fn get(&self, id: Uuid) -> Result<Option<Entry>>;
}
```

---

## 7. CLI Commands

### `entry list`

```
tankyu entry list [--state <state>] [--signal <signal>]
                  [--source <name>] [--topic <name>]
                  [--limit <n>] [--json]
```

**Filters:**
- `--state`: `new` | `scanned` | `triaged` | `read` | `archived`
- `--signal`: `high` | `medium` | `low` | `noise`
- `--source <name>` and `--topic <name>` are **mutually exclusive** (clear error if both given)
- `--state` and `--signal` compose additively with either source filter
- `--limit <n>`: applied last, after all other filters

**Table columns (sorted by `scanned_at` descending):**

| Column | Field | Notes |
|---|---|---|
| ID | `entry.id` | First 8 hex chars |
| Type | `entry.type` | kebab-case |
| State | `entry.state` | kebab-case |
| Signal | `entry.signal` | kebab-case or `—` |
| Title | `entry.title` | Truncated (table library handles width) |
| Scanned | `entry.scanned_at` | `%Y-%m-%d` |

**JSON output:** `serde_json::to_string(&entries)` — `Vec<Entry>` directly, no envelope wrapper.

### `entry inspect <id>`

```
tankyu entry inspect <id> [--json]
```

**Plain output:**
```
ID:        <full uuid>
Type:      <type>
State:     <state>
Signal:    <signal or —>
Title:     <full title>
URL:       <url>
Source:    <source_id>
Summary:   <summary or —>
Scanned:   <scanned_at>
Created:   <created_at>
```

**JSON output:** `serde_json::to_string(&entry)` — full `Entry` directly.

### Error messages

| Condition | Error |
|---|---|
| `--topic` and `--source` both given | `--topic and --source are mutually exclusive` |
| `--state <s>` invalid | `Invalid state '{s}'. Valid: new, scanned, triaged, read, archived` |
| `--signal <s>` invalid | `Invalid signal '{s}'. Valid: high, medium, low, noise` |
| `--topic <name>` not found | `Topic '{name}' not found` |
| `--source <name>` not found | `Source '{name}' not found` |
| `entry inspect <id>` UUID parse fail | `Invalid UUID: {id}` |
| `entry inspect <id>` not found | `Entry '{id}' not found` |

---

## 8. Testing Strategy

### Pyramid

```
E2E (acceptance)     cucumber 0.22 .feature scenarios (1 file: entry.feature)
CLI integration      assert_cmd + insta snapshots in tests/cli_entry.rs
Contract suite       shared contract fn run against JsonEntryStore (new)
Store integration    tempdir round-trips in #[cfg(test)] of entry_store.rs
Property-based       proptest (existing suite extended with entry roundtrip)
Unit                 #[cfg(test)] in entry_manager.rs with stub stores
```

### Required test coverage by layer

| Change | Unit | Store-compat | CLI assert_cmd | CLI snapshot | BDD |
|---|---|---|---|---|---|
| `EntryManager` (full) | Required | — | — | — | — |
| `SourceManager::get_by_name` | Required | — | — | — | — |
| `entry list` command | — | — | Required | Required | Required |
| `entry inspect` command | — | — | Required | Required | — (intentional: inspect is a single-entity lookup, not a user-facing workflow worth a Gherkin scenario) |
| Entry fixture in `common/mod.rs` | — | Required | — | — | — |
| `AppContext` wiring fix | — | — | Required (status) | — | — |

### New tooling required

```toml
# workspace [workspace.dependencies] — add:
cucumber = "0.22"

# crates/tankyu-cli/Cargo.toml [dev-dependencies] — add:
cucumber = { workspace = true }

# crates/tankyu-cli/Cargo.toml — add test binary:
[[test]]
name = "acceptance"
path = "tests/acceptance/main.rs"
harness = false   # required — cucumber replaces the test harness
```

```bash
# Developer install (once):
cargo install cargo-mutants
```

### Mutation gate

80% kill rate required on `entry_manager.rs` before committing. Run:
```bash
cargo mutants -p tankyu-core --file src/features/entry/entry_manager.rs
```

### BDD acceptance scenario (entry.feature)

```gherkin
Feature: Entry management
  As a researcher
  I want to list and inspect my collected entries
  So that I can review what has been gathered and focus on what needs attention

  Background:
    Given a Tankyu data directory with 3 entries of mixed state and signal

  Scenario: List all entries in table format
    When I run "entry list"
    Then the command exits successfully
    And the output contains column headers

  Scenario: Filter entries by state
    When I run "entry list --state new"
    Then only entries with state "new" appear in the output

  Scenario: Invalid state is rejected
    When I run "entry list --state garbage"
    Then the command exits with failure
    And stderr contains "Invalid state"

  Scenario: topic and source flags are mutually exclusive
    When I run "entry list --topic foo --source bar"
    Then the command exits with failure
    And stderr contains "mutually exclusive"

  Scenario: List entries as JSON
    When I run "entry list --json"
    Then the output is valid JSON
    And the output is a JSON array
```

### CI integration

| Check | On push | On PR | Nightly |
|---|---|---|---|
| `cargo test --all` | ✓ | ✓ | — |
| BDD acceptance (`--test acceptance`) | ✓ | ✓ | — |
| `cargo clippy -- -D warnings` | ✓ | ✓ | — |
| Coverage (tarpaulin) | — | ✓ (info) | — |
| Mutation diff-mode (changed lines) | — | ✓ (info, `continue-on-error`) | — |
| Mutation full (enforced 75% threshold) | — | — | ✓ |

---

## 9. Implementation Sequence

Ordered by dependency — each step must be complete (tests passing) before the next:

1. **Fix `graph_store` Arc clone** in `AppContext::new()` — `Arc::clone` instead of move
2. **Rename `TopicManager::list()` → `list_all()`** — 3 call sites: `commands/topic.rs`, `commands/status.rs`, and the unit tests inside `topic_manager.rs`
3. **Add `SourceManager::get_by_name()`** — linear scan at manager level; unit test
4. **Create `features/entry/entry_manager.rs`** — write failing unit tests first (stub stores), then implement each method (red → green per method)
5. **Export `EntryManager` from `features/mod.rs`** — add `pub mod entry;`
6. **Replace `AppContext.entry_store` with `entry_mgr: EntryManager`** — *depends on steps 1, 4, 5*; re-type `entry_store` as `Arc<dyn IEntryStore>` at construction, pass `Arc::clone`s to `EntryManager::new`; update `status.rs` to call `ctx.entry_mgr.list_all()`
7. **Add entry fixture to `common/mod.rs`** and parse test to `store_compat.rs`
8. **Add cucumber to `Cargo.toml`** and scaffold `tests/acceptance/`
9. **Write `tests/acceptance/features/entry.feature`** — scenarios before step defs
10. **Write step definitions** in `tests/acceptance/steps/`
11. **Add `EntryCommands` to `cli.rs`** and arm to `main.rs`
12. **Create `commands/entry.rs`** — write CLI assert_cmd tests first (entry_list_exits_success, entry_list_state_filter, entry_list_mutual_exclusion, etc.)
13. **Add `tests/cli_entry.rs`** — snapshot tests (plain + json, filtered variants)
14. **Run `cargo mutants`** on `entry_manager.rs` — enforce 80% threshold
15. **Run full gate:** `cargo test --all`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo insta review`

---

## 10. Source Configuration Model Decision

**Question:** Should sources be registered in a config file with new sources requiring explicit config entries?

**Decision: No.** Bad design for this system.

Sources are created dynamically by the TypeScript scanner — they are not pre-declared. A config registry would:
- Create a dual source-of-truth (`~/.tankyu/sources/*.json` + config file)
- Conflict with the dynamic discovery model
- Add ceremony (register before use) that conflicts with how the tool works

**The store IS the registry.** `SourceManager.get_by_name()` does a linear scan of the store. Shell completions generated from the store at runtime are the right future answer if discoverability becomes a concern.

---

## 11. TDD Process Reference

See `TDD_WORKFLOW.md` in the repository root. That document is the mandatory process reference for all development on this project. The implementation sequence in §9 must follow its prescribed layer order (unit → store-compat → CLI e2e) with no skipping.

The Iron Law: **no production code without a failing test first.**
