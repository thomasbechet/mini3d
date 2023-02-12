use serde::{Serialize, Deserialize};

use crate::{scene::container::Component, uid::UID};

#[derive(Default, Serialize, Deserialize)]
pub struct Rotator {
    pub speed: f32,
}

impl Component for Rotator {}

impl Rotator {
    pub const NAME: &'static str = "rotator";
    pub const UID: UID = Rotator::NAME.into();
}