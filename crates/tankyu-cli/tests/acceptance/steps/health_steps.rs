// cucumber step macros require `&mut World` even for read-only steps, and
// `String` captures by value — suppress the resulting pedantic lints globally.
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::TankyuWorld;
use cucumber::given;
use uuid::Uuid;

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
