use crate::world::TankyuWorld;
use cucumber::{given, then, when};

#[given("the data directory contains 3 entries with mixed state")]
#[allow(clippy::needless_pass_by_ref_mut)]
fn given_three_entries(world: &mut TankyuWorld) {
    world.write_entry(
        "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
        "Alpha entry",
        "new",
        Some("high"),
    );
    world.write_entry(
        "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
        "Beta entry",
        "read",
        Some("low"),
    );
    world.write_entry(
        "cccccccc-cccc-cccc-cccc-cccccccccccc",
        "Gamma entry",
        "triaged",
        None,
    );
}

#[when(expr = "I run {string}")]
#[allow(clippy::needless_pass_by_value)]
fn when_run(world: &mut TankyuWorld, cmd_str: String) {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    world.run_cmd(&parts);
}

#[then("the command exits successfully")]
#[allow(clippy::needless_pass_by_ref_mut)]
fn then_exits_success(world: &mut TankyuWorld) {
    assert_eq!(
        world.last_exit_code,
        Some(0),
        "Expected exit 0, got {:?}\nstdout: {}\nstderr: {}",
        world.last_exit_code,
        world.last_stdout,
        world.last_stderr
    );
}

#[then("the command exits with failure")]
#[allow(clippy::needless_pass_by_ref_mut)]
fn then_exits_failure(world: &mut TankyuWorld) {
    assert_ne!(
        world.last_exit_code,
        Some(0),
        "Expected non-zero exit\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stdout contains {string}")]
#[allow(clippy::needless_pass_by_ref_mut)]
#[allow(clippy::needless_pass_by_value)]
fn then_stdout_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        world.last_stdout.contains(&needle),
        "stdout did not contain {needle:?}\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stdout does not contain {string}")]
#[allow(clippy::needless_pass_by_ref_mut)]
#[allow(clippy::needless_pass_by_value)]
fn then_stdout_not_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        !world.last_stdout.contains(&needle),
        "stdout should NOT contain {needle:?}\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stderr contains {string}")]
#[allow(clippy::needless_pass_by_ref_mut)]
#[allow(clippy::needless_pass_by_value)]
fn then_stderr_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        world
            .last_stderr
            .to_lowercase()
            .contains(&needle.to_lowercase()),
        "stderr did not contain {needle:?}\nstderr: {}",
        world.last_stderr
    );
}

#[then(expr = "stdout is a JSON array of length {int}")]
#[allow(clippy::needless_pass_by_ref_mut)]
fn then_json_array_length(world: &mut TankyuWorld, len: i64) {
    let v: serde_json::Value =
        serde_json::from_str(&world.last_stdout).expect("stdout is not valid JSON");
    let arr = v.as_array().expect("stdout is not a JSON array");
    let expected_len = usize::try_from(len).expect("array length must be non-negative");
    assert_eq!(
        arr.len(),
        expected_len,
        "expected {expected_len} items, got {}",
        arr.len()
    );
}
