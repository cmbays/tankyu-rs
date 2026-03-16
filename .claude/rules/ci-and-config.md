---
paths:
  - ".github/**"
  - "Cargo.toml"
  - "Cargo.lock"
  - "crates/*/Cargo.toml"
---

# CI & Configuration Rules

When working with CI workflows or Cargo manifests:

1. **Workspace dependencies** — shared deps go in `[workspace.dependencies]` in the root `Cargo.toml`; crate-level `Cargo.toml` files reference them with `{ workspace = true }`
2. **Cargo.lock is committed** — this is a binary project, not a library. Always commit lock file changes
3. **CI gates** (in `ci.yml`): `cargo test --all`, `cargo clippy -- -D warnings`, `cargo fmt --check` must all pass
4. **Never skip CI checks** — no `[skip ci]` or `--no-verify` unless explicitly asked
5. **Release workflow** (`release.yml`) is separate — don't modify it when changing CI
6. **Dev-dependencies** go in the crate's `[dev-dependencies]` section, never in workspace `[dependencies]`
