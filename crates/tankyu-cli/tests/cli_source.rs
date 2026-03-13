mod common;
use common::{cmd, create_fixture};

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
