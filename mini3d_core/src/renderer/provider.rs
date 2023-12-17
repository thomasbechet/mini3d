use alloc::boxed::Box;
use mini3d_derive::Error;

use crate::{define_provider_handle, math::mat::M4I32F16};

use super::{
    event::RendererEvent,
    resource::{
        diffuse::{DiffusePassCommand, DiffusePassInfo, DiffusePassRenderInfo},
        mesh::Mesh,
        renderpass::canvas::{CanvasPassCommand, CanvasPassInfo, CanvasPassRenderInfo},
        texture::Texture,
    },
};

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

pub struct ProviderMaterialInfo {
    pub diffuse: RendererProviderHandle,
}

pub trait RendererProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);

    /// Global API

    fn next_event(&mut self) -> Option<RendererEvent>;
    fn reset(&mut self) -> Result<(), RendererProviderError>;

    /// Resource API

    fn add_mesh(&mut self, mesh: &Mesh) -> Result<RendererProviderHandle, RendererProviderError>;
    fn remove_mesh(&mut self, handle: RendererProviderHandle) -> Result<(), RendererProviderError>;

    fn add_texture(
        &mut self,
        texture: &Texture,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn remove_texture(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;

    fn add_material(
        &mut self,
        desc: ProviderMaterialInfo,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn remove_material(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;

    fn add_transform(&mut self) -> Result<RendererProviderHandle, RendererProviderError>;
    fn remove_transform(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn update_transform(
        &mut self,
        handle: RendererProviderHandle,
        mat: M4I32F16,
    ) -> Result<(), RendererProviderError>;

    fn add_diffuse_pass(
        &mut self,
        info: &DiffusePassInfo,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn remove_diffuse_pass(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn submit_diffuse_pass(
        &mut self,
        pass: RendererProviderHandle,
        command: &DiffusePassCommand,
    ) -> Result<(), RendererProviderError>;
    fn render_diffuse_pass(
        &mut self,
        pass: RendererProviderHandle,
        info: &DiffusePassRenderInfo,
    ) -> Result<(), RendererProviderError>;

    fn add_canvas_pass(
        &mut self,
        info: &CanvasPassInfo,
    ) -> Result<RendererProviderHandle, RendererProviderError>;
    fn remove_canvas_pass(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError>;
    fn submit_canvas_pass(
        &mut self,
        pass: RendererProviderHandle,
        command: &CanvasPassCommand,
    ) -> Result<(), RendererProviderError>;
    fn render_canvas_pass(
        &mut self,
        pass: RendererProviderHandle,
        info: &CanvasPassRenderInfo,
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

    /// Resource API

    fn add_mesh(&mut self, mesh: &Mesh) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn remove_mesh(&mut self, handle: RendererProviderHandle) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn add_texture(
        &mut self,
        texture: &Texture,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn remove_texture(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn add_material(
        &mut self,
        desc: ProviderMaterialInfo,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(0.into())
    }
    fn remove_material(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn add_transform(&mut self) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(Default::default())
    }
    fn remove_transform(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn update_transform(
        &mut self,
        handle: RendererProviderHandle,
        mat: M4I32F16,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn add_diffuse_pass(
        &mut self,
        info: &DiffusePassInfo,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(Default::default())
    }
    fn remove_diffuse_pass(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn submit_diffuse_pass(
        &mut self,
        pass: RendererProviderHandle,
        command: &DiffusePassCommand,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn render_diffuse_pass(
        &mut self,
        pass: RendererProviderHandle,
        info: &DiffusePassRenderInfo,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }

    fn add_canvas_pass(
        &mut self,
        info: &CanvasPassInfo,
    ) -> Result<RendererProviderHandle, RendererProviderError> {
        Ok(Default::default())
    }
    fn remove_canvas_pass(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn submit_canvas_pass(
        &mut self,
        pass: RendererProviderHandle,
        command: &CanvasPassCommand,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
    fn render_canvas_pass(
        &mut self,
        pass: RendererProviderHandle,
        info: &CanvasPassRenderInfo,
    ) -> Result<(), RendererProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn RendererProvider> {
    fn default() -> Self {
        Box::<PassiveRendererProvider>::default()
    }
}
