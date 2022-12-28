use std::collections::HashMap;

use anyhow::Result;
use glam::{IVec2, UVec2};
use serde::{Serialize, Deserialize};

use crate::{renderer::backend::{RendererBackend, ViewportHandle, SceneCameraHandle}};

fn default_as_true() -> bool { true }

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub(crate) position: IVec2,
    pub(crate) resolution: UVec2,
    pub(crate) camera: Option<hecs::Entity>,
    #[serde(skip, default = "default_as_true")]
    pub(crate) out_of_date: bool,
    #[serde(skip)]
    pub(crate) handle: Option<ViewportHandle>,
}

impl Viewport {

    pub(crate) fn draw(
        &mut self, 
        cameras: &HashMap<hecs::Entity, SceneCameraHandle>,
        backend: &mut impl RendererBackend,
    ) -> Result<()> {
        if self.handle.is_none() {
            self.handle = Some(backend.viewport_add(self.resolution)?);
        }
        if self.out_of_date {
            let camera = self.camera.map(|entity| *cameras.get(&entity).unwrap());
            backend.viewport_set_camera(self.handle.unwrap(), camera)?;
            backend.viewport_set_resolution(self.handle.unwrap(), self.resolution)?;
            self.out_of_date = false;
        }
        backend.canvas_blit_viewport(self.handle.unwrap(), self.position)?;
        Ok(())
    }

    pub(crate) fn release_backend(&mut self, backend: &mut dyn RendererBackend) -> Result<()> {
        if let Some(handle) = self.handle {
            backend.viewport_remove(handle)?;
        }
        Ok(())
    }

    pub fn new(position: IVec2, resolution: UVec2) -> Self {
        Self { 
            position, 
            resolution, 
            camera: None, 
            out_of_date: true,
            handle: None,
        }
    }

    pub fn set_position(&mut self, position: IVec2) -> &mut Self {
        self.position = position;
        self.out_of_date = true;
        self
    }

    pub fn set_camera(&mut self, camera: Option<hecs::Entity>) -> &mut Self {
        self.camera = camera;
        self.out_of_date = true;
        self
    }
}