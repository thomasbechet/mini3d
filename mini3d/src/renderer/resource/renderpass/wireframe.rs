use crate::{define_resource_handle, feature::renderer::mesh::MeshHandle, renderer::color::Color};

pub(crate) enum WireframePassCommand {
    DrawMesh { mesh: MeshHandle, color: Color },
}

pub(crate) struct WireframePass {}

define_resource_handle!(WireframePassHandle);
