use std::collections::{HashMap, HashSet};

use anyhow::Result;
use glam::{UVec2, uvec2};
use serde::{Serialize, Deserialize};

use crate::{math::rect::IRect, asset::AssetManager, uid::UID, scene::SceneManager, feature::{component::{transform::TransformComponent, camera::CameraComponent, model::ModelComponent}, asset::{model::Model, material::Material, mesh::Mesh, texture::Texture, font::Font}}};

use self::{backend::{RendererBackend, BackendMaterialDescriptor, FontHandle, TextureHandle, MeshHandle, MaterialHandle, CameraHandle, ModelHandle}, command_buffer::CommandBuffer};

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

pub enum RendererModelDescriptor {
    FromAsset(UID),
}

#[derive(Default, Clone, Copy, Serialize, Deserialize)]
pub struct RendererStatistics {
    pub triangle_count: usize,
    pub draw_count: usize,
    pub viewport: (u32, u32),
}

struct RendererFont {
    _handle: FontHandle,
}

struct RendererTexture {
    handle: TextureHandle,
}

struct RendererMesh {
    handle: MeshHandle,
}

struct RendererMaterial {
    handle: MaterialHandle,
}

#[derive(Default)]
pub struct RendererManager {

    // Resources

    fonts: HashMap<UID, RendererFont>,
    textures: HashMap<UID, RendererTexture>,
    meshes: HashMap<UID, RendererMesh>,
    materials: HashMap<UID, RendererMaterial>,
    requested_fonts: HashSet<UID>,
    requested_textures: HashSet<UID>,
    requested_meshes: HashSet<UID>,
    requested_materials: HashSet<UID>,

    // Entities

    pub(crate) cameras_removed: HashSet<CameraHandle>,
    pub(crate) models_removed: HashSet<ModelHandle>,

    command_buffers: Vec<CommandBuffer>,
    statistics: RendererStatistics,
}

impl RendererManager {

    pub(crate) fn prepare(&mut self) -> Result<()> {
        self.command_buffers.clear();
        Ok(())
    }

    pub(crate) fn reset(&mut self, scene: &mut SceneManager) -> Result<()> {

        self.fonts.clear();
        self.textures.clear();
        self.meshes.clear();
        self.materials.clear();
        self.requested_fonts.clear();
        self.requested_textures.clear();
        self.requested_meshes.clear();
        self.requested_materials.clear();

        self.cameras_removed.clear();
        self.models_removed.clear();
        
        self.command_buffers.clear();

        for world in scene.iter_world() {
            for (_, camera) in world.query_mut::<&mut CameraComponent>() {
                camera.handle = None;
            }
            for (_, model) in world.query_mut::<&mut ModelComponent>() {
                model.handle = None;
            }
        }
        
        Ok(())
    }

    fn flush_requested_resources(&mut self, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<()> {
        for font_uid in self.requested_fonts.drain() {
            let font = asset.get::<Font>(font_uid)?;
            let handle = backend.font_add(font)?;
            self.fonts.insert(font_uid, RendererFont { _handle: handle });
        }
        for mesh_uid in self.requested_meshes.drain() {
            let mesh = asset.get::<Mesh>(mesh_uid)?;
            let handle = backend.mesh_add(mesh)?;
            self.meshes.insert(mesh_uid, RendererMesh { handle });
        }
        for texture_uid in self.requested_textures.drain() {
            let texture = asset.get::<Texture>(texture_uid)?;
            let handle = backend.texture_add(texture)?;
            self.textures.insert(texture_uid, RendererTexture { handle });
        }
        for material_uid in self.requested_materials.drain() {
            let material = asset.entry::<Material>(material_uid)?;
            let diffuse = self.textures.get(&material.asset.diffuse).unwrap().handle;
            let handle = backend.material_add(BackendMaterialDescriptor { diffuse, name: &material.name })?;
            self.materials.insert(material_uid, RendererMaterial { handle });
        }
        Ok(())
    }

    fn _request_font(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<FontHandle> {
        if let Some(font) = self.fonts.get(uid) {
            return Ok(font._handle);
        }
        self.requested_fonts.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.fonts.get(uid).unwrap()._handle)
    }

    fn request_mesh(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<MeshHandle> {
        if let Some(mesh) = self.meshes.get(uid) {
            return Ok(mesh.handle);
        }
        self.requested_meshes.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.meshes.get(uid).unwrap().handle)
    }

    fn _request_texture(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<TextureHandle> {
        if let Some(texture) = self.textures.get(uid) {
            return Ok(texture.handle);
        }
        self.requested_textures.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.textures.get(uid).unwrap().handle)
    }

    fn request_material(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<MaterialHandle> {
        if let Some(material) = self.materials.get(uid) {
            return Ok(material.handle);
        }
        let material = asset.get::<Material>(*uid)?;
        self.requested_textures.insert(material.diffuse);
        self.requested_materials.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.materials.get(uid).unwrap().handle)
    }
 
    pub(crate) fn update_backend(
        &mut self, 
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
        scene: &mut SceneManager,
    ) -> Result<()> {

        // Remove entities
        for handle in self.cameras_removed.drain() {
            backend.camera_remove(handle)?;
        }
        for handle in self.models_removed.drain() {
            backend.model_remove(handle)?;
        }

        // Update scene components
        for world in scene.iter_world() {

            // Update cameras
            for (_, (c, t)) in world.query_mut::<(&mut CameraComponent, &TransformComponent)>() {
                if c.handle.is_none() {
                    c.handle = Some(backend.camera_add()?);
                }
                backend.camera_update(c.handle.unwrap(), t.translation, t.forward(), t.up(), c.fov)?;
            }
            
            // Update models
            for (_, (m, t)) in world.query_mut::<(&mut ModelComponent, &TransformComponent)>() {
                if m.handle.is_none() {
                    let model = asset.get::<Model>(m.model)?;
                    let mesh_handle = self.request_mesh(&model.mesh, backend, asset)?;
                    let handle = backend.model_add(mesh_handle)?;
                    for (index, material) in model.materials.iter().enumerate() {
                        let material_handle = self.request_material(material, backend, asset)?;
                        backend.model_set_material(handle, index, material_handle)?;
                    }
                    m.handle = Some(handle);
                }
                backend.model_transfer_matrix(m.handle.unwrap(), t.matrix())?;
            }
        }

        // Send commands
        for command_buffer in self.command_buffers.drain(..) {
            backend.submit_command_buffer(command_buffer)?;
        }

        // Recover statistics of previous frame
        self.statistics = backend.statistics()?;

        Ok(())
    }

    pub fn submit_command_buffer(&mut self, buffer: CommandBuffer) -> Result<()> {
        self.command_buffers.push(buffer);
        Ok(())
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.statistics
    }
}