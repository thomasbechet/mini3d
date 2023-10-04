use mini3d_derive::{Reflect, Resource, Serialize};

#[derive(Clone, Default, Resource, Reflect, Serialize)]
pub struct Script {
    pub source: String,
}
