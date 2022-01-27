use cucumber::WorldInit;

mod common;

use common::state::World;

#[tokio::main]
async fn main() {
    World::run("./features").await;
}
