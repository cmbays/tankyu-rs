mod common;
use common::{cmd, create_fixture};

#[test]
fn doctor_plain() {
    let dir = create_fixture();
    let out = cmd(&dir).env("NO_COLOR", "1").arg("doctor").output().unwrap();
    // Snapshot only stdout (data_dir path is dynamic — strip it)
    let stdout = String::from_utf8(out.stdout).unwrap();
    let stable = stdout
        .lines()
        .map(|l| {
            if l.contains("Data dir:") {
                "  Data dir: <redacted>"
            } else {
                l
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"; // restore the trailing newline stripped by .lines()
    insta::assert_snapshot!(stable);
}

#[test]
fn doctor_json() {
    let dir = create_fixture();
    let out = cmd(&dir).args(["--json", "doctor"]).output().unwrap();
    // Strip dynamic data_dir before snapshotting
    let mut v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    v["data_dir"] = serde_json::Value::String("<redacted>".into());
    insta::assert_snapshot!(serde_json::to_string_pretty(&v).unwrap());
}
