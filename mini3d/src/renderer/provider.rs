use alloc::boxed::Box;
use mini3d_derive::Error;

use crate::{
    define_provider_handle,
    feature::renderer::{
        mesh::Mesh,
        texture::{Texture, TextureWrapMode},
    },
    math::{
        mat::M4I32F16,
        rect::IRect,
        vec::{V2I32, V2U32, V3I32F16},
    },
};

use super::{color::Color, event::RendererEvent};

#[derive(Debug, Error)]
pub enum RendererProviderError {
    #[error("Resource not found")]
    ResourceNotFound,
    #[error("Invalid material index")]
    InvalidMatrialIndex,
    #[error("Max resources reached")]
    MaxResourcesReached,
}

define_provider_handle!(RendererProviderHandle);

pub struct ProviderMaterialDescriptor<'a> {
    pub diffuse: RendererProviderHandle,
    pub name: &'a str,
}

pub trait RendererProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);

    /// Global API

    fn next_event(&mut self) -> Option<RendererEvent>;
    fn reset(&mut self) -> Result<(), RendererProviderError>;

    /// Assets API

    fn mesh_add(&mut self, mesh: &Mesh) -> Result<RendererProviderHandle, RendererProviderError>;
    fn mesh_remove(&mut self, handle: RendererProviderHandle) -> Result<(), RendererProviderError>;

    fn texture_add(
        &mut self,
        texture: &Texture,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn texture_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;

    fn material_add(
        &mut self,
        desc: ProviderMaterialDescriptor,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn material_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;

    /// Canvas API

    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<(), RendererProviderError>;
    fn scene_canvas_begin(
        &mut self,
        canvas: RendererProviderHandle,
        clear_color: Color,
    ) -> Result<(), RendererProviderError>;
    fn canvas_end(&mut self) -> Result<(), RendererProviderError>;
    fn canvas_blit_texture(
        &mut self,
        texture: RendererProviderHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) -> Result<(), RendererProviderError>;
    fn canvas_blit_viewport(
        &mut self,
        viewport: RendererProviderHandle,
        position: V2I32,
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
        x0: V2I32,
        x1: V2I32,
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

    fn viewport_add(
        &mut self,
        resolution: V2U32,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn viewport_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn viewport_set_camera(
        &mut self,
        handle: RendererProviderHandle,
        camera: Option<RendererProviderHandle>,
    ) -> Result<(), RendererProviderError>;
    fn viewport_set_resolution(
        &mut self,
        handle: RendererProviderHandle,
        resolution: V2U32,
    ) -> Result<(), RendererProviderError>;

    /// Scene API

    fn scene_add(&mut self) -> Result<RendererProviderHandle, RendererProviderError>;
    fn scene_remove(&mut self, handle: RendererProviderHandle)
        -> Result<(), RendererProviderError>;

    fn scene_camera_add(&mut self) -> Result<RendererProviderHandle, RendererProviderError>;
    fn scene_camera_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_camera_update(
        &mut self,
        handle: RendererProviderHandle,
        eye: V3I32F16,
        forward: V3I32F16,
        up: V3I32F16,
        fov: f32,
    ) -> Result<(), RendererProviderError>;

    fn scene_model_add(
        &mut self,
        mesh: RendererProviderHandle,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn scene_model_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_model_set_material(
        &mut self,
        handle: RendererProviderHandle,
        index: usize,
        material: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_model_transfer_matrix(
        &mut self,
        handle: RendererProviderHandle,
        mat: M4I32F16,
    ) -> Result<(), RendererProviderError>;

    fn scene_canvas_add(
        &mut self,
        resolution: V2U32,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn scene_canvas_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: RendererProviderHandle,
        mat: M4I32F16,
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

    fn mesh_add(&mut self, mesh: &Mesh) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn mesh_remove(&mut self, handle: RendererProviderHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn texture_add(
        &mut self,
        texture: &Texture,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn texture_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn material_add(
        &mut self,
        desc: ProviderMaterialDescriptor,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn material_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    /// Canvas API

    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_canvas_begin(
        &mut self,
        canvas: RendererProviderHandle,
        clear_color: Color,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_end(&mut self) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn canvas_blit_texture(
        &mut self,
        texture: RendererProviderHandle,
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
        viewport: RendererProviderHandle,
        position: V2I32,
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
        x0: V2I32,
        x1: V2I32,
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

    fn viewport_add(
        &mut self,
        resolution: V2U32,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn viewport_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn viewport_set_camera(
        &mut self,
        handle: RendererProviderHandle,
        camera: Option<RendererProviderHandle>,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn viewport_set_resolution(
        &mut self,
        handle: RendererProviderHandle,
        resolution: V2U32,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    /// Scene API

    fn scene_add(&mut self) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn scene_camera_add(&mut self) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_camera_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_camera_update(
        &mut self,
        handle: RendererProviderHandle,
        eye: V3I32F16,
        forward: V3I32F16,
        up: V3I32F16,
        fov: f32,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn scene_model_add(
        &mut self,
        mesh: RendererProviderHandle,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_model_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_model_set_material(
        &mut self,
        handle: RendererProviderHandle,
        index: usize,
        material: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_model_transfer_matrix(
        &mut self,
        handle: RendererProviderHandle,
        mat: M4I32F16,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn scene_canvas_add(
        &mut self,
        resolution: V2U32,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn scene_canvas_remove(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: RendererProviderHandle,
        mat: M4I32F16,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn RendererProvider> {
    fn default() -> Self {
        Box::<PassiveRendererProvider>::default()
    }
}
