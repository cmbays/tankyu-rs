---
paths:
  - "crates/*/tests/**"
  - "**/*.feature"
  - "TDD_WORKFLOW.md"
---

# Testing Rules

When working with tests:

1. **TDD is mandatory** — read `TDD_WORKFLOW.md` before writing any production code. The iron law: no production code without a failing test first
2. **Test pyramid order**: unit (stub stores) → store-compat (tempdir round-trips) → CLI integration (assert_cmd) → snapshots (insta) → BDD acceptance (cucumber 0.22)
3. **BDD with cucumber 0.22** — `.feature` files live in `tests/acceptance/features/`, step defs in `tests/acceptance/steps/`. The test binary uses `harness = false`
4. **Insta snapshots** — use `assert_snapshot!` for CLI output. Run `cargo insta review` after changes. Never manually edit snapshot files
5. **Mutation testing** — run `cargo mutants` on changed manager files. 80% kill rate required before merging
6. **Test data**: use `TANKYU_DIR` env var pointing to a tempdir with fixture JSON — never read from `~/.tankyu/` in tests
7. **`common/mod.rs`** in CLI tests provides shared fixture builders — reuse existing helpers before creating new ones
8. **proptest** for domain roundtrip properties — extend existing suites when adding new domain types
