use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource};

use super::array::RenderFormat;

#[derive(Default, Serialize, Reflect)]
pub struct RenderConstant {
    format: RenderFormat,
}

impl Resource for RenderConstant {}

impl RenderConstant {
    pub const NAME: &'static str = "RTY_RenderConstant";
}

define_resource_handle!(RenderConstantHandle);
