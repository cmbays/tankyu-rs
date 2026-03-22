mod common;
use common::{cmd, create_fixture, write_json};

#[test]
fn source_list_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "list"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn source_list_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "source", "list"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn source_inspect_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "inspect", "rust-lang-rust"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "source inspect failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn source_inspect_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "source", "inspect", "rust-lang-rust"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["name"], "rust-lang-rust");
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn source_add_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "add", "https://github.com/tokio-rs/tokio"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "source add failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("tokio-rs-tokio"));
    // Dynamic ID — strip the ID line before snapshotting
    let stable: String = stdout
        .lines()
        .filter(|l| !l.trim_start().starts_with("ID:"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    insta::assert_snapshot!(stable);
}

#[test]
fn source_list_empty_plain() {
    let dir = tempfile::TempDir::new().unwrap();
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
        b.join("graph/edges.json"),
        &serde_json::json!({ "version": 1, "edges": [] }),
    );
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["source", "list"])
        .output()
        .unwrap();
    assert!(out.status.success());
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
