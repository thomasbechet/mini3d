use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow};
use glam::{UVec2, uvec2};
use serde::{Serialize, Deserialize};

use crate::{math::rect::IRect, asset::AssetManager, uid::{UID, SequentialGenerator}, scene::SceneManager, feature::{component::{transform::TransformComponent, camera::CameraComponent, model::ModelComponent}, asset::{model::Model, material::Material, mesh::Mesh, texture::Texture, font::Font}}};

use self::{backend::{RendererBackend, BackendHandle, BackendMaterialDescriptor}, command_buffer::CommandBuffer};

pub mod backend;
pub mod color;
pub mod command_buffer;
pub mod rasterizer;

// 3:2 aspect ratio
// pub const SCREEN_WIDTH: u32 = 480;
// pub const SCREEN_HEIGHT: u32 = 320;
// // 4:3 aspect ratio
// pub const SCREEN_WIDTH: u32 = 512;
// pub const SCREEN_HEIGHT: u32 = 384;
// // 16:10 aspect ratio
// pub const SCREEN_WIDTH: u32 = 320;
// pub const SCREEN_HEIGHT: u32 = 200;
pub const SCREEN_WIDTH: u32 = 512;
pub const SCREEN_HEIGHT: u32 = 320;
// pub const SCREEN_WIDTH: u32 = 640;
// pub const SCREEN_HEIGHT: u32 = 400;
// // 16:9 aspect ratio
// pub const SCREEN_WIDTH: u32 = 384;
// pub const SCREEN_HEIGHT: u32 = 216;

pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;
pub const SCREEN_RESOLUTION: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_CENTER: UVec2 = uvec2(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
pub const SCREEN_VIEWPORT: IRect = IRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
pub const SCREEN_INV_ASPECT_RATIO: f32 = 1.0 / SCREEN_ASPECT_RATIO;

pub const TILE_SIZE: u32 = 8;
pub const TILE_HCOUNT: u32 = SCREEN_WIDTH / TILE_SIZE;
pub const TILE_VCOUNT: u32 = SCREEN_HEIGHT / TILE_SIZE;

struct RendererFont {
    handle: Option<BackendHandle>,
}

struct RendererTexture {
    handle: Option<BackendHandle>,
}

struct RendererMesh {
    handle: Option<BackendHandle>,
}

struct RendererMaterial {
    handle: Option<BackendHandle>,
}

struct RendererCamera {
    handle: Option<BackendHandle>,
}

struct RendererModel {
    mesh: UID,
    materials: Vec<UID>,
    handle: Option<BackendHandle>,
}

pub enum RendererModelDescriptor {
    FromAsset(UID),
}

pub type RendererHandle = UID;

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct RendererStatistics {
    pub triangle_count: usize,
    pub draw_count: usize,
    pub viewport: (u32, u32),
}

#[derive(Default)]
pub struct RendererManager {

    generator: SequentialGenerator,

    fonts: HashMap<UID, RendererFont>,
    textures: HashMap<UID, RendererTexture>,
    meshes: HashMap<UID, RendererMesh>,
    materials: HashMap<UID, RendererMaterial>,
    requested_fonts: HashSet<UID>,
    requested_textures: HashSet<UID>,
    requested_meshes: HashSet<UID>,
    requested_materials: HashSet<UID>,

    cameras: HashMap<RendererHandle, RendererCamera>,
    cameras_removed: HashSet<BackendHandle>,
    models: HashMap<RendererHandle, RendererModel>,
    models_removed: HashSet<BackendHandle>,

    commands: Vec<CommandBuffer>,

    statistics: RendererStatistics,
}

impl RendererManager {

    pub(crate) fn prepare(&mut self) -> Result<()> {
        self.commands.clear();
        Ok(())
    }
 
    pub(crate) fn update_backend(
        &mut self, 
        backend: &mut impl RendererBackend, 
        asset: &AssetManager,
        scene: &mut SceneManager,
    ) -> Result<()> {

        // Send requested resources
        for font_uid in self.requested_fonts.drain() {
            let font = asset.get::<Font>(font_uid)?;
            let handle = backend.font_add(font)?;
            self.fonts.insert(font_uid, RendererFont { handle: Some(handle) });
        }
        for mesh_uid in self.requested_meshes.drain() {
            let mesh = asset.get::<Mesh>(mesh_uid)?;
            let handle = backend.mesh_add(mesh)?;
            self.meshes.insert(mesh_uid, RendererMesh { handle: Some(handle) });
        }
        for texture_uid in self.requested_textures.drain() {
            let texture = asset.get::<Texture>(texture_uid)?;
            let handle = backend.texture_add(texture)?;
            self.textures.insert(texture_uid, RendererTexture { handle: Some(handle) });
        }
        for material_uid in self.requested_materials.drain() {
            let material = asset.get::<Material>(material_uid)?;
            let diffuse = self.textures.get(&material.diffuse).unwrap();
            let handle = backend.material_add(BackendMaterialDescriptor {
                diffuse: diffuse.handle.unwrap()
            })?;
            self.materials.insert(material_uid, RendererMaterial { handle: Some(handle) });
        }

        // Remove cameras
        for handle in self.cameras_removed.drain() {
            backend.camera_remove(handle)?;
        }

        // Remove objects
        for handle in self.models_removed.drain() {
            backend.model_remove(handle)?;
        }

        // Update scene components
        for world in scene.iter_world() {

            // Update cameras
            for (_, (c, t)) in world.query_mut::<(&CameraComponent, &TransformComponent)>() {
                let camera = self.cameras.get_mut(&c.handle.unwrap()).unwrap();
                if camera.handle.is_none() {
                    camera.handle = Some(backend.camera_add()?);
                }
                backend.camera_update(camera.handle.unwrap(), t.translation, t.forward(), t.up(), c.fov)?;
            }
            
            // Update models
            for (_, (m, t)) in world.query_mut::<(&ModelComponent, &TransformComponent)>() {
                let model = self.models.get_mut(&m.handle.unwrap()).unwrap();
                if model.handle.is_none() {
                    let mesh = self.meshes.get(&model.mesh).unwrap();
                    model.handle = Some(backend.model_add(mesh.handle.unwrap())?);
                    for material_uid in &model.materials {
                        let material = self.materials.get(material_uid).unwrap();
                        backend.model_set_material(model.handle.unwrap(), material.handle.unwrap())?;
                    }
                }
                backend.model_transfer_matrix(model.handle.unwrap(), t.matrix())?;
            }
        }

        // TODO:
        // // Send commands
        // for buffer in self.commands {
        //     for command in buffer.iter() {
        //         match command {
        //             Command::Print { p, text, font } => {
                        
        //             },
        //             Command::DrawLine { p0, p1 } => {
                        
        //             },
        //             Command::DrawVLine { x, y0, y1 } => {
                        
        //             },
        //             Command::DrawHLine { y, x0, x1 } => {
                        
        //             },
        //             Command::DrawRect { rect } => {
                        
        //             },
        //             Command::FillRect { rect } => {
                        
        //             },
        //             Command::DrawScene { camera, viewport } => {
                        
        //             },
        //         }
        //     }
        // }
        // backend.submit_command_buffer(self.commands)?;

        // Recover statistics of previous frame
        self.statistics = backend.retrieve_statistics()?;

        Ok(())
    }

    pub fn add_camera(&mut self) -> Result<RendererHandle> {
        let handle: RendererHandle = self.generator.next();
        self.cameras.insert(handle, RendererCamera { handle: None });
        Ok(handle)
    }
    pub fn remove_camera(&mut self, handle: RendererHandle) -> Result<()> {
        if !self.cameras.contains_key(&handle) { return Err(anyhow!("Camera not found")); }
        let camera = self.cameras.remove(&handle).unwrap();
        if let Some(uid) = camera.handle {
            self.cameras_removed.insert(uid);
        }
        Ok(())
    }

    pub fn add_model(&mut self, desc: RendererModelDescriptor, asset: &AssetManager) -> Result<RendererHandle> {
        match desc {
            RendererModelDescriptor::FromAsset(uid) => {
                let model = asset.get::<Model>(uid)?;
                // Check resources
                if !self.meshes.contains_key(&model.mesh) {
                    self.requested_meshes.insert(model.mesh);
                }
                for material_uid in &model.materials {
                    let material = asset.get::<Material>(*material_uid)?;
                    if !self.textures.contains_key(&material.diffuse) {
                        self.requested_textures.insert(material.diffuse);
                    }
                    if !self.materials.contains_key(material_uid) {
                        self.requested_materials.insert(*material_uid);
                    }
                }
                // Insert model
                let handle = self.generator.next();
                self.models.insert(handle, RendererModel { 
                    mesh: model.mesh,
                    materials: model.materials.clone(),
                    handle: None 
                });
                Ok(handle)
            },
        }
    }
    pub fn remove_model(&mut self, handle: RendererHandle) -> Result<()> {
        if !self.models.contains_key(&handle) { return Err(anyhow!("Model not found")); }
        let model = self.models.remove(&handle).unwrap();
        if let Some(uid) = model.handle {
            self.models_removed.insert(uid);
        }
        Ok(())
    }

    pub fn submit_command_buffer(&mut self, buffer: CommandBuffer) -> Result<()> {
        self.commands.push(buffer);
        Ok(())
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.statistics
    }
}