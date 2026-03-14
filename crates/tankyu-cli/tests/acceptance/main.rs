mod steps;
mod world;

use cucumber::World as _;
use world::TankyuWorld;

#[tokio::main]
async fn main() {
    TankyuWorld::run("tests/acceptance/features").await;
}
