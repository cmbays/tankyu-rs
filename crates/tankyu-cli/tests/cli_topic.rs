mod common;
use common::{cmd, create_fixture};

#[test]
fn topic_create_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["topic", "create", "New-Topic"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "topic create failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Created topic: New-Topic"));
    // Strip dynamic UUID from "Created topic: New-Topic (UUID)"
    let stable: String = stdout
        .lines()
        .map(|l| {
            if l.contains("Created topic:") {
                l.split('(').next().unwrap_or(l).trim_end().to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    insta::assert_snapshot!(stable);
}

#[test]
fn topic_create_with_tags_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["topic", "create", "Systems", "--tags", "rust,c,cpp"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("Tags: rust, c, cpp"));
    // Strip UUID-containing line
    let stable: String = stdout
        .lines()
        .map(|l| {
            if l.contains("Created topic:") {
                l.split('(').next().unwrap_or(l).trim_end().to_string()
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    insta::assert_snapshot!(stable);
}

#[test]
fn topic_create_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "topic", "create", "JSON-Topic"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["name"], "JSON-Topic");
    assert_eq!(v["scanCount"], 0);
    // Dynamic fields (id, createdAt, updatedAt) prevent full snapshot
    insta::assert_snapshot!(format!("name={} tags=[] scanCount=0", v["name"]));
}

#[test]
fn topic_list_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["topic", "list"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn topic_list_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "topic", "list"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn topic_inspect_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["topic", "inspect", "rust"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn topic_inspect_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "topic", "inspect", "rust"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
