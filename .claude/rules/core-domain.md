---
paths:
  - "crates/tankyu-core/**"
---

# Core Domain Rules

When working with tankyu-core:

1. **`#![forbid(unsafe_code)]`** must remain in `lib.rs` — never add `unsafe` blocks
2. **Serde struct convention**: `#[serde(rename_all = "camelCase")]` on all structs — the JSON files are produced by TypeScript and use camelCase
3. **Serde enum convention**: `#[serde(rename_all = "kebab-case")]` on all enums — matches TypeScript enum serialization
4. **Nullable vs optional fields**: TS `.nullable()` → `Option<T>` + `#[serde(default)]`; TS `.optional()` → `Option<T>` with no extra attribute
5. **Port traits** (in `domain/ports/`) define the storage interface — never add methods to a port trait just for convenience; prefer manager-level logic over widening the trait surface
6. **Error handling**: all fallible operations return `Result<T, TankyuError>` using `thiserror` — never use `anyhow` in core
7. **Feature managers** take `Arc<dyn Port>` dependencies — never take concrete store types; this keeps test stubs simple
8. **No manager-to-manager dependencies** — if a manager needs data from another domain, take the shared port (`Arc<dyn IGraphStore>`) directly
9. **Async traits** use the `async-trait` crate — all port trait methods are `async fn`
