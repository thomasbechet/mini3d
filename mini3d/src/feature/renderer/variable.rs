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
pub struct RenderVariable {
    format: RenderFormat,
    interpolate: bool,
}

impl Resource for RenderVariable {}

impl RenderVariable {
    pub const NAME: &'static str = "RTY_RenderVariable";
}

define_resource_handle!(RenderVariableHandle);
