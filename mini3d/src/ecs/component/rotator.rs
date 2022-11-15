use serde::{Serialize, Deserialize};

use super::Component;

#[derive(Default, Serialize, Deserialize)]
pub struct RotatorComponent {
    pub speed: f32,
}

impl Component for RotatorComponent {}