mod common;
use common::{cmd, create_fixture};

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
