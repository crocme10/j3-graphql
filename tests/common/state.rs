use async_trait::async_trait;
use cucumber::WorldInit;

use super::error::Error;

#[derive(Debug, WorldInit)]
pub struct World {
    // user: Option<String>,
// capacity: usize,
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Error;

    async fn new() -> Result<Self, Self::Error> {
        Ok(Self {}) // user: None, capacity: 0 })
    }
}
