// cucumber step macros require `&mut World` even for read-only steps, and
// `String` captures by value — suppress the resulting pedantic lints globally.
#![allow(clippy::needless_pass_by_ref_mut)]
#![allow(clippy::needless_pass_by_value)]

use crate::world::TankyuWorld;
use cucumber::{given, then};

#[given("no research graph database exists")]
fn given_no_db(world: &mut TankyuWorld) {
    let db_path = world.data_dir.path().join("db");
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path).unwrap();
    }
}

#[given("the configuration file is missing")]
fn given_no_config(world: &mut TankyuWorld) {
    let config_path = world.data_dir.path().join("config.json");
    if config_path.exists() {
        std::fs::remove_file(&config_path).unwrap();
    }
}

#[then("the research graph database exists")]
fn then_db_exists(world: &mut TankyuWorld) {
    let db_path = world.data_dir.path().join("db");
    assert!(
        db_path.join("schema.ir.json").exists(),
        "expected research graph DB at {}, but it does not exist",
        db_path.display()
    );
}
