use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::resource::{Resource, ResourceTypeHandle},
};

#[derive(Default, Reflect, Serialize, Clone)]
pub(crate) enum MaterialType {
    #[default]
    Opaque,
    Transparent,
}

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Material {
    pub(crate) ty: MaterialType,
    pub diffuse: ResourceTypeHandle,
}

impl Material {
    pub const NAME: &'static str = "RTY_Material";
}

impl Resource for Material {}

define_resource_handle!(MaterialHandle);
