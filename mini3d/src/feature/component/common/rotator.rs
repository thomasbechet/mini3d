use mini3d_derive::{Component, Reflect, Serialize};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Rotator {
    pub speed: f32,
}
