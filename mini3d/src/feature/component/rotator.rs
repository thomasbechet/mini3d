use serde::{Serialize, Deserialize};

use crate::{uid::UID, registry::component::Component};

#[derive(Default, Serialize, Deserialize)]
pub struct Rotator {
    pub speed: f32,
}

impl Component for Rotator {}

impl Rotator {
    pub const NAME: &'static str = "rotator";
    pub const UID: UID = Rotator::NAME.into();
}