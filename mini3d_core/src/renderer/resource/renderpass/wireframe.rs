use crate::renderer::{color::Color, resource::MeshHandle};

pub(crate) enum WireframePassCommand {
    DrawMesh { mesh: MeshHandle, color: Color },
}

pub(crate) struct WireframePass {}
