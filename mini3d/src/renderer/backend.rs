use anyhow::Result;
use glam::{Mat4, Vec3};

use crate::{uid::UID, feature::asset::{texture::Texture, mesh::Mesh, font::Font}};

use super::{command_buffer::CommandBuffer, RendererStatistics};

macro_rules! define_handle {
    ($name:ident) => {
        #[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);
        impl From<u64> for $name {
            fn from(value: u64) -> Self { Self(value) }
        }
        impl From<$name> for u64 {
            fn from(handle: $name) -> Self { handle.0 }
        }
        impl From<UID> for $name {
            fn from(uid: UID) -> Self { Self(uid.into()) }
        }
        impl From<$name> for UID {
            fn from(handle: $name) -> Self { UID::from(handle.0) }
        }
    };
}

define_handle!(MeshHandle);
define_handle!(TextureHandle);
define_handle!(MaterialHandle);
define_handle!(FontHandle);

define_handle!(CameraHandle);
define_handle!(ModelHandle);

pub struct BackendMaterialDescriptor<'a> {
    pub diffuse: TextureHandle,
    pub name: &'a str,
}

#[allow(unused_variables)]
pub trait RendererBackend {

    /// Global API

    fn reset(&mut self) -> Result<()> { Ok(()) }

    /// Resources API
    
    fn mesh_add(&mut self, mesh: &Mesh) -> Result<MeshHandle> { Ok(0.into()) }
    fn mesh_remove(&mut self, handle: MeshHandle) -> Result<()> { Ok(()) }

    fn texture_add(&mut self, texture: &Texture) -> Result<TextureHandle> { Ok(0.into()) }
    fn texture_remove(&mut self, handle: TextureHandle) -> Result<()> { Ok(()) }

    fn material_add(&mut self, desc: BackendMaterialDescriptor) -> Result<MaterialHandle> { Ok(0.into()) }
    fn material_remove(&mut self, handle: MaterialHandle) -> Result<()> { Ok(()) }

    fn font_add(&mut self, font: &Font) -> Result<FontHandle> { Ok(0.into()) }
    fn font_remove(&mut self, handle: FontHandle) -> Result<()> { Ok(()) }

    /// Objects API

    fn camera_add(&mut self) -> Result<CameraHandle> { Ok(0.into()) }
    fn camera_remove(&mut self, handle: CameraHandle) -> Result<()> { Ok(()) }
    fn camera_update(&mut self, handle: CameraHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> { Ok(()) }

    fn model_add(&mut self, mesh: MeshHandle) -> Result<ModelHandle> { Ok(0.into()) }
    fn model_remove(&mut self, handle: ModelHandle) -> Result<()> { Ok(()) }
    fn model_set_material(&mut self, handle: ModelHandle, index: usize, material: MaterialHandle) -> Result<()> { Ok(()) }
    fn model_transfer_matrix(&mut self, handle: ModelHandle, mat: Mat4) -> Result<()> { Ok(()) }
    
    /// Commands API

    fn submit_command_buffer(&mut self, command: CommandBuffer) -> Result<()> { Ok(()) }

    /// Statistics API

    fn statistics(&self) -> Result<RendererStatistics> { Ok(RendererStatistics { triangle_count: 0, draw_count: 0, viewport: (0, 0) }) }

}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {}