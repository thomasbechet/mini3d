use mini3d_derive::{Asset, Reflect, Serialize};

#[derive(Clone, Default, Asset, Reflect, Serialize)]
pub struct Script {
    pub source: String,
}
