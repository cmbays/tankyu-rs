# Architecture Rules

## Port Traits

1. **Per-feature port traits** — each feature/use-case defines its own port trait. Never create a single god trait for all graph operations.
2. **Port trait methods must use domain types only** — if a method signature contains query text, query names, or generic parameter maps, the abstraction is wrong.
3. **Port traits must have ≤5 methods** — if more are needed, split into separate per-consumer traits.

## Layer Boundaries

4. **No `include_str!` across module boundaries** — query files, schema files, and templates must only be referenced by the infrastructure module that owns them. Features must never use `include_str!("../../infrastructure/...")`.
5. **Features must not import from infrastructure** — except in `#[cfg(test)]` blocks for integration tests using `open_in_memory()`.
6. **Infrastructure owns all engine-specific details** — query syntax, query file paths, engine configuration. Features see only domain-typed trait methods.

## Error Handling

7. **Port traits return `Result<T, TankyuError>`** — never `anyhow::Result` in port trait signatures.
