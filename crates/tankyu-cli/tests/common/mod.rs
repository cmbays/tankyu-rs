use std::path::Path;

use assert_cmd::Command;
use tempfile::TempDir;

pub const TOPIC_ID: &str = "11111111-1111-1111-1111-111111111111";
pub const SOURCE_ID: &str = "22222222-2222-2222-2222-222222222222";
pub const ENTRY_ID: &str = "33333333-3333-3333-3333-333333333333";

pub fn create_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let b = dir.path();
    std::fs::create_dir_all(b.join("topics")).unwrap();
    std::fs::create_dir_all(b.join("sources")).unwrap();
    std::fs::create_dir_all(b.join("entries")).unwrap();
    std::fs::create_dir_all(b.join("graph")).unwrap();

    write_json(
        b.join("config.json"),
        &serde_json::json!({
            "version": 1, "defaultScanLimit": 20, "staleDays": 7,
            "dormantDays": 30, "llmClassify": false, "localRepoPaths": {}
        }),
    );
    write_json(
        b.join(format!("topics/{TOPIC_ID}.json")),
        &serde_json::json!({
            "id": TOPIC_ID, "name": "rust", "description": "Rust programming",
            "tags": ["systems"], "projects": [],
            "createdAt": "2025-01-01T00:00:00Z", "updatedAt": "2025-01-01T00:00:00Z",
            "lastScannedAt": null, "scanCount": 5
        }),
    );
    write_json(
        b.join(format!("sources/{SOURCE_ID}.json")),
        &serde_json::json!({
            "id": SOURCE_ID, "type": "github-repo", "name": "rust-lang/rust",
            "url": "https://github.com/rust-lang/rust", "state": "active",
            "discoveredVia": null, "discoveryReason": null,
            "lastCheckedAt": null, "lastNewContentAt": null,
            "checkCount": 0, "hitCount": 0, "missCount": 0,
            "createdAt": "2025-01-01T00:00:00Z"
        }),
    );
    write_json(
        b.join(format!("entries/{ENTRY_ID}.json")),
        &serde_json::json!({
            "id": ENTRY_ID,
            "sourceId": SOURCE_ID,
            "type": "commit",
            "title": "feat: add entry management",
            "url": "https://github.com/rust-lang/rust/commit/abc123",
            "summary": null,
            "contentHash": null,
            "state": "new",
            "signal": "high",
            "scannedAt": "2025-01-15T10:00:00Z",
            "metadata": null,
            "createdAt": "2025-01-15T10:00:00Z"
        }),
    );
    write_json(
        b.join("graph/edges.json"),
        &serde_json::json!({ "version": 1, "edges": [] }),
    );
    dir
}

pub fn write_json(path: impl AsRef<Path>, value: &serde_json::Value) {
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

pub fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("tankyu").unwrap();
    c.env("TANKYU_DIR", dir.path());
    c
}

/// Write an additional entry fixture to an existing fixture dir.
pub fn write_entry(
    dir: &TempDir,
    id: &str,
    title: &str,
    state: &str,
    signal: Option<&str>,
) {
    write_json(
        dir.path().join(format!("entries/{id}.json")),
        &serde_json::json!({
            "id": id,
            "sourceId": SOURCE_ID,
            "type": "article",
            "title": title,
            "url": format!("https://example.com/{id}"),
            "summary": null,
            "contentHash": null,
            "state": state,
            "signal": signal,
            "scannedAt": "2025-01-15T10:00:00Z",
            "metadata": null,
            "createdAt": "2025-01-15T10:00:00Z"
        }),
    );
}
