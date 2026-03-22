# CLAUDE.md — tankyu-rs

Rust port of Tankyu research intelligence CLI. Package: `tankyu`.

## Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build
cargo test --all               # Run all tests
cargo test -p tankyu-core      # Core tests only
cargo clippy -- -D warnings    # Lint (must be clean)
cargo fmt                      # Format
cargo insta review             # Review snapshot changes
```

## Architecture

Cargo workspace with two crates:
- `tankyu-core` — domain types, ports (traits), persistence (JSON stores), features
- `tankyu-cli` — clap binary, output rendering, command handlers

Reads `~/.tankyu/` data directory produced by the TypeScript CLI.
Set `TANKYU_DIR` env var to override the data directory (used in tests).

## Language Context

Language: Rust
Framework: clap (CLI)
Idiom reference: `~/.claude/knowledge/languages/rust/`

Consult the idiom reference before making architectural decisions (trait design, module boundaries, port abstractions).

## Key Conventions

- `#![forbid(unsafe_code)]` in every crate root
- `serde(rename_all = "camelCase")` on all structs (matches TypeScript JSON)
- `serde(rename_all = "kebab-case")` on all enums (matches TypeScript enum values)
- Nullable fields (`.nullable()` in TS): `Option<T>` + `#[serde(default)]`
- Optional fields (`.optional()` in TS): `Option<T>` with no extra annotation
- Async traits via `async-trait` crate
- Error handling: `TankyuError` (thiserror) in core, `anyhow` at CLI boundary

## Rule Maintenance

Scoped rules live in `.claude/rules/`. When you discover a new pattern, convention, or footgun during development:

1. Add it to the matching rule file (by `paths:` scope)
2. If no rule file matches, create one with appropriate `paths:` globs
3. Keep rules concise — enforcement language ("Never", "Always", "Must"), not explanations

## Compact Instructions

When context is compressed during long sessions, **preserve**:

- Current task objective and acceptance criteria
- Active file paths being modified
- Test results (pass/fail state, which tests)
- TDD phase (red/green/refactor) and what's next
- Any user decisions or corrections from this session

**Discard**:

- File contents read more than 5 tool calls ago (re-read if needed)
- Exploratory searches that didn't yield results
- Intermediate build output (only keep final pass/fail)
- Plan text if already being executed (keep current step only)
