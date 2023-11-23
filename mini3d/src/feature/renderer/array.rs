use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource};

#[derive(Default, Serialize)]
pub enum RenderFormat {
    I8x2,
    I8x4,
    I16x2,
    I16x4,
    I32x2,
    I32x4,
    F32x2,
    #[default]
    F32x4,
    M4x4,
}

#[derive(Default, Serialize, Reflect)]
pub enum RenderArrayUsage {
    #[default]
    Static,
    Dynamic,
}

#[derive(Default, Serialize, Reflect)]
pub struct RenderArray {
    format: RenderFormat,
    usage: RenderArrayUsage,
    size: u32,
}

impl Resource for RenderArray {}

impl RenderArray {
    pub const NAME: &'static str = "RTY_RenderArray";
}

define_resource_handle!(RenderArrayHandle);
