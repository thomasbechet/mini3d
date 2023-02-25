use anyhow::Result;
use glam::{Mat4, Vec3, IVec2, UVec2};

use crate::{uid::UID, feature::asset::{texture::Texture, mesh::Mesh}, math::rect::IRect};

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

define_handle!(ViewportHandle);

define_handle!(SceneHandle);
define_handle!(SceneCameraHandle);
define_handle!(SceneModelHandle);
define_handle!(SceneCanvasHandle);

pub struct BackendMaterialDescriptor<'a> {
    pub diffuse: TextureHandle,
    pub name: &'a str,
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

    /// Canvas API
    
    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<()> { Ok(()) }
    fn scene_canvas_begin(&mut self, canvas: SceneCanvasHandle, clear_color: Color) -> Result<()> { Ok(()) }
    fn canvas_end(&mut self) -> Result<()> { Ok(()) }
    fn canvas_blit_texture(&mut self, texture: TextureHandle, extent: IRect, position: IVec2, filtering: Color, alpha_threshold: u8) -> Result<()> { Ok(()) }
    fn canvas_blit_viewport(&mut self, viewport: ViewportHandle, position: IVec2) -> Result<()> { Ok(()) }
    fn canvas_fill_rect(&mut self, extent: IRect, color: Color) -> Result<()> { Ok(()) }
    fn canvas_draw_rect(&mut self, extent: IRect, color: Color) -> Result<()> { Ok(()) }
    fn canvas_draw_line(&mut self, x0: IVec2, x1: IVec2, color: Color) -> Result<()> { Ok(()) }
    fn canvas_draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) -> Result<()> { Ok(()) }
    fn canvas_draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) -> Result<()> { Ok(()) }
    fn canvas_scissor(&mut self, extent: Option<IRect>) -> Result<()> { Ok(()) }

    /// Viewport API
    
    fn viewport_add(&mut self, resolution: UVec2) -> Result<ViewportHandle> { Ok(0.into()) }
    fn viewport_remove(&mut self, handle: ViewportHandle) -> Result<()> { Ok(()) }
    fn viewport_set_camera(&mut self, handle: ViewportHandle, camera: Option<SceneCameraHandle>) -> Result<()> { Ok(()) }
    fn viewport_set_resolution(&mut self, handle: ViewportHandle, resolution: UVec2) -> Result<()> { Ok(()) }

    /// Scene API
    
    fn scene_add(&mut self) -> Result<SceneHandle> { Ok(0.into()) }
    fn scene_remove(&mut self, handle: SceneHandle) -> Result<()> { Ok(()) }

    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle> { Ok(0.into()) }
    fn scene_camera_remove(&mut self, handle: SceneCameraHandle) -> Result<()> { Ok(()) }
    fn scene_camera_update(&mut self, handle: SceneCameraHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> { Ok(()) }

    fn scene_model_add(&mut self, mesh: MeshHandle) -> Result<SceneModelHandle> { Ok(0.into()) }
    fn scene_model_remove(&mut self, handle: SceneModelHandle) -> Result<()> { Ok(()) }
    fn scene_model_set_material(&mut self, handle: SceneModelHandle, index: usize, material: MaterialHandle) -> Result<()> { Ok(()) }
    fn scene_model_transfer_matrix(&mut self, handle: SceneModelHandle, mat: Mat4) -> Result<()> { Ok(()) }

    fn scene_canvas_add(&mut self, resolution: UVec2) -> Result<SceneCanvasHandle> { Ok(0.into()) }
    fn scene_canvas_remove(&mut self, handle: SceneCanvasHandle) -> Result<()> { Ok(()) }
    fn scene_canvas_transfer_matrix(&mut self, handle: SceneCanvasHandle, mat: Mat4) -> Result<()> { Ok(()) }

    /// Statistics API

    fn statistics(&self) -> Result<RendererStatistics> { Ok(RendererStatistics { triangle_count: 0, draw_count: 0 }) }

}

#[derive(Default)]
pub struct DummyRendererBackend;

impl RendererBackend for DummyRendererBackend {}