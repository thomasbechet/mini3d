use serde::{Serialize, Deserialize};

use crate::{uid::UID, ecs::component::Component};

#[derive(Serialize, Deserialize)]
pub struct Lifecycle {
    pub alive: bool,
}

impl Component for Lifecycle {}

impl Lifecycle {
    
    pub fn alive() -> Self {
        Self { alive: true }
    }

    pub fn dead() -> Self {
        Self { alive: false }
    }
}

impl Lifecycle {
    pub const NAME: &'static str = "life_cycle";
    pub const UID: UID = UID::new(Lifecycle::NAME);
}