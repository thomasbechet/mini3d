use mini3d_derive::{Reflect, Resource, Serialize};

use crate::define_resource_handle;

#[derive(Clone, Default, Resource, Reflect, Serialize)]
pub struct Script {
    pub source: String,
}

define_resource_handle!(ScriptHandle);
