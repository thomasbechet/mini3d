use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource};

use super::variable::RenderFormat;

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
    interpolate: bool,
    size: u32,
}

impl Resource for RenderArray {}

impl RenderArray {
    pub const NAME: &'static str = "RTY_RenderArray";
}

define_resource_handle!(RenderArrayHandle);
