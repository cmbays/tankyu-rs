# TDD-First Development Workflow — tankyu-rs

This document is the mandatory process reference for all feature development on tankyu-rs.
Hand it to Claude or a developer at the start of any new feature ticket.
It extends (and is fully compatible with) the `superpowers:test-driven-development` skill.

---

## The Iron Law

```
NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST.
```

`#[derive(...)]` on a new type, trait bounds, and `use` statements are the only lines
that may be written before a test exists.
If you wrote implementation logic before a test: delete it, start over.

---

## Architecture Reference (read before touching any test)

```
crates/
  tankyu-core/
    src/
      domain/
        types.rs        ← All domain structs and enums (serde shapes, camelCase/kebab-case)
        ports.rs        ← Port traits (ITopicStore, IEntryStore, ISourceStore, …)
      features/
        topic/topic_manager.rs   ← Orchestration logic; has inline #[cfg(test)] module
        source/source_manager.rs ← Same pattern with two port dependencies
      infrastructure/
        stores/          ← Real JSON-file implementations of port traits
        graph/           ← JsonGraphStore
    tests/
      store_compat.rs   ← Integration: real stores against fixture JSON files
  tankyu-cli/
    src/
      context.rs        ← AppContext wires managers + stores
      commands/         ← Thin async handlers (call manager, format output)
    tests/
      common/mod.rs     ← create_fixture() + cmd() helpers
      cmd_tests.rs      ← assert_cmd integration: exit codes + JSON field checks
      cli_topic.rs      ← insta snapshot: rendered output (plain + --json)
```

Key invariants from `CLAUDE.md`:
- All structs: `#[serde(rename_all = "camelCase")]`
- All enums:   `#[serde(rename_all = "kebab-case")]`
- Nullable TS fields: `Option<T>` + `#[serde(default)]`
- Optional TS fields: `Option<T>` + `#[serde(skip_serializing_if = "Option::is_none")]`
- `#![forbid(unsafe_code)]` in every crate root

---

## Phase A — Understand Before Writing

Do all of the following before writing the first test line.

1. **Read the relevant port trait** in `ports.rs`.
   Understand which methods the new manager will call.

2. **Read the domain struct** the feature operates on (`types.rs`).
   Note which fields are `Option` (nullable vs optional), which are required.

3. **Read an analogous manager** that already exists.
   For a new read-only manager: `topic_manager.rs`.
   For a manager that joins stores (e.g., uses graph): `source_manager.rs`.

4. **Read `common/mod.rs`** in the CLI test suite.
   Understand what `create_fixture()` provides and what fields are present.

5. **Answer these questions** before opening any file for editing:
   - What is the single new behavior I am adding right now?
   - Which test layer is the entry point? (unit in `#[cfg(test)]`, integration in
     `crates/tankyu-core/tests/`, or e2e in `crates/tankyu-cli/tests/`)
   - What is the failure message I expect to see when the test first runs?

---

## Phase B — Write Failing Tests First

### Layer order for a new feature manager

1. **Unit (in-module)** — test the manager in isolation against a stub store.
2. **Store-compat (integration)** — test the real JSON store parses fixture data
   correctly, once the domain struct is stable.
3. **CLI e2e** — test the binary with `assert_cmd` + `insta` snapshots.

Never advance to the next layer until the previous layer is green.

### Layer order for a new CLI command only (no new manager method)

1. **e2e assert_cmd** — exit-code and JSON-field checks in `cmd_tests.rs`.
2. **e2e snapshot** — `insta::assert_snapshot!` in a dedicated `cli_<noun>.rs` file.

### Concretely: first test for `EntryManager`

The very first test to write is the simplest method — `list`:

```rust
// crates/tankyu-core/src/features/entry/entry_manager.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ports::IEntryStore, types::*};
    use anyhow::Result;
    use async_trait::async_trait;
    use std::sync::Arc;
    use uuid::Uuid;
    use chrono::Utc;

    struct StubEntryStore {
        entries: Vec<Entry>,
    }

    #[async_trait]
    impl IEntryStore for StubEntryStore {
        async fn create(&self, _e: Entry) -> Result<()> { unimplemented!() }
        async fn get(&self, id: Uuid) -> Result<Option<Entry>> {
            Ok(self.entries.iter().find(|e| e.id == id).cloned())
        }
        async fn get_by_url(&self, _url: &str) -> Result<Option<Entry>> { unimplemented!() }
        async fn get_by_content_hash(&self, _h: &str) -> Result<Option<Entry>> { unimplemented!() }
        async fn list_by_source(&self, _sid: Uuid) -> Result<Vec<Entry>> { unimplemented!() }
        async fn list(&self) -> Result<Vec<Entry>> { Ok(self.entries.clone()) }
        async fn update(&self, _id: Uuid, _u: EntryUpdate) -> Result<Entry> { unimplemented!() }
    }

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

    #[tokio::test]
    async fn list_returns_all_entries() {
        let store = Arc::new(StubEntryStore {
            entries: vec![
                make_entry("Alpha", EntryState::New),
                make_entry("Beta", EntryState::Read),
            ],
        });
        let mgr = EntryManager::new(store);
        assert_eq!(mgr.list().await.unwrap().len(), 2);
    }
}
```

Run `cargo test -p tankyu-core entry_manager` — watch it fail because `EntryManager`
does not exist yet. Only then write the struct.

### Concretely: first test for `entry list` CLI command

```rust
// crates/tankyu-cli/tests/cmd_tests.rs  (add to existing file)

#[test]
fn entry_list_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["entry", "list"]).assert().success();
}
```

Run `cargo test -p tankyu-cli entry_list_exits_success` — watch it fail because the
subcommand doesn't exist yet. That's the correct RED.

Then add the snapshot test in a new `cli_entry.rs`:

```rust
// crates/tankyu-cli/tests/cli_entry.rs
mod common;
use common::{cmd, create_fixture};

#[test]
fn entry_list_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "entry", "list"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
```

Snapshots are written on first acceptance review (`cargo insta review`). Until
reviewed and accepted, the test is RED — that is correct.

### Writing a Gherkin scenario first (BDD / ATDD layer)

For features with user-facing behavior, write the `.feature` file before any Rust:

```gherkin
# crates/tankyu-cli/tests/features/entry_filter.feature
Feature: Entry state filtering
  As a researcher
  I want to list my entries filtered by state
  So that I can focus on what needs attention

  Background:
    Given the data directory contains entries:
      | title      | state    |
      | Alpha post | new      |
      | Beta post  | read     |
      | Gamma post | triaged  |

  Scenario: list only new entries
    When I run "tankyu entry list --state new"
    Then the output contains "Alpha post"
    And the output does not contain "Beta post"
    And the output does not contain "Gamma post"

  Scenario: list only read entries
    When I run "tankyu entry list --state read"
    Then the output contains "Beta post"
    And the output does not contain "Alpha post"

  Scenario: invalid state is rejected
    When I run "tankyu entry list --state garbage"
    Then the exit code is non-zero
```

This file must exist and compile before any implementation. The step definitions
are written next, the cucumber runner is hooked up, the tests run RED, and only
then does implementation begin.

---

## Phase C — Implement Minimally

"Minimal" means: the fewest lines of production code that turn all current tests green.

Rules:
- Do not add a method to a manager that has no test for it.
- Do not add a CLI flag that has no test for it.
- Do not add serde fields beyond what is required by a failing test.
- If you find yourself writing a `match` arm for a case no test exercises, stop
  and write the test first.
- `unimplemented!()` is the correct placeholder for port trait methods that the
  current test does not exercise. Leave them as `unimplemented!()` until a test
  forces them to be real.

The stub pattern is canonical for unit tests in this project:

```rust
struct StubXxxStore { /* in-memory data */ }

#[async_trait]
impl IXxxStore for StubXxxStore {
    // Methods the test exercises: real logic
    async fn list(&self) -> Result<Vec<Xxx>> { Ok(self.data.clone()) }
    // Methods no current test exercises: unimplemented!()
    async fn create(&self, _: Xxx) -> Result<()> { unimplemented!() }
}
```

Never use `panic!("not yet")` or `todo!()` — use `unimplemented!()` so that if a
test accidentally calls a stub method it gets a clear failure, not a silent pass.

---

## Phase D — Refactor Safely

Refactor only when all tests are green. Check for these signals:

| Signal | Action |
|---|---|
| `make_xxx` helper duplicated across two test modules | Extract to `crates/tankyu-core/src/domain/test_helpers.rs` (gated `#[cfg(test)]`) |
| Manager method > ~15 lines | Extract private helper; add unit test for helper |
| CLI command handler calls store directly (bypasses manager) | Move logic to manager, add unit test there |
| Stub struct repeated in 3+ test modules | Move to shared test module |
| insta snapshot contains unstable data (timestamps, UUIDs) | Scrub with `insta::Settings` redaction |

During refactor, run after every change:

```bash
cargo test --all
cargo clippy -- -D warnings
```

If any test fails during refactor: revert the refactor, diagnose, fix tests first.

---

## Phase E — Verify and Commit

These commands must all pass clean before any commit.

```bash
# 1. Full test suite
cargo test --all

# 2. Linting (zero warnings)
cargo clippy -- -D warnings

# 3. Formatting
cargo fmt --check

# 4. Review any new/changed snapshots
cargo insta review

# 5. Mutation testing on the changed module (target the specific module)
#    Install once: cargo install cargo-mutants
cargo mutants -p tankyu-core --file src/features/entry/entry_manager.rs

# 6. If mutation score < 80%: add tests, go back to step 1
```

Commit only after step 6 passes. Use a worktree (never switch branches in the main
repo):

```bash
git worktree add ../tankyu-rs-feat-entry feat/entry-manager
```

---

## The TDD Contract — Specific Rules for tankyu-rs

### 1. No implementation without a failing test (with narrow exceptions)

Allowed before a test:
- `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
- `pub struct Foo { ... }` field declarations
- `pub enum Bar { ... }` variant declarations
- `use` and `mod` declarations
- Trait bounds on a struct/impl block

Not allowed before a test:
- Any method body with logic
- `impl Foo { pub fn new(...) -> Self { ... } }`
- Any match arm, if/else, iterator chain

### 2. Required test layers per change type

| Change type | Unit (in-module) | Store-compat | CLI e2e assert_cmd | CLI snapshot |
|---|---|---|---|---|
| New domain struct / enum | — | Required (parse round-trip) | — | — |
| New port trait method | — | Required | — | — |
| New feature manager method | Required | — | — | — |
| New feature manager (full) | Required | — | — | — |
| New CLI subcommand | — | — | Required | Required |
| New CLI flag on existing cmd | — | — | Required | Required |
| Bug fix (any layer) | Required at bug layer | if store-related | if CLI-related | if output changed |

### 3. Stub construction rules

Every `IXxxStore` stub lives in the `#[cfg(test)]` module of the manager that uses it.
Stubs implement the full trait. Methods not under test use `unimplemented!()`.
Stubs never touch the filesystem.

The `StubXxxStore` pattern from `topic_manager.rs` and `source_manager.rs` is canonical.
Copy it, do not invent a new pattern.

### 4. Fixture rules

`common::create_fixture()` owns the CLI test fixture shape. When a new domain entity
needs to appear in CLI tests:
1. Add the fixture JSON to `create_fixture()` in `common/mod.rs`.
2. Add the corresponding parse test to `crates/tankyu-core/tests/store_compat.rs`.
3. Only then add the CLI test.

### 5. Snapshot rules

- Every `cli_<noun>.rs` file tests both `plain` and `json` variants.
- Snapshots are stored in `tests/snapshots/` and committed to git.
- Run `cargo insta review` before every commit to either accept or reject snapshot
  changes. Committing unreviewed snapshots is not allowed.
- Unstable data (timestamps, generated UUIDs) must be redacted with
  `insta::Settings::bind_to_scope()`.

### 6. Mutation score target

80% of mutants must be killed for each modified module.
Run `cargo mutants --file <path>` to check.
If below threshold: add focused tests, do not weaken the assertion.

---

## The BDD Full Cycle — Concrete Example

Feature: "As a researcher, I want to list my entries filtered by state so that I can
focus on what needs attention."

### Step 1 — Write the `.feature` file (Gherkin)

Create `crates/tankyu-cli/tests/features/entry_filter.feature` (shown in Phase B above).
Commit this file alone. It is the specification.

### Step 2 — Write step definitions in Rust

```toml
# Add to crates/tankyu-cli/Cargo.toml [dev-dependencies]
cucumber = "0.20"
```

```rust
// crates/tankyu-cli/tests/entry_filter.rs
use cucumber::{World, given, then, when};

#[derive(Debug, Default, World)]
pub struct EntryWorld {
    dir: Option<tempfile::TempDir>,
    output: String,
    exit_code: Option<i32>,
}

#[given(expr = "the data directory contains entries:")]
async fn setup_entries(world: &mut EntryWorld, step: &cucumber::Step) {
    let dir = common::create_fixture();
    // populate entries from the DataTable in step
    for row in step.table.as_ref().unwrap().rows.iter().skip(1) {
        common::write_entry_fixture(&dir, &row[0], &row[1]);
    }
    world.dir = Some(dir);
}

#[when(expr = "I run {string}")]
async fn run_command(world: &mut EntryWorld, cmd_str: String) {
    let dir = world.dir.as_ref().unwrap();
    let args: Vec<&str> = cmd_str.trim_start_matches("tankyu ").split(' ').collect();
    let out = common::cmd(dir)
        .env("NO_COLOR", "1")
        .args(&args)
        .output()
        .unwrap();
    world.output = String::from_utf8(out.stdout).unwrap();
    world.exit_code = out.status.code();
}

#[then(expr = "the output contains {string}")]
async fn output_contains(world: &mut EntryWorld, expected: String) {
    assert!(
        world.output.contains(&expected),
        "Expected output to contain {expected:?}\nGot:\n{}",
        world.output
    );
}

#[then(expr = "the output does not contain {string}")]
async fn output_not_contains(world: &mut EntryWorld, expected: String) {
    assert!(
        !world.output.contains(&expected),
        "Expected output NOT to contain {expected:?}\nGot:\n{}",
        world.output
    );
}

#[then(expr = "the exit code is non-zero")]
async fn exit_nonzero(world: &mut EntryWorld) {
    assert_ne!(world.exit_code, Some(0));
}

#[tokio::main]
async fn main() {
    EntryWorld::run("tests/features/entry_filter.feature").await;
}
```

Add to `Cargo.toml`:
```toml
[[test]]
name = "entry_filter"
harness = false
```

### Step 3 — Run cucumber → RED

```bash
cargo test --test entry_filter
```

Expected failure: step definitions compile, runner starts, all scenarios fail
because `tankyu entry list --state` is not yet a valid subcommand.

### Step 4 — Implement the feature (inner TDD loop)

For each failing scenario, follow the full red-green-refactor cycle:

4a. Write the unit test for the new manager method `list_by_state`:
```rust
// in entry_manager.rs #[cfg(test)]
#[tokio::test]
async fn list_by_state_filters_correctly() {
    let store = Arc::new(StubEntryStore {
        entries: vec![
            make_entry("Alpha", EntryState::New),
            make_entry("Beta", EntryState::Read),
        ],
    });
    let mgr = EntryManager::new(store);
    let new_only = mgr.list_by_state(EntryState::New).await.unwrap();
    assert_eq!(new_only.len(), 1);
    assert_eq!(new_only[0].title, "Alpha");
}
```

Run → RED. Implement `list_by_state`. Run → GREEN.

4b. Add `--state` flag to `entry list` clap command.
Write CLI assert_cmd test first:
```rust
#[test]
fn entry_list_state_filter_new() {
    let dir = create_fixture_with_entries();
    let out = cmd(&dir).args(["entry", "list", "--state", "new"]).output().unwrap();
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.contains("Alpha post"));
    assert!(!text.contains("Beta post"));
}
```

Run → RED. Implement CLI flag plumbing. Run → GREEN.

4c. Add insta snapshot:
```rust
#[test]
fn entry_list_state_new_plain() {
    let dir = create_fixture_with_entries();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--state", "new"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
```

Run `cargo insta review` to accept. Run again → GREEN.

### Step 5 — Run cucumber → GREEN

```bash
cargo test --test entry_filter
```

All scenarios pass.

### Step 6 — Run mutation testing

```bash
cargo mutants -p tankyu-core --file src/features/entry/entry_manager.rs
```

If any `list_by_state` mutant survives (e.g., changing `==` to `!=` in the filter
predicate and all tests still pass), add a targeted test that kills it.

### Step 7 — Final gate

```bash
cargo test --all
cargo clippy -- -D warnings
cargo fmt --check
cargo insta review   # ensure nothing unreviewed
```

All clean. Commit.

---

## Tooling Checklist

### Already present in workspace

| Tool / Crate | Location | Purpose |
|---|---|---|
| `tokio` | workspace deps | async test runtime (`#[tokio::test]`) |
| `insta` | workspace dev-deps | Snapshot testing |
| `assert_cmd` | workspace dev-deps | Binary e2e testing |
| `tempfile` | workspace dev-deps | Temporary data directories in tests |
| `proptest` | `tankyu-core` dev-deps | Property-based testing |
| `serde_json` | workspace deps | JSON assertion in cmd_tests |

### Must be added for BDD

```toml
# workspace [workspace.dependencies]
cucumber = "0.20"

# crates/tankyu-cli/Cargo.toml [dev-dependencies]
cucumber = { workspace = true }
```

Each cucumber test binary requires `harness = false` in its `[[test]]` block.

### Must be installed for mutation testing

```bash
cargo install cargo-mutants
```

Run per-module, not workspace-wide (workspace runs are slow and noisy):

```bash
cargo mutants -p tankyu-core --file src/features/entry/entry_manager.rs
cargo mutants -p tankyu-core --file src/infrastructure/stores/entry_store.rs
```

### Optional but recommended

```bash
cargo install cargo-nextest   # faster test runner: cargo nextest run --all
cargo install cargo-llvm-cov  # line coverage: cargo llvm-cov --all
```

---

## Quick Reference — Commands

```bash
# Run all tests
cargo test --all

# Run only core unit tests
cargo test -p tankyu-core

# Run only CLI e2e tests
cargo test -p tankyu-cli

# Run a specific test
cargo test -p tankyu-core list_by_state

# Lint
cargo clippy -- -D warnings

# Format check
cargo fmt --check

# Apply formatting
cargo fmt

# Review snapshot changes
cargo insta review

# Accept all new snapshots non-interactively (CI only)
cargo insta accept

# Run BDD cucumber tests
cargo test --test entry_filter

# Run mutation testing on a specific file
cargo mutants -p tankyu-core --file src/features/entry/entry_manager.rs
```

---

## Anti-Patterns — Never Do These

- Writing `impl EntryManager` before a test for `EntryManager` exists.
- Adding a `list_by_state` method and calling it in the CLI handler, with only
  a CLI test (no unit test for the manager method).
- Using `todo!()` or `panic!()` in stubs (use `unimplemented!()`).
- Keeping a snapshot in a "pending review" state and committing.
- Writing tests after the implementation and rationalizing it as equivalent.
- Sharing mutable state between tests (each test calls `create_fixture()` fresh).
- Touching real files in unit tests (unit tests never touch disk; only
  `store_compat.rs` and CLI tests are allowed to).
- Adding fields to a domain struct without a round-trip `serde` test in `types.rs`.
