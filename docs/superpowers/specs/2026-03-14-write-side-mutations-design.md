# Design Spec: Write-Side Mutations + Source Inspect + Health Command

**Date:** 2026-03-14
**Issue:** #2 — feat: write-side mutations + source inspect + health command
**Status:** Approved

---

## Overview

This session ports the write-side CLI mutations and two missing read commands to
tankyu-rs. All changes follow the established hexagonal architecture (Approach B):
mutation methods are added to existing managers in `tankyu-core`; CLI handlers
stay thin (parse → call manager → render). The session also seeds key abstractions
that will evolve into source plugins, idempotent ingest loops, and health
monitoring services as tankyu grows toward an autonomous research graph.

---

## Scope

Eight deliverables in priority order:

1. `source inspect <name>` — full source detail
2. `health` — config-driven source health report (stale / dormant / empty)
3. `topic create <name>` — create a new research topic
4. `source add <url>` — add a source with URL-detection and optional graph link
5. `source remove <name>` — mark a source as pruned
6. `entry update <id>` — patch entry state and/or signal
7. `entry list --unclassified` — entries with no `tagged-with` graph edges
8. `source list` — empty-state hint (minor UX polish to existing command)

---

## Architecture

### Approach: Approach B — mutation methods on existing managers

Managers orchestrate. They do not contain source-type-specific logic. CLI
handlers parse arguments, call managers, and render output. Configuration values
never enter `tankyu-core` — thresholds are passed as plain numeric parameters.

### New files

```
tankyu-core/
  features/source/url_detect.rs      # new — detect_source_type(), name_from_url()
  features/health/mod.rs             # new — HealthManager, HealthReport, HealthWarning

tankyu-cli/
  commands/health.rs                 # new — health::run() handler
  tests/acceptance/features/source.feature   # new BDD
  tests/acceptance/features/topic.feature    # new BDD
  tests/acceptance/features/health.feature   # new BDD
  tests/acceptance/steps/source_steps.rs     # new step defs
  tests/acceptance/steps/topic_steps.rs      # new step defs
  tests/acceptance/steps/health_steps.rs     # new step defs
```

### Modified files

```
tankyu-core/
  features/source/source_manager.rs  # add add(), remove()
  features/source/mod.rs             # re-export url_detect
  features/topic/topic_manager.rs    # add create()
  features/entry/entry_manager.rs    # add update()
  features/mod.rs                    # pub mod health
  shared/error.rs                    # add Duplicate variant to TankyuError

tankyu-cli/
  cli.rs                             # new clap variants
  commands/source.rs                 # add inspect(), add(), remove(); polish list()
  commands/topic.rs                  # add create()
  commands/entry.rs                  # add update(); --unclassified in list()
  commands/mod.rs                    # pub mod health
  main.rs                            # wire new commands
  context.rs                         # add health_mgr, expose graph_store
  tests/cli_source.rs                # extend with new snapshots
  tests/cli_topic.rs                 # extend with new snapshots
  tests/cli_entry.rs                 # extend with new snapshots
  tests/acceptance/features/entry.feature    # extend with new scenarios
  tests/acceptance/steps/entry_steps.rs      # extend step defs
```

---

## Core Domain Design

### `url_detect.rs` — pure functions, no I/O

```rust
pub fn detect_source_type(url: &str) -> SourceType
pub fn name_from_url(url: &str) -> String
```

Pattern matching order (priority matters — must match before more general patterns):

| URL pattern | `SourceType` |
|---|---|
| `github.com/*/*/issues` | `GithubIssues` |
| `github.com/*/*/releases` | `GithubReleases` |
| `github.com/*/*` | `GithubRepo` |
| `github.com/*` | `GithubUser` |
| `x.com/` or `twitter.com/` | `XAccount` |
| `.blog`, `medium.com`, `substack.com`, `dev.to` | `Blog` |
| `/feed`, `/rss`, `/atom`, `.xml` suffix | `RssFeed` |
| `file:` scheme | `AgentReport` |
| fallback | `WebPage` |

`name_from_url` extracts `owner/repo` for GitHub URLs (first two path segments),
single path segment for other paths, or hostname as last resort.

Both functions are `pub` — future source plugins will call them directly.

### `SourceManager` additions

```rust
pub struct AddSourceInput {
    pub url: String,
    pub name: Option<String>,
    pub source_type: Option<SourceType>,
    pub role: Option<SourceRole>,
    pub topic_id: Option<Uuid>,      // CLI resolves name→id before calling add
}

pub async fn add(&self, input: AddSourceInput) -> Result<Source>
pub async fn remove(&self, name: &str) -> Result<Source>
```

**`add` logic:**
1. `store.get_by_url(&input.url)` — if exists:
   - If `input.role` is `Some(r)` and `existing.role != Some(r)`:
     call `store.update(id, SourceUpdate { role: Some(r), .. })` to update role
   - If `topic_id` provided: check for existing `Monitors` edge (dedup), add if absent
   - Return (possibly-updated) existing source (fully idempotent)
2. If new: apply `input.source_type` or `detect_source_type()`, apply
   `input.name` or `name_from_url()`, build `Source` with fresh UUID,
   `state: Active`, zero counters, `Utc::now()` for `created_at`
3. `store.create(source)`
4. If `topic_id` provided: check for existing `Monitors` edge (dedup),
   add if absent via `graph.add_edge()`
5. Return source

**`remove` logic:**
1. `get_by_name(name)` — `TankyuError::NotFound` if absent
2. `store.update(id, SourceUpdate { state: Some(SourceState::Pruned), .. })`
3. Return updated source

### `TopicManager` addition

```rust
pub struct CreateTopicInput {
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
}

pub async fn create(&self, input: CreateTopicInput) -> Result<Topic>
```

**Logic:** `store.get_by_name()` → `TankyuError::Duplicate` if exists → build
`Topic` with new UUID, `Utc::now()` for `created_at` and `updated_at`, zero
`scan_count`, `last_scanned_at: None`, empty `projects` → `store.create()` →
return topic.

### `EntryManager` addition

```rust
pub async fn update(&self, id: Uuid, patch: EntryUpdate) -> Result<Entry>
```

Delegates to `store.update(id, patch)`. The store already handles not-found as
an error. `EntryUpdate` struct already exists with `state`, `signal`, `summary`
fields.

### `HealthManager` — new module at `features/health/mod.rs`

```rust
pub struct HealthManager {
    source_store: Arc<dyn ISourceStore>,
    entry_store: Arc<dyn IEntryStore>,
}

pub struct HealthThresholds {
    pub stale_days: u32,
    pub dormant_days: u32,
}

pub enum HealthWarningKind { Stale, Dormant, Empty }

pub struct HealthWarning {
    pub source_id: Uuid,
    pub source_name: String,
    pub source_type: SourceType,
    pub kind: HealthWarningKind,
    pub detail: String,
}

pub struct HealthReport {
    pub ok: bool,
    pub warnings: Vec<HealthWarning>,
    pub checked_at: DateTime<Utc>,
}

impl HealthManager {
    pub async fn health(&self, thresholds: HealthThresholds) -> Result<HealthReport>
}
```

**`health` logic:**
1. Load all sources; load all entries
2. Build `HashSet<Uuid>` of source IDs that have at least one entry
3. For each source where `state != Pruned`:
   - If `last_checked_at IS NULL` → `Stale` warning (detail: "never checked")
   - Else compute `age_days` = days since `last_checked_at`:
     - If `age_days > dormant_days` → `Dormant` warning
     - Else if `age_days > stale_days` → `Stale` warning
   - If source ID not in entries set → `Empty` warning (independent of above)
4. Return `HealthReport { ok: warnings.is_empty(), warnings, checked_at: Utc::now() }`

Note: A source can have both a staleness warning and an `Empty` warning simultaneously.
Never-checked sources emit `Stale` (not `Dormant`) — we treat absence of data as
stale, not dormant, matching TS behavior. Dormant requires confirmed inactivity over time.

`HealthThresholds` is constructed in `AppContext` from `config.stale_days` and
`config.dormant_days`. Config never enters core.

### `TankyuError` — new `Duplicate` variant

```rust
#[error("duplicate {kind}: '{name}' already exists")]
Duplicate { kind: String, name: String },
// Usage: TankyuError::Duplicate { kind: "topic".into(), name: topic_name }
```

Used by `TopicManager::create()`. `SourceManager::add()` is idempotent (returns
existing source), so it does not use `Duplicate`.

---

## CLI Layer Design

### New clap variants (`cli.rs`)

```
topic create <name> [--description <text>] [--tags <t1,t2>]
source inspect <name>
source add <url> [--name <name>] [--topic <topic>] [--role <role>] [--type <type>]
source remove <name>
entry update <id> [--state <state>] [--signal <signal>]
entry list ... [--unclassified]   # new flag on existing subcommand
health                             # new top-level command
```

### Handler contracts

**`topic::create(ctx, name, description, tags_csv)`**
- Split `tags_csv` on `,`, trim, filter empty
- Call `TopicManager::create()`
- Plain: `Created topic: {name} ({id})` + tags line if non-empty
- JSON: serialize `Topic`
- Error: `Duplicate` → non-zero exit with message

**`source::inspect(ctx, name)`**
- `source_mgr.get_by_name(name)` → error if `None`
- Plain: key-value block (Name, Type, State, Role, URL, Check count, Hit count,
  Miss count, Last checked, Created)
- JSON: serialize `Source`

**`source::add(ctx, url, name, topic, role, source_type)`**
- Parse `role` → `SourceRole` if provided (error on invalid)
- Parse `source_type` → `SourceType` if provided (error on invalid)
- If `--topic`: resolve name → UUID via `topic_mgr.get_by_name()` (error if not found)
- Call `source_mgr.add(AddSourceInput { url, name, source_type, role, topic_id })`
- Plain: `Added source: {name} ({type})` + URL line + ID line + topic line if linked
- JSON: serialize `Source`

**`source::remove(ctx, name)`**
- Call `source_mgr.remove(name)`
- Plain: `Removed source: {name} (marked as pruned)`
- JSON: serialize updated `Source`

**`source::list` — empty-state polish**
- When result is empty:
  - No filters: `No sources yet. Add one with: tankyu source add <url>`
  - `--topic <t>`: `No sources for "{t}". Add one with: tankyu source add <url> --topic {t}`
  - `--role <r>`: `No {r} sources.`

**`entry::update(ctx, id, state, signal)`**
- At least one of `--state` / `--signal` must be provided (error otherwise)
- Parse both if present
- Call `entry_mgr.update(uuid, EntryUpdate { state, signal, summary: None })`
- Plain: `Updated entry: {title}` + changed-field lines
- JSON: serialize `Entry`

**`entry::list` — `--unclassified` path**
- `graph_store.query(GraphQuery { edge_type: Some(TaggedWith), from_type: Some(Entry), .. })`
  → collect `from_id` set of classified entry IDs
- Retain entries where `!classified_ids.contains(&e.id)`
- Mutually exclusive with `--topic` and `--source` (error if combined)

**`health::run(ctx)`**
- Construct `HealthThresholds { stale_days: ctx.config.stale_days, dormant_days: ctx.config.dormant_days }`
- Call `health_mgr.health(thresholds)`
- Plain (ok): `All sources healthy`
- Plain (warnings): table with columns [Kind, Source, Type, Detail]
- JSON: serialize `HealthReport`
- **Exit code 1** if `!report.ok`

### `AppContext` additions

```rust
pub health_mgr: HealthManager,
pub graph_store: Arc<dyn IGraphStore>,  // exposed for --unclassified query
```

---

## Test Strategy

### Layer 1 — Unit tests (inline `#[cfg(test)]` in `tankyu-core`)

**`url_detect.rs`** — one test per `SourceType` variant + mutation killers:
- All 9 source type patterns
- Priority ordering: `github.com/o/r/issues` → `GithubIssues` not `GithubRepo`
- `name_from_url` extracts `owner/repo`, single segment, or hostname

**`source_manager.rs`** — new tests:
- `add` creates new source when URL unseen
- `add` returns existing source when URL already exists (idempotent)
- `add` updates role on existing source if role differs
- `add` with `topic_id` creates `Monitors` edge
- `add` with `topic_id` skips edge if already exists
- `remove` sets state to `Pruned`
- `remove` errors on unknown name

**`topic_manager.rs`** — new tests:
- `create` happy path returns topic with correct fields
- `create` duplicate name returns `TankyuError::Duplicate`

**`entry_manager.rs`** — new tests:
- `update` with state only
- `update` with signal only
- `update` with both state and signal

**`health/mod.rs`** — mutation-targeted tests:
- Source checked within `stale_days` → no warning
- Source not checked in > `stale_days` → `Stale`
- Source not checked in > `dormant_days` → `Dormant` (not `Stale`)
- Source with entries → no `Empty`
- Source with zero entries → `Empty`
- Pruned source → skipped entirely
- Never-checked source → `Stale` (explicit NULL check, not age comparison)
- `ok: true` iff no warnings
- `ok: false` iff any warnings

### Layer 2 — BDD acceptance tests (cucumber `.feature`)

**`source.feature`** (new scenarios):
```
Inspect an existing source
Inspect a non-existent source fails
Add a new source
Add a source linked to a topic
Add duplicate source URL returns existing (idempotent)
Remove a source marks it as pruned
Remove a non-existent source fails
List sources shows empty hint when no sources
List sources with topic filter shows empty hint
```

**`topic.feature`** (new):
```
Create a topic
Create a topic with tags
Create a duplicate topic fails
List topics shows empty hint when no topics
```

**`health.feature`** (new):
```
All sources healthy exits 0
Stale source produces warning and exits 1
Dormant source produces warning and exits 1
Empty source produces warning and exits 1
Never-checked source produces stale warning and exits 1
Pruned source is ignored by health check
Health report as JSON
```

**`entry.feature`** (extended):
```
Update entry state
Update entry signal
Update non-existent entry fails
List unclassified entries excludes classified entries
List unclassified when all classified shows empty
```

World helpers to add to `TankyuWorld`:
- `write_source(id, name, url, state, last_checked_at_days_ago)`
- `write_topic(id, name)`
- `write_tagged_with_edge(entry_id, topic_id)`

### Layer 3 — Insta snapshots

Plain + JSON snapshots for each new command:
- `cli_source.rs`: inspect, add, remove, list-empty
- `cli_topic.rs`: create, create-with-tags
- `cli_entry.rs`: update, list-unclassified
- `cli_health.rs` (new): healthy, warnings, json

Snapshots lock exact rendered output — any formatting regression fails CI.

### Mutation coverage targets

These expressions are the highest mutation risk:
- URL pattern ordering in `detect_source_type` (`&&` / `||` in condition chains)
- `> stale_days` vs `> dormant_days` threshold comparisons in `HealthManager`
- `!= Pruned` skip logic in health loop
- Edge dedup `if !already_exists` in `source add`

Unit tests are written explicitly to kill these mutants.

---

## ADRs

### ADR-1: `source add` is idempotent, `topic create` is not

`source add` returns the existing source silently if the URL already exists.
This matches TS behavior and seeds the idempotent ingest pattern needed for
future autonomous scan loops. `topic create` errors on duplicate because topic
names are user-chosen identifiers — silent dedup would hide mistakes.

### ADR-2: Health thresholds come from config, not hardcoded constants

The TS `health.ts` uses hardcoded 48h / >10 misses. The Rust port uses
`config.stale_days` and `config.dormant_days` — single source of truth,
user-tunable, aligns with the config's intent. Core receives `HealthThresholds`
as a plain struct, never the full config.

### ADR-3: `--unclassified` uses graph traversal, not signal field

Issue description says "entries with no signal." Actual semantic intent (and TS
behavior) is "entries not yet assigned to a topic via `tagged-with` edges."
Graph traversal is the correct implementation. This also seeds the query pattern
for future nanograph integration.

### ADR-4: `source remove` takes `<name>`, not `<id>`

TS takes an ID. The issue spec says `<name>`. Name is the ergonomic CLI
interface; UUID lookup is an implementation detail. This is consistent with all
other by-name lookups in the CLI (`topic inspect`, `source list --topic`, etc.).

Unlike `entry update <id>`, which explicitly uses UUID: entries are not
user-named (their titles are not stable identifiers), so UUID is the only
reliable handle. Sources, by contrast, are routinely referred to by name in CLI
workflows and the name is the natural identifier from the user's perspective.

### ADR-5: `graph_store` exposed on `AppContext`

The `--unclassified` filter requires a direct graph query that no existing
manager exposes. Rather than adding a pass-through method to a manager,
`graph_store` is exposed directly on `AppContext` for CLI-layer use. This is
consistent with how `config` and `data_dir` are already exposed directly.
