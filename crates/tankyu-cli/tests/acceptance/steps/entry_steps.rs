// cucumber step macros require `&mut World` even for read-only steps, and
// `String` captures by value — suppress the resulting pedantic lints globally.
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::{TankyuWorld, DEFAULT_SOURCE_ID};
use cucumber::{given, then, when};

// ── Existing Given steps ──────────────────────────────────────────────

#[given(expr = "entry {string} is classified under topic {string}")]
fn given_entry_classified(world: &mut TankyuWorld, entry_id: String, topic_id: String) {
    world.write_topic(&topic_id, "Test Topic");
    world.write_tagged_with_edge(&entry_id, &topic_id);
}

#[given("the data directory contains 3 entries with mixed state")]
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

// ── New Given steps for entry.feature ─────────────────────────────────

#[given("entries exist in the research graph")]
fn given_entries_exist(world: &mut TankyuWorld) {
    world.write_source(
        DEFAULT_SOURCE_ID,
        "test-source",
        "https://example.com",
        "active",
        Some(1),
    );
    world.write_named_entry("Entry Alpha", DEFAULT_SOURCE_ID, "new", Some("high"));
    world.write_named_entry("Entry Beta", DEFAULT_SOURCE_ID, "read", None);
}

#[given(expr = "a source {string} exists with entries")]
fn given_source_with_entries(world: &mut TankyuWorld, name: String) {
    let id = uuid::Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &name,
        &format!("https://example.com/{name}"),
        "active",
        Some(1),
    );
    write_entry_pair_for_source(world, &name, &id);
}

#[given(expr = "a topic {string} exists monitoring source {string}")]
fn given_topic_monitoring_source(world: &mut TankyuWorld, topic_name: String, source_name: String) {
    let topic_id = uuid::Uuid::new_v4().to_string();
    world.write_topic(&topic_id, &topic_name);
    let source_id = find_or_create_source(world, &source_name);
    world.write_monitors_edge(&topic_id, &source_id);
}

#[given(expr = "entries exist for source {string}")]
fn given_entries_for_source(world: &mut TankyuWorld, source_name: String) {
    let source_id = find_or_create_source(world, &source_name);
    write_entry_pair_for_source(world, &source_name, &source_id);
}

#[given(regex = r#"^(?:an )?entry "([^"]+)" is tagged with topic "([^"]+)"$"#)]
fn given_entry_tagged(world: &mut TankyuWorld, entry_slug: String, topic_name: String) {
    if !world.entry_ids.contains_key(&entry_slug) {
        world.write_named_entry(&entry_slug, DEFAULT_SOURCE_ID, "new", None);
    }
    let entry_id = world.entry_ids[&entry_slug].clone();

    let topic_id = world
        .topic_ids
        .get(&topic_name)
        .cloned()
        .unwrap_or_else(|| {
            let tid = uuid::Uuid::new_v4().to_string();
            world.write_topic(&tid, &topic_name);
            tid
        });

    world.write_tagged_with_edge(&entry_id, &topic_id);
}

#[given(expr = "an entry {string} has no topic tags")]
fn given_entry_no_tags(world: &mut TankyuWorld, entry_slug: String) {
    if !world.entry_ids.contains_key(&entry_slug) {
        world.write_named_entry(&entry_slug, DEFAULT_SOURCE_ID, "new", None);
    }
}

#[given("no entries are tagged with any topic")]
#[allow(clippy::missing_const_for_fn)]
fn given_no_entries_tagged(world: &mut TankyuWorld) {
    // No-op: the world starts with no edges, so all entries are unclassified.
    let _ = world;
}

#[given(expr = "an entry {string} exists with source {string}")]
fn given_entry_with_source(world: &mut TankyuWorld, entry_slug: String, source_name: String) {
    let source_id = find_or_create_source(world, &source_name);
    world.write_named_entry(&entry_slug, &source_id, "new", None);
}

#[given(expr = "an entry {string} exists")]
fn given_entry_exists(world: &mut TankyuWorld, entry_slug: String) {
    if !world.entry_ids.contains_key(&entry_slug) {
        ensure_default_source(world);
        world.write_named_entry(&entry_slug, DEFAULT_SOURCE_ID, "new", None);
    }
}

// ── When steps ────────────────────────────────────────────────────────

#[when(expr = "I run {string}")]
fn when_run(world: &mut TankyuWorld, cmd_str: String) {
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    world.run_cmd(&parts);
}

// ── Existing Then steps ───────────────────────────────────────────────

#[then("the command exits successfully")]
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
fn then_exits_failure(world: &mut TankyuWorld) {
    assert_ne!(
        world.last_exit_code,
        Some(0),
        "Expected non-zero exit\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stdout contains {string}")]
fn then_stdout_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        world.last_stdout.contains(&needle),
        "stdout did not contain {needle:?}\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stdout does not contain {string}")]
fn then_stdout_not_contains(world: &mut TankyuWorld, needle: String) {
    assert!(
        !world.last_stdout.contains(&needle),
        "stdout should NOT contain {needle:?}\nstdout: {}",
        world.last_stdout
    );
}

#[then(expr = "stderr contains {string}")]
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

// ── New Then steps for entry.feature ──────────────────────────────────

#[then("stdout contains the entry titles")]
fn then_stdout_contains_entry_titles(world: &mut TankyuWorld) {
    assert_all_tracked_titles_in_stdout(world);
}

#[then(expr = "all listed entries belong to source {string}")]
fn then_all_entries_belong_to_source(world: &mut TankyuWorld, source_name: String) {
    assert!(
        !world.last_stdout.contains("No entries yet"),
        "Expected entries in output but got empty state\nstdout: {}",
        world.last_stdout
    );
    // Verify the entries for this source are present
    if let Some(titles) = world.source_entry_titles.get(&source_name) {
        for title in titles {
            assert!(
                world.last_stdout.contains(title),
                "stdout did not contain expected entry {title:?} from source {source_name:?}\nstdout: {}",
                world.last_stdout
            );
        }
    }
}

#[then(expr = "stdout does not contain entries from source {string}")]
fn then_stdout_no_entries_from_source(world: &mut TankyuWorld, source_name: String) {
    let titles = world.source_entry_titles.get(&source_name).unwrap_or_else(|| {
        panic!("No tracked entries for source {source_name:?} — Given step may not have created them")
    });
    for title in titles {
        assert!(
            !world.last_stdout.contains(title),
            "stdout should NOT contain entry {title:?} from excluded source {source_name:?}\nstdout: {}",
            world.last_stdout
        );
    }
}

#[then("all entries are listed")]
fn then_all_entries_listed(world: &mut TankyuWorld) {
    assert_all_tracked_titles_in_stdout(world);
}

// ── Helpers ───────────────────────────────────────────────────────────

fn ensure_default_source(world: &mut TankyuWorld) {
    let path = world
        .data_dir
        .path()
        .join(format!("sources/{DEFAULT_SOURCE_ID}.json"));
    if !path.exists() {
        world.write_source(
            DEFAULT_SOURCE_ID,
            "default-source",
            "https://example.com/default",
            "active",
            Some(1),
        );
    }
}

fn find_or_create_source(world: &mut TankyuWorld, name: &str) -> String {
    let sources_dir = world.data_dir.path().join("sources");
    for entry in std::fs::read_dir(&sources_dir).unwrap() {
        let path = entry.unwrap().path();
        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        if content["name"].as_str() == Some(name) {
            return content["id"].as_str().unwrap().to_string();
        }
    }
    let id = uuid::Uuid::new_v4().to_string();
    world.write_source(
        &id,
        name,
        &format!("https://example.com/{name}"),
        "active",
        Some(1),
    );
    id
}

fn write_entry_pair_for_source(world: &mut TankyuWorld, source_name: &str, source_id: &str) {
    let title1 = format!("{source_name} entry 1");
    let title2 = format!("{source_name} entry 2");
    world.write_named_entry(&title1, source_id, "new", None);
    world.write_named_entry(&title2, source_id, "read", None);
    world
        .source_entry_titles
        .insert(source_name.to_string(), vec![title1, title2]);
}

fn assert_all_tracked_titles_in_stdout(world: &TankyuWorld) {
    assert!(
        !world.created_entry_titles.is_empty(),
        "No entry titles were tracked — Given step may not have created entries"
    );
    for title in &world.created_entry_titles {
        assert!(
            world.last_stdout.contains(title),
            "stdout did not contain entry title {title:?}\nstdout: {}",
            world.last_stdout
        );
    }
}
