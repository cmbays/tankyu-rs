mod steps;
mod world;

use cucumber::World as _;
use world::TankyuWorld;

#[tokio::main]
async fn main() {
    TankyuWorld::cucumber()
        .filter_run("tests/acceptance/features", |_, _, sc| {
            !sc.tags.iter().any(|t| t == "wip")
        })
        .await;
}
