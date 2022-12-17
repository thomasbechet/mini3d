use anyhow::Result;
use glam::{Mat4, Vec3, IVec2, UVec2};

use crate::{uid::UID, feature::asset::{texture::Texture, mesh::Mesh, font::Font}, math::rect::IRect};

use super::{RendererStatistics, color::Color};

macro_rules! define_handle {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

define_handle!(CanvasHandle);
define_handle!(CanvasViewportHandle);
define_handle!(CanvasSpriteHandle);
define_handle!(CanvasPrimitiveHandle);
define_handle!(CanvasScissorHandle);

define_handle!(SceneCameraHandle);
define_handle!(SceneModelHandle);
define_handle!(SceneCanvasHandle);

define_handle!(SurfaceCanvasHandle);

pub struct BackendMaterialDescriptor<'a> {
    pub diffuse: TextureHandle,
    pub name: &'a str,
}

pub enum BackendCanvasPrimitive {
    Rectangle { extent: IRect, color: Color },
    Box { extent: IRect, color: Color },
    Line { x0: IVec2, x1: IVec2, color: Color },
    VerticalLine { x: i32, y0: i32, y1: i32, color: Color },
    HorizontalLine { y: i32, x0: i32, x1: i32, color: Color },
}

#[allow(unused_variables)]
pub trait RendererBackend {

    /// Global API

    fn reset(&mut self) -> Result<()> { Ok(()) }

    /// Assets API
    
    fn mesh_add(&mut self, mesh: &Mesh) -> Result<MeshHandle> { Ok(0.into()) }
    fn mesh_remove(&mut self, handle: MeshHandle) -> Result<()> { Ok(()) }

    fn texture_add(&mut self, texture: &Texture) -> Result<TextureHandle> { Ok(0.into()) }
    fn texture_remove(&mut self, handle: TextureHandle) -> Result<()> { Ok(()) }

    fn material_add(&mut self, desc: BackendMaterialDescriptor) -> Result<MaterialHandle> { Ok(0.into()) }
    fn material_remove(&mut self, handle: MaterialHandle) -> Result<()> { Ok(()) }

    fn font_add(&mut self, font: &Font) -> Result<FontHandle> { Ok(0.into()) }
    fn font_remove(&mut self, handle: FontHandle) -> Result<()> { Ok(()) }

    /// Canvas API
    
    fn canvas_add(&mut self, width: u32, height: u32) -> Result<CanvasHandle> { Ok(0.into()) }
    fn canvas_remove(&mut self, handle: CanvasHandle) -> Result<()> { Ok(()) }
    fn canvas_set_clear_color(&mut self, handle: CanvasHandle, color: Color) -> Result<()> { Ok(()) }

    fn canvas_sprite_add(&mut self, canvas: CanvasHandle, texture: TextureHandle, position: IVec2, extent: IRect) -> Result<CanvasSpriteHandle> { Ok(0.into()) }
    fn canvas_sprite_remove(&mut self, handle: CanvasSpriteHandle) -> Result<()> { Ok(()) }
    fn canvas_sprite_set_position(&mut self, handle: CanvasSpriteHandle, position: IVec2) -> Result<()> { Ok(()) }
    fn canvas_sprite_set_extent(&mut self, handle: CanvasSpriteHandle, extent: IRect) -> Result<()> { Ok(()) }
    fn canvas_sprite_set_z_index(&mut self, handle: CanvasSpriteHandle, z_index: i32) -> Result<()> { Ok(()) }
    fn canvas_sprite_set_texture(&mut self, handle: CanvasSpriteHandle, texture: TextureHandle) -> Result<()> { Ok(()) }
    fn canvas_sprite_set_color(&mut self, handle: CanvasSpriteHandle, color: Color) -> Result<()> { Ok(()) }

    fn canvas_primitive_add(&mut self, canvas: CanvasHandle) -> Result<CanvasPrimitiveHandle> { Ok(0.into()) }
    fn canvas_primitive_remove(&mut self, handle: CanvasPrimitiveHandle) -> Result<()> { Ok(()) }
    fn canvas_primitive_set_z_index(&mut self, handle: CanvasPrimitiveHandle, z_index: i32) -> Result<()> { Ok(()) }
    fn canvas_primitive_update(&mut self, handle: CanvasPrimitiveHandle, primitive: BackendCanvasPrimitive) -> Result<()> { Ok(()) }

    fn canvas_scissor_add(&mut self, canvas: CanvasHandle) -> Result<CanvasScissorHandle> { Ok(0.into()) }
    fn canvas_scissor_remove(&mut self, handle: CanvasScissorHandle) -> Result<()> { Ok(()) }
    fn canvas_scissor_set_z_index(&mut self, handle: CanvasScissorHandle, z_index: i32) -> Result<()> { Ok(()) }
    fn canvas_scissor_set_view(&mut self, handle: CanvasScissorHandle, canvas: CanvasHandle, extent: IRect) -> Result<()> { Ok(()) }
    fn canvas_scissor_extent(&mut self, handle: CanvasScissorHandle, extent: IRect) -> Result<()> { Ok(()) }

    fn canvas_viewport_add(&mut self, canvas: CanvasHandle, position: IVec2, resolution: UVec2) -> Result<CanvasViewportHandle> { Ok(0.into()) }
    fn canvas_viewport_remove(&mut self, handle: CanvasViewportHandle) -> Result<()> { Ok(()) }
    fn canvas_viewport_set_z_index(&mut self, handle: CanvasViewportHandle, z_index: i32) -> Result<()> { Ok(()) }
    fn canvas_viewport_set_position(&mut self, handle: CanvasViewportHandle, position: IVec2) -> Result<()> { Ok(()) }
    fn canvas_viewport_set_camera(&mut self, handle: CanvasViewportHandle, camera: Option<SceneCameraHandle>) -> Result<()> { Ok(()) }

    /// Scene API

    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle> { Ok(0.into()) }
    fn scene_camera_remove(&mut self, handle: SceneCameraHandle) -> Result<()> { Ok(()) }
    fn scene_camera_update(&mut self, handle: SceneCameraHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> { Ok(()) }

    fn scene_model_add(&mut self, mesh: MeshHandle) -> Result<SceneModelHandle> { Ok(0.into()) }
    fn scene_model_remove(&mut self, handle: SceneModelHandle) -> Result<()> { Ok(()) }
    fn scene_model_set_material(&mut self, handle: SceneModelHandle, index: usize, material: MaterialHandle) -> Result<()> { Ok(()) }
    fn scene_model_transfer_matrix(&mut self, handle: SceneModelHandle, mat: Mat4) -> Result<()> { Ok(()) }

    fn scene_canvas_add(&mut self, canvas: CanvasHandle) -> Result<SceneCanvasHandle> { Ok(0.into()) }
    fn scene_canvas_remove(&mut self, handle: SceneCanvasHandle) -> Result<()> { Ok(()) }
    fn scene_canvas_transfer_matrix(&mut self, handle: SceneCanvasHandle, mat: Mat4) -> Result<()> { Ok(()) }

    /// Surface API

    fn surface_canvas_add(&mut self, canvas: CanvasHandle, position: IVec2, z_index: i32) -> Result<SurfaceCanvasHandle> { Ok(0.into()) }
    fn surface_canvas_remove(&mut self, handle: SurfaceCanvasHandle) -> Result<()> { Ok(()) }

    /// Statistics API

    fn statistics(&self) -> Result<RendererStatistics> { Ok(RendererStatistics { triangle_count: 0, draw_count: 0 }) }

}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {}