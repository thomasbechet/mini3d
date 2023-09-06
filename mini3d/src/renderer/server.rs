use glam::{IVec2, Mat4, UVec2, Vec3};
use mini3d_derive::Error;

use crate::{
    define_server_handle,
    feature::component::renderer::{mesh::Mesh, texture::Texture},
    math::rect::IRect,
};

use super::{color::Color, event::RendererEvent, graphics::TextureWrapMode};

#[derive(Debug, Error)]
pub enum RendererServerError {
    #[error("Resource not found")]
    ResourceNotFound,
    #[error("Invalid material index")]
    InvalidMatrialIndex,
    #[error("Max resources reached")]
    MaxResourcesReached,
}

define_server_handle!(CommandBufferHandle);

define_server_handle!(MeshHandle);
define_server_handle!(TextureHandle);
define_server_handle!(MaterialHandle);

define_server_handle!(ViewportHandle);

define_server_handle!(SceneHandle);
define_server_handle!(SceneCameraHandle);
define_server_handle!(SceneModelHandle);
define_server_handle!(SceneCanvasHandle);

pub struct ServerMaterialDescriptor<'a> {
    pub diffuse: TextureHandle,
    pub name: &'a str,
}

#[allow(unused_variables)]
pub trait RendererServer {
    /// Global API

    fn events(&self) -> &[RendererEvent] {
        &[]
    }

    fn reset(&mut self) -> Result<(), RendererServerError> {
        Ok(())
    }

    /// Assets API

    fn mesh_add(&mut self, mesh: &Mesh) -> Result<MeshHandle, RendererServerError> {
        Ok(0.into())
    }
    fn mesh_remove(&mut self, handle: MeshHandle) -> Result<(), RendererServerError> {
        Ok(())
    }

    fn texture_add(&mut self, texture: &Texture) -> Result<TextureHandle, RendererServerError> {
        Ok(0.into())
    }
    fn texture_remove(&mut self, handle: TextureHandle) -> Result<(), RendererServerError> {
        Ok(())
    }

    fn material_add(
        &mut self,
        desc: ServerMaterialDescriptor,
    ) -> Result<MaterialHandle, RendererServerError> {
        Ok(0.into())
    }
    fn material_remove(&mut self, handle: MaterialHandle) -> Result<(), RendererServerError> {
        Ok(())
    }

    /// Canvas API

    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn scene_canvas_begin(
        &mut self,
        canvas: SceneCanvasHandle,
        clear_color: Color,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_end(&mut self) -> Result<(), RendererServerError> {
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
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_blit_viewport(
        &mut self,
        viewport: ViewportHandle,
        position: IVec2,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_fill_rect(&mut self, extent: IRect, color: Color) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_draw_rect(&mut self, extent: IRect, color: Color) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_draw_line(
        &mut self,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_draw_vline(
        &mut self,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_draw_hline(
        &mut self,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn canvas_scissor(&mut self, extent: Option<IRect>) -> Result<(), RendererServerError> {
        Ok(())
    }

    /// Viewport API

    fn viewport_add(&mut self, resolution: UVec2) -> Result<ViewportHandle, RendererServerError> {
        Ok(0.into())
    }
    fn viewport_remove(&mut self, handle: ViewportHandle) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn viewport_set_camera(
        &mut self,
        handle: ViewportHandle,
        camera: Option<SceneCameraHandle>,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn viewport_set_resolution(
        &mut self,
        handle: ViewportHandle,
        resolution: UVec2,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }

    /// Scene API

    fn scene_add(&mut self) -> Result<SceneHandle, RendererServerError> {
        Ok(0.into())
    }
    fn scene_remove(&mut self, handle: SceneHandle) -> Result<(), RendererServerError> {
        Ok(())
    }

    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle, RendererServerError> {
        Ok(0.into())
    }
    fn scene_camera_remove(
        &mut self,
        handle: SceneCameraHandle,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn scene_camera_update(
        &mut self,
        handle: SceneCameraHandle,
        eye: Vec3,
        forward: Vec3,
        up: Vec3,
        fov: f32,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }

    fn scene_model_add(
        &mut self,
        mesh: MeshHandle,
    ) -> Result<SceneModelHandle, RendererServerError> {
        Ok(0.into())
    }
    fn scene_model_remove(&mut self, handle: SceneModelHandle) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn scene_model_set_material(
        &mut self,
        handle: SceneModelHandle,
        index: usize,
        material: MaterialHandle,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn scene_model_transfer_matrix(
        &mut self,
        handle: SceneModelHandle,
        mat: Mat4,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }

    fn scene_canvas_add(
        &mut self,
        resolution: UVec2,
    ) -> Result<SceneCanvasHandle, RendererServerError> {
        Ok(0.into())
    }
    fn scene_canvas_remove(
        &mut self,
        handle: SceneCanvasHandle,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: SceneCanvasHandle,
        mat: Mat4,
    ) -> Result<(), RendererServerError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DummyRendererserver;

impl RendererServer for DummyRendererserver {}
