use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{uid::UID, registry::component::{Component, EntityResolver, ComponentDefinition}};

#[derive(Default, Serialize, Deserialize)]
pub struct Rotator {
    pub speed: f32,
}

impl Component for Rotator {}

impl Rotator {
    pub const NAME: &'static str = "rotator";
    pub const UID: UID = UID::new(Rotator::NAME);
}