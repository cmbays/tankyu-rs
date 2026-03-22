mod common;
use common::{cmd, create_fixture, write_entry, ENTRY_ID};

#[test]
fn entry_update_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "update", ENTRY_ID, "--state", "read"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "entry update failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_unclassified_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--unclassified"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "entry list --unclassified failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("feat: add entry management"),
        "fixture entry must be unclassified: {stdout}"
    );
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_list_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "entry list failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "entry", "list"])
        .output()
        .unwrap();
    // The fixture uses fixed ISO timestamps so the snapshot is deterministic.
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    let arr = v.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["id"], ENTRY_ID);
    assert_eq!(arr[0]["state"], "new");
    assert_eq!(arr[0]["signal"], "high");
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_list_filtered_by_state() {
    let dir = create_fixture();
    // Add a second entry with different state
    write_entry(
        &dir,
        "44444444-4444-4444-4444-444444444444",
        "A read entry",
        "read",
        None,
    );
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--state", "new"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("feat: add entry management"));
    assert!(!stdout.contains("A read entry"));
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_list_filtered_by_signal() {
    let dir = create_fixture();
    write_entry(
        &dir,
        "55555555-5555-5555-5555-555555555555",
        "Low signal entry",
        "new",
        Some("low"),
    );
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--signal", "high"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("feat: add entry management"));
    assert!(!stdout.contains("Low signal entry"));
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_list_filtered_by_source() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "list", "--source", "rust-lang-rust"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "entry list --source failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.contains("feat: add entry management"),
        "expected fixture entry in output: {stdout}"
    );
    insta::assert_snapshot!(stdout);
}

#[test]
fn entry_inspect_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["entry", "inspect", ENTRY_ID])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "entry inspect failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn entry_inspect_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "entry", "inspect", ENTRY_ID])
        .output()
        .unwrap();
    assert!(out.status.success());
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["id"], ENTRY_ID);
    assert_eq!(v["state"], "new");
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
