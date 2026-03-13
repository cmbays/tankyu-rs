use assert_cmd::Command;

#[test]
fn cli_help_shows_all_subcommands() {
    let output = Command::cargo_bin("tankyu")
        .unwrap()
        .arg("--help")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("status"), "help should list 'status'");
    assert!(stdout.contains("topic"), "help should list 'topic'");
    assert!(stdout.contains("source"), "help should list 'source'");
    assert!(stdout.contains("config"), "help should list 'config'");
    assert!(stdout.contains("doctor"), "help should list 'doctor'");
}
