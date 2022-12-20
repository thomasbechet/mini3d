use std::collections::HashMap;

use anyhow::Result;
use glam::{IVec2, UVec2};
use serde::{Serialize, Deserialize};

use crate::{renderer::{backend::{CanvasViewportHandle, RendererBackend, CanvasHandle, SceneCameraHandle}, RendererManager}};

fn default_as_true() -> bool { true }

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub(crate) position: IVec2,
    pub(crate) resolution: UVec2,
    pub(crate) z_index: i32,
    pub(crate) camera: Option<hecs::Entity>,
    #[serde(skip, default = "default_as_true")]
    pub(crate) out_of_date: bool,
    #[serde(skip)]
    pub(crate) handle: Option<CanvasViewportHandle>,
}

impl Viewport {

    pub(crate) fn update_renderer(
        &mut self,
        canvas: CanvasHandle,
        cameras: &HashMap<hecs::Entity, SceneCameraHandle>,
        backend: &mut impl RendererBackend,
    ) -> Result<()> {
        if self.handle.is_none() {
            self.handle = Some(backend.canvas_viewport_add(canvas, self.position, self.resolution)?);
        }
        if self.out_of_date {
            let camera = self.camera.map(|entity| *cameras.get(&entity).unwrap());
            backend.canvas_viewport_set_camera(self.handle.unwrap(), camera)?;
            backend.canvas_viewport_set_position(self.handle.unwrap(), self.position)?;
            backend.canvas_viewport_set_z_index(self.handle.unwrap(), self.z_index)?;
            self.out_of_date = false;
        }
        Ok(())
    }

    pub(crate) fn release_renderer(
        &mut self,
        backend: &mut dyn RendererBackend,
    ) -> Result<()> {
        if let Some(handle) = self.handle {
            backend.canvas_viewport_remove(handle)?;
        }
        Ok(())
    }

    pub fn new(position: IVec2, resolution: UVec2) -> Self {
        Self { 
            position, 
            resolution, 
            z_index: 0, 
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

    pub fn set_z_index(&mut self, z_index: i32) -> &mut Self {
        self.z_index = z_index;
        self.out_of_date = true;
        self
    }

    pub fn set_camera(&mut self, camera: Option<hecs::Entity>) -> &mut Self {
        self.camera = camera;
        self.out_of_date = true;
        self
    }
}