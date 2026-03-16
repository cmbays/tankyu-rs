# tankyu-rs

Rust port of the [Tankyu](https://github.com/cmbays/tankyu) research intelligence CLI.

> Status: Core read + write commands implemented. See [tankyu](https://github.com/cmbays/tankyu) for the active TypeScript release.

## Install

```bash
cargo install --path crates/tankyu-cli
```

## Usage

### Read Commands

```bash
tankyu status                              # Dashboard — topic/source/entry counts
tankyu topic list                          # List all research topics
tankyu topic inspect <name>                # Full topic detail
tankyu source list [--topic <name>]        # List sources, optionally by topic
tankyu entry list [OPTIONS]                # List entries with filters
tankyu entry inspect <id>                  # Full entry detail
tankyu health                              # Check source health (stale, dormant, empty)
tankyu config show                         # Show configuration
tankyu doctor                              # Verify data directory
```

#### Entry list filters

```bash
tankyu entry list --state <state>          # Filter by state (e.g., unread, read)
tankyu entry list --signal <signal>        # Filter by signal (e.g., high, medium, low)
tankyu entry list --source <name>          # Filter by source name
tankyu entry list --topic <name>           # Filter by topic name
tankyu entry list --unclassified           # Show entries with no topic
tankyu entry list --limit <n>              # Limit results
```

### Write Commands

```bash
tankyu topic create <name> [--description "..."] [--tags "a,b,c"]
tankyu source add <url> [--name x] [--topic t] [--role r] [--source-type t]
tankyu source remove <name>                # Mark source as pruned
tankyu entry update <id> --state <state>   # Update entry state
tankyu entry update <id> --signal <signal> # Update entry signal
```

All commands support `--json` for machine-readable output.

## Data Directory

Reads `~/.tankyu/` — same directory used by the TypeScript CLI. Both binaries are compatible.

Set `TANKYU_DIR` to override (e.g., `TANKYU_DIR=/tmp/test-data tankyu status`).

## Development

```bash
cargo build                    # Debug build
cargo test --all               # Run all tests (unit, integration, BDD, snapshots)
cargo clippy -- -D warnings    # Lint (must be clean)
cargo fmt                      # Format
cargo insta review             # Review snapshot changes
```

Cargo workspace: `tankyu-core` (domain + ports) and `tankyu-cli` (clap binary + rendering).
