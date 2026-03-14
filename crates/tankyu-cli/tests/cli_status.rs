mod common;
use common::{cmd, create_fixture};

#[test]
fn status_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .arg("status")
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!(stdout);
}

#[test]
fn status_json() {
    let dir = create_fixture();
    let out = cmd(&dir).args(["--json", "status"]).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    insta::assert_snapshot!(stdout);
}
