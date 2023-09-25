use mini3d_derive::{Component, Reflect, Serialize};

#[derive(Clone, Default, Component, Reflect, Serialize)]
pub struct Script {
    pub source: String,
}
