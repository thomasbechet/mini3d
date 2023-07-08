use std::collections::{hash_map, HashMap, HashSet};

use crate::serialize::{Decoder, DecoderError, Serialize};
use crate::{
    asset::AssetManager,
    ecs::{entity::Entity, view::StaticComponentView, ECSManager},
    feature::{
        asset::{
            font::{Font, FontAtlas},
            material::Material,
            mesh::Mesh,
            model::Model,
            texture::Texture,
        },
        component::{
            camera::Camera, canvas::Canvas, local_to_world::LocalToWorld, static_mesh::StaticMesh,
            viewport::Viewport,
        },
    },
    math::rect::IRect,
    registry::{asset::Asset, component::Component},
    serialize::{Encoder, EncoderError},
    uid::UID,
};
use glam::{uvec2, UVec2};
use mini3d_derive::Serialize;

use self::{
    backend::{
        BackendMaterialDescriptor, MaterialHandle, MeshHandle, RendererBackend,
        RendererBackendError, SceneCameraHandle, SceneCanvasHandle, SceneHandle, SceneModelHandle,
        TextureHandle, ViewportHandle,
    },
    color::Color,
    graphics::Graphics,
};

pub mod backend;
pub mod color;
pub mod graphics;
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

#[derive(Default, Clone, Copy, Serialize)]
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

fn load_font(
    uid: UID,
    backend: &mut impl RendererBackend,
    asset: &AssetManager,
) -> Result<RendererFont, RendererBackendError> {
    let font = asset.get::<Font>(Font::UID, uid).unwrap().unwrap();
    let atlas = FontAtlas::new(font);
    let handle = backend.texture_add(&atlas.texture)?;
    Ok(RendererFont { atlas, handle })
}

fn load_mesh(
    uid: UID,
    backend: &mut impl RendererBackend,
    asset: &AssetManager,
) -> Result<RendererMesh, RendererBackendError> {
    let mesh = asset.get::<Mesh>(Mesh::UID, uid).unwrap().unwrap();
    let handle = backend.mesh_add(mesh)?;
    Ok(RendererMesh { handle })
}

fn load_texture(
    uid: UID,
    backend: &mut impl RendererBackend,
    asset: &AssetManager,
) -> Result<RendererTexture, RendererBackendError> {
    let texture = asset.get::<Texture>(Texture::UID, uid).unwrap().unwrap();
    let handle = backend.texture_add(texture)?;
    Ok(RendererTexture { handle })
}

fn load_material(
    uid: UID,
    textures: &HashMap<UID, RendererTexture>,
    backend: &mut impl RendererBackend,
    asset: &AssetManager,
) -> Result<RendererMaterial, RendererBackendError> {
    let material = asset
        .entry::<Material>(Material::UID, uid)
        .unwrap()
        .unwrap();
    let diffuse = textures.get(&material.asset.diffuse).unwrap().handle;
    let handle = backend.material_add(BackendMaterialDescriptor {
        diffuse,
        name: &material.name,
    })?;
    Ok(RendererMaterial { handle })
}

impl RendererResourceManager {
    fn reset(&mut self) {
        self.fonts.clear();
        self.textures.clear();
        self.meshes.clear();
        self.materials.clear();
    }

    pub(crate) fn request_font<'a>(
        &'a mut self,
        uid: &UID,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
    ) -> Result<&'a RendererFont, RendererBackendError> {
        match self.fonts.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let font = load_font(*uid, backend, asset)?;
                Ok(e.insert(font))
            }
        }
    }

    pub(crate) fn request_mesh(
        &mut self,
        uid: &UID,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
    ) -> Result<&RendererMesh, RendererBackendError> {
        match self.meshes.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let mesh = load_mesh(*uid, backend, asset)?;
                Ok(e.insert(mesh))
            }
        }
    }

    pub(crate) fn request_texture(
        &mut self,
        uid: &UID,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
    ) -> Result<&RendererTexture, RendererBackendError> {
        match self.textures.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let texture = load_texture(*uid, backend, asset)?;
                Ok(e.insert(texture))
            }
        }
    }

    pub(crate) fn request_material(
        &mut self,
        uid: &UID,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
    ) -> Result<&RendererMaterial, RendererBackendError> {
        match self.materials.entry(*uid) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let material = asset.get::<Material>(Material::UID, *uid).unwrap().unwrap();
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

    // Cached resources
    scenes: HashMap<UID, SceneHandle>,
    cameras: HashMap<Entity, SceneCameraHandle>,
    viewports: HashMap<Entity, ViewportHandle>,

    // Persistent data
    statistics: RendererStatistics,
    graphics: Graphics,
    clear_color: Color,
}

impl RendererManager {
    pub(crate) fn reset(&mut self, ecs: &mut ECSManager) {
        self.resources.reset();

        self.scene_cameras_removed.clear();
        self.scene_models_removed.clear();
        self.scene_canvases_removed.clear();

        for world in ecs.worlds.get_mut().values_mut() {
            for camera in world
                .get_mut()
                .view_mut_static::<Camera>(Camera::UID)
                .unwrap()
                .iter()
            {
                camera.handle = None;
            }
            for static_mesh in world
                .get_mut()
                .view_mut_static::<StaticMesh>(StaticMesh::UID)
                .unwrap()
                .iter()
            {
                static_mesh.handle = None;
            }
            for canvas in world
                .get_mut()
                .view_mut_static::<Canvas>(Canvas::UID)
                .unwrap()
                .iter()
            {
                canvas.handle = None;
            }
        }

        self.scenes.clear();
        self.cameras.clear();
    }

    pub(crate) fn synchronize_backend(
        &mut self,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
        ecs: &mut ECSManager,
    ) -> Result<(), RendererBackendError> {
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

        // Update scene
        if let hash_map::Entry::Vacant(e) = self.scenes.entry(ecs.active_world) {
            let handle = backend.scene_add()?;
            e.insert(handle);
        }

        // Update scene components
        {
            let world = ecs
                .worlds
                .get_mut()
                .get_mut(&ecs.active_world)
                .unwrap()
                .get_mut();

            // Prepare views
            let local_to_world = world
                .view_mut_static::<LocalToWorld>(LocalToWorld::UID)
                .unwrap();
            let mut cameras = world.view_mut_static::<Camera>(Camera::UID).unwrap();
            let mut viewports = world.view_mut_static::<Viewport>(Viewport::UID).unwrap();
            let mut static_meshes = world
                .view_mut_static::<StaticMesh>(StaticMesh::UID)
                .unwrap();
            let mut canvases = world.view_mut_static::<Canvas>(Canvas::UID).unwrap();

            // Update cameras
            for e in &world.query(&[Camera::UID, LocalToWorld::UID]) {
                let c = cameras.get_mut(e).unwrap();
                let t = local_to_world.get(e).unwrap();
                if c.handle.is_none() {
                    let handle = backend.scene_camera_add()?;
                    self.cameras.insert(e, handle);
                    c.handle = Some(handle);
                }
                backend.scene_camera_update(
                    c.handle.unwrap(),
                    t.translation(),
                    t.forward(),
                    t.up(),
                    c.fov,
                )?;
            }

            // Update viewports
            for e in &world.query(&[Viewport::UID]) {
                let v = viewports.get_mut(e).unwrap();
                if v.handle.is_none() {
                    v.handle = Some(backend.viewport_add(v.resolution)?);
                    v.out_of_date = true;
                    self.viewports.insert(e, v.handle.unwrap());
                }
                if v.out_of_date {
                    let camera = v.camera.map(|entity| *self.cameras.get(&entity).unwrap());
                    backend.viewport_set_camera(v.handle.unwrap(), camera)?;
                    backend.viewport_set_resolution(v.handle.unwrap(), v.resolution)?;
                    v.out_of_date = false;
                }
            }

            // Update static meshes
            for e in &world.query(&[StaticMesh::UID, LocalToWorld::UID]) {
                let s = static_meshes.get_mut(e).unwrap();
                let t = local_to_world.get(e).unwrap();
                if s.handle.is_none() {
                    let model: &Model = asset.get(Model::UID, s.model).unwrap().unwrap();
                    let mesh_handle = self
                        .resources
                        .request_mesh(&model.mesh, backend, asset)?
                        .handle;
                    let handle = backend.scene_model_add(mesh_handle)?;
                    for (index, material) in model.materials.iter().enumerate() {
                        let material_handle = self
                            .resources
                            .request_material(material, backend, asset)?
                            .handle;
                        backend.scene_model_set_material(handle, index, material_handle)?;
                    }
                    s.handle = Some(handle);
                }
                backend.scene_model_transfer_matrix(s.handle.unwrap(), t.matrix)?;
            }

            // Update Scene Canvas
            for e in &world.query(&[Canvas::UID, LocalToWorld::UID]) {
                let c = canvases.get_mut(e).unwrap();
                let t = local_to_world.get(e).unwrap();
                if c.handle.is_none() {
                    c.handle = Some(backend.scene_canvas_add(c.resolution)?);
                }
                backend.scene_canvas_transfer_matrix(c.handle.unwrap(), t.matrix)?;
            }
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

        // Recover statistics of previous frame
        self.statistics = backend.statistics()?;

        Ok(())
    }

    pub(crate) fn prepare(&mut self) {
        self.graphics.clear();
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.graphics.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        self.graphics = Graphics::deserialize(decoder, &Default::default())?;
        Ok(())
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
