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

#[given(expr = "a source exists with URL {string}")]
fn given_source_by_url(world: &mut TankyuWorld, url: String) {
    let id = Uuid::new_v4().to_string();
    let name = slug_from_url(&url);
    world.write_source(&id, &name, &url, "active", Some(1));
}

#[given(expr = "a source exists linked to topic {string} with URL {string}")]
fn given_source_linked_to_topic(world: &mut TankyuWorld, topic_name: String, url: String) {
    let source_id = Uuid::new_v4().to_string();
    let name = slug_from_url(&url);
    world.write_source(&source_id, &name, &url, "active", Some(1));
    // Write a monitors edge from topic → source.
    // Find the topic file to get its ID.
    let topics_dir = world.data_dir.path().join("topics");
    for entry in std::fs::read_dir(&topics_dir).unwrap() {
        let path = entry.unwrap().path();
        let content: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        if content["name"].as_str() == Some(&topic_name) {
            let topic_id = content["id"].as_str().unwrap().to_string();
            world.write_monitors_edge(&topic_id, &source_id);
            return;
        }
    }
    panic!("Topic '{topic_name}' not found — ensure `Given a topic exists` runs first");
}

#[given(expr = "a topic exists with name {string}")]
fn given_topic(world: &mut TankyuWorld, name: String) {
    let id = Uuid::new_v4().to_string();
    world.write_topic(&id, &name);
}

/// Delegate to the core `name_from_url` to avoid logic duplication.
fn slug_from_url(url: &str) -> String {
    tankyu_core::features::source::name_from_url(url)
}
