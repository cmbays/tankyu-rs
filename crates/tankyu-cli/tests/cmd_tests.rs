mod common;
use common::{cmd, create_fixture};

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
    assert_eq!(v["entries"], 1);
}

#[test]
fn topic_list_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["topic", "list"]).assert().success();
}

#[test]
fn topic_inspect_found() {
    let dir = create_fixture();
    cmd(&dir)
        .args(["topic", "inspect", "rust"])
        .assert()
        .success();
}

#[test]
fn topic_inspect_missing_fails() {
    let dir = create_fixture();
    cmd(&dir)
        .args(["topic", "inspect", "nope"])
        .assert()
        .failure();
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

#[test]
fn entry_list_exits_success() {
    let dir = create_fixture();
    cmd(&dir).args(["entry", "list"]).assert().success();
}

#[test]
fn entry_list_json_is_array() {
    let dir = create_fixture();
    let output = cmd(&dir)
        .args(["--json", "entry", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(v.is_array());
}

#[test]
fn entry_inspect_missing_fails() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "inspect", "00000000-0000-0000-0000-000000000000"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("not found"),
        "expected 'not found' in stderr, got: {stderr}"
    );
}

#[test]
fn entry_list_invalid_state_fails() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "list", "--state", "garbage"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("Invalid state"),
        "expected 'Invalid state' in stderr, got: {stderr}"
    );
}

#[test]
fn entry_list_source_and_topic_mutual_exclusion() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["entry", "list", "--source", "foo", "--topic", "bar"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("mutually exclusive"),
        "expected 'mutually exclusive' in stderr, got: {stderr}"
    );
}
