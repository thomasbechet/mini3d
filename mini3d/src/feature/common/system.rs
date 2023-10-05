use mini3d_derive::{Reflect, Resource, Serialize};

#[derive(Default, Debug, Resource, Serialize, Reflect, Clone)]
pub struct System {}

pub struct SystemStage {}
