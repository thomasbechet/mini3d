use mini3d::{ecs::component::Component, uid::UID};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OS {
    pub layout_active: bool,
}

impl Component for OS {}

impl OS {
    pub const NAME: &'static str = "os";
    pub const UID: UID = UID::new(Self::NAME);
}