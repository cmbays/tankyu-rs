// cucumber step macros require `&mut World` even for read-only steps, and
// `String` captures by value — suppress the resulting pedantic lints globally.
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::TankyuWorld;
use cucumber::given;
use uuid::Uuid;

// ── Existing parameterized Given steps (used by other features) ───────

#[given(expr = "a source exists with name {string} checked {int} day ago with entries")]
fn given_source_with_entries(world: &mut TankyuWorld, name: String, days: i64) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &name,
        &format!("https://example.com/{name}"),
        "active",
        Some(days),
    );
    world.write_entry_for_source(&id);
}

#[given(expr = "a source exists with name {string} that has never been checked")]
fn given_never_checked_source(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &name,
        &format!("https://example.com/{name}"),
        "active",
        None,
    );
}

#[given(expr = "a source exists with name {string} last checked {int} days ago")]
fn given_stale_source(world: &mut TankyuWorld, name: String, days: i64) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &name,
        &format!("https://example.com/{name}"),
        "active",
        Some(days),
    );
}

#[given(expr = "a source exists with name {string} that has no entries")]
fn given_empty_source(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &name,
        &format!("https://example.com/{name}"),
        "active",
        Some(1),
    );
}

#[given(expr = "a pruned source exists with name {string}")]
fn given_pruned_source(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &name,
        &format!("https://example.com/{name}"),
        "pruned",
        None,
    );
}

// ── New non-parameterized Given steps (for health.feature) ────────────

#[given("a source exists that was checked recently with entries")]
fn given_recently_checked_source_with_entries(world: &mut TankyuWorld) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        "healthy-source",
        "https://example.com/healthy",
        "active",
        Some(1),
    );
    world.write_entry_for_source(&id);
}

#[given("a source exists that has never been checked")]
fn given_source_never_checked(world: &mut TankyuWorld) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        "never-checked-source",
        "https://example.com/never-checked",
        "active",
        None,
    );
    // Add an entry so it doesn't also trigger "empty" warning
    world.write_entry_for_source(&id);
}

#[given(expr = "a source exists last checked {int} days ago")]
fn given_source_last_checked_days_ago(world: &mut TankyuWorld, days: i64) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        &format!("stale-{days}d-source"),
        &format!("https://example.com/stale-{days}d"),
        "active",
        Some(days),
    );
    // Add an entry so it doesn't also trigger "empty" warning
    world.write_entry_for_source(&id);
}

#[given("a source exists that has no entries")]
fn given_source_no_entries(world: &mut TankyuWorld) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        "empty-source",
        "https://example.com/empty",
        "active",
        Some(1), // recently checked but no entries
    );
}

#[given("only a pruned source exists")]
fn given_only_pruned_source(world: &mut TankyuWorld) {
    let id = Uuid::new_v4().to_string();
    world.write_source(
        &id,
        "pruned-source",
        "https://example.com/pruned",
        "pruned",
        None,
    );
}

#[given(expr = "config has stale_days set to {int}")]
fn given_config_stale_days(world: &mut TankyuWorld, days: i64) {
    let config_path = world.data_dir.path().join("config.json");
    let mut config: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&config_path).unwrap()).unwrap();
    config["staleDays"] = serde_json::json!(days);
    std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
}
