# tankyu-rs

Rust port of the [Tankyu](https://github.com/cmbays/tankyu) research intelligence CLI.

> Status: M2 scaffolding in progress. See [tankyu](https://github.com/cmbays/tankyu) for the active TypeScript release.

## Install

```bash
# Homebrew (coming soon)
brew install cmbays/tap/tankyu-rs
```

## Usage

```bash
tankyu status
tankyu topic list
tankyu topic inspect <name>
tankyu source list [--topic <name>]
tankyu config show
tankyu doctor
```

All commands support `--json` for machine-readable output.

## Data Directory

Reads `~/.tankyu/` — same directory used by the TypeScript CLI. Both binaries are compatible.
