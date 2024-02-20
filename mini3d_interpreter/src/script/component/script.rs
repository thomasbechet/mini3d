use alloc::string::String;
use mini3d_derive::{Reflect, Serialize};

#[derive(Clone, Default, Reflect, Serialize)]
pub struct Script {
    pub source: String,
}
