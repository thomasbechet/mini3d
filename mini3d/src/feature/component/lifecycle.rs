use serde::{Serialize, Deserialize};

use crate::scene::component::Component;

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