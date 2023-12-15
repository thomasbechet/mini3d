use crate::{
    define_resource_handle,
    renderer::{color::Color, resource::MeshHandle},
};

pub(crate) enum WireframePassCommand {
    DrawMesh { mesh: MeshHandle, color: Color },
}

pub(crate) struct WireframePass {}

define_resource_handle!(WireframePassHandle);
