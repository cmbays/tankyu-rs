mod common;
use common::{cmd, create_fixture, write_json, SOURCE_ID};

fn patch_source_recently_checked(dir: &tempfile::TempDir) {
    // Use a timestamp that is always within the stale threshold (7 days).
    // We compute "now minus 1 day" at test runtime so the fixture stays valid
    // regardless of when the test runs.
    let recently = (chrono::Utc::now() - chrono::Duration::days(1))
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string();
    write_json(
        dir.path().join(format!("sources/{SOURCE_ID}.json")),
        &serde_json::json!({
            "id": SOURCE_ID, "type": "github-repo", "name": "rust-lang/rust",
            "url": "https://github.com/rust-lang/rust", "state": "active",
            "discoveredVia": null, "discoveryReason": null,
            "lastCheckedAt": recently, "lastNewContentAt": null,
            "checkCount": 1, "hitCount": 0, "missCount": 0,
            "createdAt": "2025-01-01T00:00:00Z"
        }),
    );
}

#[test]
fn health_all_healthy_plain() {
    let dir = create_fixture();
    patch_source_recently_checked(&dir);
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["health"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "health should exit 0 when healthy: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn health_never_checked_stale_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["health"])
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "health should exit 1 when there are stale sources"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("stale"), "expected stale warning: {stdout}");
    assert!(
        stdout.contains("never checked"),
        "expected never-checked detail: {stdout}"
    );
    insta::assert_snapshot!(stdout);
}

#[test]
fn health_json() {
    let dir = create_fixture();
    patch_source_recently_checked(&dir);
    let out = cmd(&dir).args(["--json", "health"]).output().unwrap();
    assert!(
        out.status.success(),
        "health --json should exit 0 when healthy"
    );
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["warnings"].as_array().unwrap().is_empty());
    // Strip dynamic checkedAt before snapshotting
    let mut stable = v.clone();
    stable["checkedAt"] = serde_json::Value::String("<dynamic>".to_string());
    insta::assert_snapshot!(serde_json::to_string_pretty(&stable).unwrap());
}
