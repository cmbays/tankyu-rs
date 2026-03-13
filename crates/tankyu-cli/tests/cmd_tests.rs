use assert_cmd::Command;
use std::path::Path;
use tempfile::TempDir;

const TOPIC_ID: &str = "11111111-1111-1111-1111-111111111111";
const SOURCE_ID: &str = "22222222-2222-2222-2222-222222222222";

fn create_fixture() -> TempDir {
    let dir = TempDir::new().unwrap();
    let b = dir.path();
    std::fs::create_dir_all(b.join("topics")).unwrap();
    std::fs::create_dir_all(b.join("sources")).unwrap();
    std::fs::create_dir_all(b.join("entries")).unwrap();
    std::fs::create_dir_all(b.join("graph")).unwrap();

    write_json(b.join("config.json"), &serde_json::json!({
        "version": 1,
        "defaultScanLimit": 20,
        "staleDays": 7,
        "dormantDays": 30,
        "llmClassify": false,
        "localRepoPaths": {}
    }));
    write_json(b.join(format!("topics/{TOPIC_ID}.json")), &serde_json::json!({
        "id": TOPIC_ID,
        "name": "rust",
        "description": "Rust programming",
        "tags": ["systems"],
        "projects": [],
        "createdAt": "2025-01-01T00:00:00Z",
        "updatedAt": "2025-01-01T00:00:00Z",
        "lastScannedAt": null,
        "scanCount": 5
    }));
    write_json(b.join(format!("sources/{SOURCE_ID}.json")), &serde_json::json!({
        "id": SOURCE_ID,
        "type": "github-repo",
        "name": "rust-lang/rust",
        "url": "https://github.com/rust-lang/rust",
        "state": "active",
        "discoveredVia": null,
        "discoveryReason": null,
        "lastCheckedAt": null,
        "lastNewContentAt": null,
        "checkCount": 0,
        "hitCount": 0,
        "missCount": 0,
        "createdAt": "2025-01-01T00:00:00Z"
    }));
    write_json(b.join("graph/edges.json"), &serde_json::json!({ "version": 1, "edges": [] }));
    dir
}

fn write_json(path: impl AsRef<Path>, value: &serde_json::Value) {
    std::fs::write(path, serde_json::to_string_pretty(value).unwrap()).unwrap();
}

fn cmd(dir: &TempDir) -> Command {
    let mut c = Command::cargo_bin("tankyu").unwrap();
    c.env("TANKYU_DIR", dir.path());
    c
}

#[test]
fn status_exits_success() {
    let dir = create_fixture();
    cmd(&dir).arg("status").assert().success();
}

#[test]
fn status_json_has_counts() {
    let dir = create_fixture();
    let output = cmd(&dir).args(["--json", "status"]).output().unwrap();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["topics"], 1);
    assert_eq!(v["sources"], 1);
    assert_eq!(v["entries"], 0);
}

#[test]
fn topic_list_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["topic", "list"]).assert().success();
}

#[test]
fn topic_inspect_found() {
    let dir = create_fixture();
    cmd(&dir).args(["topic", "inspect", "rust"]).assert().success();
}

#[test]
fn topic_inspect_missing_fails() {
    let dir = create_fixture();
    cmd(&dir).args(["topic", "inspect", "nope"]).assert().failure();
}

#[test]
fn source_list_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["source", "list"]).assert().success();
}

#[test]
fn config_show_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["config", "show"]).assert().success();
}

#[test]
fn doctor_exits_success() {
    let dir = create_fixture();
    cmd(&dir).arg("doctor").assert().success();
}
