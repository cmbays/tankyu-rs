mod common;
use common::{cmd, create_fixture};

#[test]
fn config_show_plain() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .env("NO_COLOR", "1")
        .args(["config", "show"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}

#[test]
fn config_show_json() {
    let dir = create_fixture();
    let out = cmd(&dir)
        .args(["--json", "config", "show"])
        .output()
        .unwrap();
    insta::assert_snapshot!(String::from_utf8(out.stdout).unwrap());
}
