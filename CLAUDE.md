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

## Key Conventions

- `#![forbid(unsafe_code)]` in every crate root
- `serde(rename_all = "camelCase")` on all structs (matches TypeScript JSON)
- `serde(rename_all = "kebab-case")` on all enums (matches TypeScript enum values)
- Nullable fields (`.nullable()` in TS): `Option<T>` + `#[serde(default)]`
- Optional fields (`.optional()` in TS): `Option<T>` with no extra annotation
- Async traits via `async-trait` crate
- Error handling: `TankyuError` (thiserror) in core, `anyhow` at CLI boundary
