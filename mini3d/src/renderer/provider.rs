use glam::{IVec2, Mat4, UVec2, Vec3};
use mini3d_derive::Error;

use crate::{
    define_provider_handle,
    feature::component::renderer::{mesh::Mesh, texture::Texture},
    math::rect::IRect,
};

use super::{color::Color, event::RendererEvent, graphics::TextureWrapMode};

#[derive(Debug, Error)]
pub enum RendererProviderError {
    #[error("Resource not found")]
    ResourceNotFound,
    #[error("Invalid material index")]
    InvalidMatrialIndex,
    #[error("Max resources reached")]
    MaxResourcesReached,
}

define_provider_handle!(CommandBufferHandle);

define_provider_handle!(MeshHandle);
define_provider_handle!(TextureHandle);
define_provider_handle!(MaterialHandle);

define_provider_handle!(ViewportHandle);

define_provider_handle!(SceneHandle);
define_provider_handle!(SceneCameraHandle);
define_provider_handle!(SceneModelHandle);
define_provider_handle!(SceneCanvasHandle);

pub struct ProviderMaterialDescriptor<'a> {
    pub diffuse: TextureHandle,
    pub name: &'a str,
}

pub trait RendererProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);

    /// Global API

    fn next_event(&mut self) -> Option<RendererEvent>;
    fn reset(&mut self) -> Result<(), RendererProviderError>;

    /// Assets API

    fn mesh_add(&mut self, mesh: &Mesh) -> Result<MeshHandle, RendererProviderError>;
    fn mesh_remove(&mut self, handle: MeshHandle) -> Result<(), RendererProviderError>;

    fn texture_add(&mut self, texture: &Texture) -> Result<TextureHandle, RendererProviderError>;
    fn texture_remove(&mut self, handle: TextureHandle) -> Result<(), RendererProviderError>;

    fn material_add(
        &mut self,
        desc: ProviderMaterialDescriptor,
    ) -> Result<MaterialHandle, RendererProviderError>;
    fn material_remove(&mut self, handle: MaterialHandle) -> Result<(), RendererProviderError>;

    /// Canvas API

    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<(), RendererProviderError>;
    fn scene_canvas_begin(
        &mut self,
        canvas: SceneCanvasHandle,
        clear_color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_end(&mut self) -> Result<(), RendererProviderError>;
    fn canvas_blit_texture(
        &mut self,
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) -> Result<(), RendererProviderError>;
    fn canvas_blit_viewport(
        &mut self,
        viewport: ViewportHandle,
        position: IVec2,
    ) -> Result<(), RendererProviderError>;
    fn canvas_fill_rect(
        &mut self,
        extent: IRect,
        color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_draw_rect(
        &mut self,
        extent: IRect,
        color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_draw_line(
        &mut self,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_draw_vline(
        &mut self,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_draw_hline(
        &mut self,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_scissor(&mut self, extent: Option<IRect>) -> Result<(), RendererProviderError>;

    /// Viewport API

    fn viewport_add(&mut self, resolution: UVec2) -> Result<ViewportHandle, RendererProviderError>;
    fn viewport_remove(&mut self, handle: ViewportHandle) -> Result<(), RendererProviderError>;
    fn viewport_set_camera(
        &mut self,
        handle: ViewportHandle,
        camera: Option<SceneCameraHandle>,
    ) -> Result<(), RendererProviderError>;
    fn viewport_set_resolution(
        &mut self,
        handle: ViewportHandle,
        resolution: UVec2,
    ) -> Result<(), RendererProviderError>;

    /// Scene API

    fn scene_add(&mut self) -> Result<SceneHandle, RendererProviderError>;
    fn scene_remove(&mut self, handle: SceneHandle) -> Result<(), RendererProviderError>;

    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle, RendererProviderError>;
    fn scene_camera_remove(
        &mut self,
        handle: SceneCameraHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_camera_update(
        &mut self,
        handle: SceneCameraHandle,
        eye: Vec3,
        forward: Vec3,
        up: Vec3,
        fov: f32,
    ) -> Result<(), RendererProviderError>;

    fn scene_model_add(
        &mut self,
        mesh: MeshHandle,
    ) -> Result<SceneModelHandle, RendererProviderError>;
    fn scene_model_remove(&mut self, handle: SceneModelHandle)
        -> Result<(), RendererProviderError>;
    fn scene_model_set_material(
        &mut self,
        handle: SceneModelHandle,
        index: usize,
        material: MaterialHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_model_transfer_matrix(
        &mut self,
        handle: SceneModelHandle,
        mat: Mat4,
    ) -> Result<(), RendererProviderError>;

    fn scene_canvas_add(
        &mut self,
        resolution: UVec2,
    ) -> Result<SceneCanvasHandle, RendererProviderError>;
    fn scene_canvas_remove(
        &mut self,
        handle: SceneCanvasHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: SceneCanvasHandle,
        mat: Mat4,
    ) -> Result<(), RendererProviderError>;
}

#[derive(Default)]
pub struct PassiveRendererProvider;

#[allow(unused_variables)]
impl RendererProvider for PassiveRendererProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    /// Global API

    fn next_event(&mut self) -> Option<RendererEvent> {
        None
    }
    fn reset(&mut self) -> Result<(), RendererProviderError> {
        Ok(())
    }

    /// Assets API

    fn mesh_add(&mut self, mesh: &Mesh) -> Result<MeshHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn mesh_remove(&mut self, handle: MeshHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn texture_add(&mut self, texture: &Texture) -> Result<TextureHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn texture_remove(&mut self, handle: TextureHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn material_add(
        &mut self,
        desc: ProviderMaterialDescriptor,
    ) -> Result<MaterialHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn material_remove(&mut self, handle: MaterialHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }

    /// Canvas API

    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_canvas_begin(
        &mut self,
        canvas: SceneCanvasHandle,
        clear_color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_end(&mut self) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_blit_texture(
        &mut self,
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_blit_viewport(
        &mut self,
        viewport: ViewportHandle,
        position: IVec2,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_fill_rect(
        &mut self,
        extent: IRect,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_draw_rect(
        &mut self,
        extent: IRect,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_draw_line(
        &mut self,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_draw_vline(
        &mut self,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_draw_hline(
        &mut self,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_scissor(&mut self, extent: Option<IRect>) -> Result<(), RendererProviderError> {
        Ok(())
    }

    /// Viewport API

    fn viewport_add(&mut self, resolution: UVec2) -> Result<ViewportHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn viewport_remove(&mut self, handle: ViewportHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn viewport_set_camera(
        &mut self,
        handle: ViewportHandle,
        camera: Option<SceneCameraHandle>,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn viewport_set_resolution(
        &mut self,
        handle: ViewportHandle,
        resolution: UVec2,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    /// Scene API

    fn scene_add(&mut self) -> Result<SceneHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_remove(&mut self, handle: SceneHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_camera_remove(
        &mut self,
        handle: SceneCameraHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_camera_update(
        &mut self,
        handle: SceneCameraHandle,
        eye: Vec3,
        forward: Vec3,
        up: Vec3,
        fov: f32,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn scene_model_add(
        &mut self,
        mesh: MeshHandle,
    ) -> Result<SceneModelHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_model_remove(
        &mut self,
        handle: SceneModelHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_model_set_material(
        &mut self,
        handle: SceneModelHandle,
        index: usize,
        material: MaterialHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_model_transfer_matrix(
        &mut self,
        handle: SceneModelHandle,
        mat: Mat4,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn scene_canvas_add(
        &mut self,
        resolution: UVec2,
    ) -> Result<SceneCanvasHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_canvas_remove(
        &mut self,
        handle: SceneCanvasHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: SceneCanvasHandle,
        mat: Mat4,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn RendererProvider> {
    fn default() -> Self {
        Box::<PassiveRendererProvider>::default()
    }
}
