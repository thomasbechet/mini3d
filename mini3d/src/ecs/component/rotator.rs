use serde::{Serialize, Deserialize};

#[derive(Default, Serialize, Deserialize)]
pub struct RotatorComponent {
    pub speed: f32,
}