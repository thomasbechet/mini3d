use serde::{Serialize, Deserialize};

use crate::scene::component::Component;

#[derive(Default, Serialize, Deserialize)]
pub struct Rotator {
    pub speed: f32,
}

impl Component for Rotator {}