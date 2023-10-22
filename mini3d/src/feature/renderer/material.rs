use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{renderer::provider::RendererProviderHandle, resource::handle::ResourceHandle};

#[derive(Default, Clone, Resource, Serialize, Reflect)]
pub struct Material {
    pub diffuse: ResourceHandle,
    pub(crate) handle: RendererProviderHandle,
}
