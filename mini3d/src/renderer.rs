use std::collections::{HashMap, HashSet, hash_map};

use anyhow::{Result, Context};
use glam::{UVec2, uvec2};
use serde::{Serialize, Deserialize, Serializer, ser::SerializeTuple, Deserializer, de::Visitor};

use crate::{math::rect::IRect, asset::AssetManager, uid::UID, feature::{component::{local_to_world::LocalToWorld, camera::Camera, model::Model, viewport::Viewport, canvas::Canvas}, asset::{material::Material, mesh::Mesh, texture::Texture, font::{Font, FontAtlas}, self}}, ecs::{ECSManager, entity::Entity}};

use self::{backend::{RendererBackend, BackendMaterialDescriptor, TextureHandle, MeshHandle, MaterialHandle, SceneCameraHandle, SceneModelHandle, SceneCanvasHandle, ViewportHandle}, graphics::Graphics, color::Color};

pub mod backend;
pub mod color;
pub mod rasterizer;
pub mod graphics;

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

pub(crate) struct RendererFont {
    pub(crate) atlas: FontAtlas,
    pub(crate) handle: TextureHandle,
}

pub(crate) struct RendererTexture {
    pub(crate) handle: TextureHandle,
}

pub(crate) struct RendererMesh {
    pub(crate) handle: MeshHandle,
}

pub(crate) struct RendererMaterial {
    pub(crate) handle: MaterialHandle,
}

#[derive(Default)]
pub(crate) struct RendererResourceManager {
    fonts: HashMap<UID, RendererFont>,
    textures: HashMap<UID, RendererTexture>,
    meshes: HashMap<UID, RendererMesh>,
    materials: HashMap<UID, RendererMaterial>,
}

fn load_font(uid: UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<RendererFont> {
    let font = asset.get::<Font>(Font::UID, uid)?;
    let atlas = FontAtlas::new(font);
    let handle = backend.texture_add(&atlas.texture)?;
    Ok(RendererFont { atlas, handle })
}

fn load_mesh(uid: UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<RendererMesh> {
    let mesh = asset.get::<Mesh>(Mesh::UID, uid)?;
    let handle = backend.mesh_add(mesh)?;
    Ok(RendererMesh { handle })
}

fn load_texture(uid: UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<RendererTexture> {
    let texture = asset.get::<Texture>(Texture::UID, uid)?;
    let handle = backend.texture_add(texture)?;
    Ok(RendererTexture { handle })
}

fn load_material(uid: UID, textures: &HashMap<UID, RendererTexture>, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<RendererMaterial> {
    let material = asset.entry::<Material>(Material::UID, uid)?;
    let diffuse = textures.get(&material.asset.diffuse).unwrap().handle;
    let handle = backend.material_add(BackendMaterialDescriptor { diffuse, name: &material.name })?;
    Ok(RendererMaterial { handle })
}

impl RendererResourceManager {
    
    fn reset(&mut self) {
        self.fonts.clear();
        self.textures.clear();
        self.meshes.clear();
        self.materials.clear();
    }

    pub(crate) fn request_font<'a>(&'a mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<&'a RendererFont> {
        match self.fonts.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let font = load_font(*uid, backend, asset)?;
                Ok(e.insert(font))
            }
        }
    }

    pub(crate) fn request_mesh(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<&RendererMesh> {
        match self.meshes.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let mesh = load_mesh(*uid, backend, asset)?;
                Ok(e.insert(mesh))
            }
        }
    }

    pub(crate) fn request_texture(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<&RendererTexture> {
        match self.textures.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let texture = load_texture(*uid, backend, asset)?;
                Ok(e.insert(texture))
            }
        }
    }

    pub(crate) fn request_material(&mut self, uid: &UID, backend: &mut impl RendererBackend, asset: &AssetManager) -> Result<&RendererMaterial> {
        match self.materials.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let material = asset.get::<Material>(Material::UID, *uid)?;
                if let hash_map::Entry::Vacant(e) = self.textures.entry(material.diffuse) {
                    let diffuse = load_texture(*uid, backend, asset)?;
                    e.insert(diffuse);
                }
                let material = load_material(*uid, &self.textures, backend, asset)?;
                Ok(e.insert(material))
            }
        }
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
    pub(crate) viewports_removed: HashSet<ViewportHandle>,

    // Cached entities
    cameras: HashMap<Entity, SceneCameraHandle>,
    viewports: HashMap<Entity, ViewportHandle>,

    // Persistent data
    statistics: RendererStatistics,
    graphics: Graphics,
    clear_color: Color,
}

impl RendererManager {

    pub(crate) fn reset(&mut self, ecs: &mut ECSManager) -> Result<()> {

        self.resources.reset();

        self.scene_cameras_removed.clear();
        self.scene_models_removed.clear();
        self.scene_canvases_removed.clear();

        for world in scene.iter_worlds_mut() {

            let cameras = world.view_mut::<Camera>("camera".into())?;

            for (_, camera) in world.query_mut::<&mut Camera>() {
                camera.handle = None;
            }
            for (_, model) in world.query_mut::<&mut Model>() {
                model.handle = None;
            }
            for (_, canvas) in world.query_mut::<&mut Canvas>() {
                canvas.handle = None;
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
        for handle in self.scene_canvases_removed.drain() {
            backend.scene_canvas_remove(handle)?;
        }
        for handle in self.viewports_removed.drain() {
            backend.viewport_remove(handle)?;
        }

        // Update scene components
        for world in scene.iter_worlds_mut() {

            // Update cameras
            for (entity, (c, t)) in world.query_mut::<(&mut Camera, &LocalToWorld)>() {
                if c.handle.is_none() {
                    let handle = backend.scene_camera_add()?;
                    self.cameras.insert(entity, handle);
                    c.handle = Some(handle);
                }
                backend.scene_camera_update(c.handle.unwrap(), t.translation(), t.forward(), t.up(), c.fov)?;
            }

            // Update viewports
            for (e, viewport) in world.query_mut::<&mut Viewport>() {
                if viewport.handle.is_none() {
                    viewport.handle = Some(backend.viewport_add(viewport.resolution)?);
                    viewport.out_of_date = true;
                    self.viewports.insert(e, viewport.handle.unwrap());
                }
                if viewport.out_of_date {
                    let camera = viewport.camera.map(|entity| *self.cameras.get(&entity).unwrap());
                    backend.viewport_set_camera(viewport.handle.unwrap(), camera)?;
                    backend.viewport_set_resolution(viewport.handle.unwrap(), viewport.resolution)?;
                    viewport.out_of_date = false;
                }
            }
            
            // Update models
            for (_, (m, t)) in world.query_mut::<(&mut Model, &LocalToWorld)>() {
                if m.handle.is_none() {
                    let model = asset.get::<asset::model::Model>(m.model)?;
                    let mesh_handle = self.resources.request_mesh(&model.mesh, backend, asset)?.handle;
                    let handle = backend.scene_model_add(mesh_handle)?;
                    for (index, material) in model.materials.iter().enumerate() {
                        let material_handle = self.resources.request_material(material, backend, asset)?.handle;
                        backend.scene_model_set_material(handle, index, material_handle)?;
                    }
                    m.handle = Some(handle);
                }
                backend.scene_model_transfer_matrix(m.handle.unwrap(), t.matrix)?;
            }

            // Update Scene Canvas
            for (_, (c, t)) in world.query_mut::<(&mut Canvas, &LocalToWorld)>() {

                if c.handle.is_none() {
                    c.handle = Some(backend.scene_canvas_add(c.resolution)?);
                }
                backend.scene_canvas_transfer_matrix(c.handle.unwrap(), t.matrix)?;        
            }

            // Render main screen
            self.graphics.submit_backend(
                None,
                Color::TRANSPARENT,
                &mut self.resources,
                asset,
                &self.viewports,
                backend,
            )?;
        }

        // Recover statistics of previous frame
        self.statistics = backend.statistics()?;

        Ok(())
    }

    pub(crate) fn prepare(&mut self) -> Result<()> {
        self.graphics.clear();
        Ok(())
    }

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(1)?;
        tuple.serialize_element(&self.graphics)?;
        tuple.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct RendererVisitor<'a> {
            manager: &'a mut RendererManager,
        }
        impl<'de, 'a> Visitor<'de> for RendererVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("Renderer manager data")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: serde::de::SeqAccess<'de> {
                use serde::de::Error;
                self.manager.graphics = seq.next_element()?.with_context(|| "Expect graphics").map_err(Error::custom)?;
                Ok(())
            }
        }
        deserializer.deserialize_tuple(1, RendererVisitor { manager: self })
    }

    pub fn graphics(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    pub fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub fn statistics(&self) -> RendererStatistics {
        self.statistics
    }
}