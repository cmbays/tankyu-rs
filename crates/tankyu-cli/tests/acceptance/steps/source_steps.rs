// cucumber step macros require `&mut World` even for read-only steps, and
// `String` captures by value — suppress the resulting pedantic lints globally.
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::TankyuWorld;
use cucumber::given;
use uuid::Uuid;

#[given(expr = "a source exists with name {string} and URL {string}")]
fn given_source(world: &mut TankyuWorld, name: String, url: String) {
    let id = Uuid::new_v4().to_string();
    world.write_source(&id, &name, &url, "active", Some(1));
}

#[given(expr = "a topic exists with name {string}")]
fn given_topic(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_topic(&id, &name);
}
