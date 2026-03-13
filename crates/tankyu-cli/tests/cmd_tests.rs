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
