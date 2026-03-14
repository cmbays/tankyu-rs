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

#[test]
fn status_json_counts_entries_via_mgr() {
    // Regression guard: verifies AppContext wires entry_mgr (not raw entry_store)
    // by confirming status still works after the refactor
    let dir = create_fixture();
    let output = cmd(&dir).args(["--json", "status"]).output().unwrap();
    assert!(output.status.success());
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["entries"], 1); // entry fixture added in Task 7
}
