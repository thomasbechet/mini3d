use std::collections::{HashMap, HashSet};

use anyhow::Result;
use glam::{UVec2, uvec2};
use serde::{Serialize, Deserialize};

use crate::{math::rect::IRect, asset::AssetManager, uid::UID, scene::SceneManager, feature::{component::{transform::TransformComponent, camera::CameraComponent, model::ModelComponent, ui::{UIComponent, SceneUIComponent}}, asset::{model::Model, material::Material, mesh::Mesh, texture::Texture, font::Font}}};

use self::{backend::{RendererBackend, BackendMaterialDescriptor, FontHandle, TextureHandle, MeshHandle, MaterialHandle, SceneCameraHandle, SceneModelHandle, CanvasHandle, SurfaceCanvasHandle, SceneCanvasHandle}};

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
// pub const SCREEN_WIDTH: u32 = 512;
// pub const SCREEN_HEIGHT: u32 = 320;
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 400;
// pub const SCREEN_WIDTH: u32 = 768;
// pub const SCREEN_HEIGHT: u32 = 480;
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
}

struct RendererFont {
    handle: FontHandle,
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
pub(crate) struct RendererResourceManager {
    fonts: HashMap<UID, RendererFont>,
    textures: HashMap<UID, RendererTexture>,
    meshes: HashMap<UID, RendererMesh>,
    materials: HashMap<UID, RendererMaterial>,
    requested_fonts: HashSet<UID>,
    requested_textures: HashSet<UID>,
    requested_meshes: HashSet<UID>,
    requested_materials: HashSet<UID>,
}

impl RendererResourceManager {
    
    fn reset(&mut self) {
        self.fonts.clear();
        self.textures.clear();
        self.meshes.clear();
        self.materials.clear();
        self.requested_fonts.clear();
        self.requested_textures.clear();
        self.requested_meshes.clear();
        self.requested_materials.clear();
    }

    fn flush_requested_resources(&mut self, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<()> {
        for font_uid in self.requested_fonts.drain() {
            let font = asset.get::<Font>(font_uid)?;
            let handle = backend.font_add(font)?;
            self.fonts.insert(font_uid, RendererFont { handle });
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

    pub(crate) fn request_font(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<FontHandle> {
        if let Some(font) = self.fonts.get(uid) {
            return Ok(font.handle);
        }
        self.requested_fonts.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.fonts.get(uid).unwrap().handle)
    }

    pub(crate) fn request_mesh(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<MeshHandle> {
        if let Some(mesh) = self.meshes.get(uid) {
            return Ok(mesh.handle);
        }
        self.requested_meshes.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.meshes.get(uid).unwrap().handle)
    }

    pub(crate) fn request_texture(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<TextureHandle> {
        if let Some(texture) = self.textures.get(uid) {
            return Ok(texture.handle);
        }
        self.requested_textures.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.textures.get(uid).unwrap().handle)
    }

    pub(crate) fn request_material(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<MaterialHandle> {
        if let Some(material) = self.materials.get(uid) {
            return Ok(material.handle);
        }
        let material = asset.get::<Material>(*uid)?;
        self.requested_textures.insert(material.diffuse);
        self.requested_materials.insert(*uid);
        self.flush_requested_resources(backend, asset)?;
        Ok(self.materials.get(uid).unwrap().handle)
    }
}

#[derive(Default)]
pub struct RendererManager {

    // Resources
    resources: RendererResourceManager,

    // Destroyed handles
    pub(crate) scene_cameras_removed: HashSet<SceneCameraHandle>,
    pub(crate) scene_models_removed: HashSet<SceneModelHandle>,
    pub(crate) scene_canvases_removed: HashSet<SceneCanvasHandle>,
    pub(crate) canvases_removed: HashSet<CanvasHandle>,
    pub(crate) surface_canvases_removed: HashSet<SurfaceCanvasHandle>,

    // Cached entities
    cameras: HashMap<hecs::Entity, SceneCameraHandle>,

    statistics: RendererStatistics,
}

impl RendererManager {

    pub(crate) fn reset(&mut self, scene: &mut SceneManager) -> Result<()> {

        self.resources.reset();

        self.scene_cameras_removed.clear();
        self.scene_models_removed.clear();
        self.scene_canvases_removed.clear();
        self.canvases_removed.clear();
        self.surface_canvases_removed.clear();

        for world in scene.iter_world() {
            for (_, camera) in world.query_mut::<&mut CameraComponent>() {
                camera.handle = None;
            }
            for (_, model) in world.query_mut::<&mut ModelComponent>() {
                model.handle = None;
            }
        }

        self.cameras.clear();
        
        Ok(())
    }

    pub(crate) fn update_backend(
        &mut self, 
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
        scene: &mut SceneManager,
    ) -> Result<()> {

        // Remove entities
        for handle in self.scene_cameras_removed.drain() {
            backend.scene_camera_remove(handle)?;
        }
        for handle in self.scene_models_removed.drain() {
            backend.scene_model_remove(handle)?;
        }
        for handle in self.scene_canvases_removed.drain() {
            backend.scene_canvas_remove(handle)?;
        }
        for handle in self.canvases_removed.drain() {
            backend.canvas_remove(handle)?;
        }
        for handle in self.surface_canvases_removed.drain() {
            backend.surface_canvas_remove(handle)?;
        }

        // Update scene components
        for world in scene.iter_world() {

            // Update cameras
            for (entity, (c, t)) in world.query_mut::<(&mut CameraComponent, &TransformComponent)>() {
                if c.handle.is_none() {
                    let handle = backend.scene_camera_add()?; 
                    self.cameras.insert(entity, handle);
                    c.handle = Some(handle);
                }
                backend.scene_camera_update(c.handle.unwrap(), t.translation, t.forward(), t.up(), c.fov)?;
            }
            
            // Update models
            for (_, (m, t)) in world.query_mut::<(&mut ModelComponent, &TransformComponent)>() {
                if m.handle.is_none() {
                    let model = asset.get::<Model>(m.model)?;
                    let mesh_handle = self.resources.request_mesh(&model.mesh, backend, asset)?;
                    let handle = backend.scene_model_add(mesh_handle)?;
                    for (index, material) in model.materials.iter().enumerate() {
                        let material_handle = self.resources.request_material(material, backend, asset)?;
                        backend.scene_model_set_material(handle, index, material_handle)?;
                    }
                    m.handle = Some(handle);
                }
                backend.scene_model_transfer_matrix(m.handle.unwrap(), t.matrix())?;
            }

            // Update Surface Canvas
            for (_, c) in world.query_mut::<&mut UIComponent>() {

                // Update UI
                c.ui.update_renderer(backend, &mut self.resources, &self.cameras, asset)?;

                // Update surface
                if c.handle.is_none() && c.ui.handle.is_some() {
                    c.handle = Some(backend.surface_canvas_add(c.ui.handle.unwrap(), c.position, c.z_index)?);
                }
            }

            // Update Scene Canvas
            for (_, (c, t)) in world.query_mut::<(&mut SceneUIComponent, &TransformComponent)>() {

                // Update UI
                c.ui.update_renderer(backend, &mut self.resources, &self.cameras, asset)?;

                // Update scene object
                if c.handle.is_none() && c.ui.handle.is_some() {
                    c.handle = Some(backend.scene_canvas_add(c.ui.handle.unwrap())?);
                }
                backend.scene_canvas_transfer_matrix(c.handle.unwrap(), t.matrix())?;
            }
        }

        // Recover statistics of previous frame
        self.statistics = backend.statistics()?;

        Ok(())
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.statistics
    }
}