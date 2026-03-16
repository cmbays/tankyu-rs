---
paths:
  - "crates/tankyu-cli/src/**"
---

# CLI Rules

When working with tankyu-cli source code:

1. **Thin handlers only** — command handlers parse args, call a manager method, and render output. No domain logic in the CLI crate
2. **Error boundary**: use `anyhow` at the CLI boundary — convert `TankyuError` to user-friendly messages, never expose internal error types to stderr
3. **Clap 4 derive style** — all commands use `#[derive(Parser)]` / `#[derive(Subcommand)]`, not builder pattern
4. **Table output**: use `comfy-table` for tabular rendering — match existing column patterns (short UUID, kebab-case enums, truncated titles)
5. **JSON output**: `--json` flag uses `serde_json::to_string_pretty` on the domain type directly — no wrapper envelopes
6. **`AppContext`** owns all managers — never expose raw stores through context; commands access `ctx.entry_mgr`, `ctx.source_mgr`, etc.
7. **Mutual exclusion**: when CLI flags conflict (e.g., `--topic` + `--source`), fail with a clear error message before calling any manager method
