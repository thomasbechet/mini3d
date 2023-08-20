use std::collections::{hash_map, HashMap};

use crate::asset::handle::{AssetHandle, StaticAsset};
use crate::ecs::component::ComponentTable;
use crate::feature::component::renderer::camera::Camera;
use crate::feature::component::renderer::font::{Font, FontAtlas};
use crate::feature::component::renderer::material::Material;
use crate::feature::component::renderer::mesh::Mesh;
use crate::feature::component::renderer::model::Model;
use crate::feature::component::renderer::static_mesh::StaticMesh;
use crate::feature::component::renderer::texture::Texture;
use crate::feature::component::renderer::viewport::Viewport;
use crate::feature::component::scene::local_to_world::LocalToWorld;
use crate::feature::component::ui::canvas::Canvas;
use crate::registry::component::{Component, ComponentRegistry, StaticComponent};
use crate::registry::error::RegistryError;
use crate::serialize::{Decoder, DecoderError, Serialize};
use crate::utils::generation::GenerationId;
use crate::utils::uid::UID;
use crate::{
    asset::AssetManager,
    ecs::ECSManager,
    math::rect::IRect,
    serialize::{Encoder, EncoderError},
};
use glam::{uvec2, UVec2};
use mini3d_derive::Serialize;

use self::event::RendererEvent;
use self::{
    backend::{
        BackendMaterialDescriptor, MaterialHandle, MeshHandle, RendererBackend,
        RendererBackendError, TextureHandle,
    },
    color::Color,
    graphics::Graphics,
};

pub mod backend;
pub mod color;
pub mod event;
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
    fonts: HashMap<GenerationId, RendererFont>,
    textures: HashMap<GenerationId, RendererTexture>,
    meshes: HashMap<GenerationId, RendererMesh>,
    materials: HashMap<GenerationId, RendererMaterial>,
}

fn load_font(
    handle: StaticAsset<Font>,
    backend: &mut dyn RendererBackend,
    asset: &AssetManager,
) -> Result<RendererFont, RendererBackendError> {
    let font = asset.read(handle).unwrap();
    let atlas = FontAtlas::new(font);
    let handle = backend.texture_add(&atlas.texture)?;
    Ok(RendererFont { atlas, handle })
}

fn load_mesh(
    handle: StaticAsset<Mesh>,
    backend: &mut dyn RendererBackend,
    asset: &AssetManager,
) -> Result<RendererMesh, RendererBackendError> {
    let mesh = asset.read(handle).unwrap();
    let handle = backend.mesh_add(mesh)?;
    Ok(RendererMesh { handle })
}

fn load_texture(
    handle: StaticAsset<Texture>,
    backend: &mut dyn RendererBackend,
    asset: &AssetManager,
) -> Result<RendererTexture, RendererBackendError> {
    let texture = asset.read(handle).unwrap();
    let handle = backend.texture_add(texture)?;
    Ok(RendererTexture { handle })
}

fn load_material(
    handle: StaticAsset<Material>,
    textures: &HashMap<GenerationId, RendererTexture>,
    backend: &mut dyn RendererBackend,
    asset: &AssetManager,
) -> Result<RendererMaterial, RendererBackendError> {
    let material = asset.read(handle).unwrap();
    let info = asset.info(handle).unwrap();
    let diffuse = textures.get(&material.diffuse.id()).unwrap().handle;
    let handle = backend.material_add(BackendMaterialDescriptor {
        diffuse,
        name: &info.name,
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
        handle: StaticAsset<Font>,
        backend: &mut impl RendererBackend,
        asset: &AssetManager,
    ) -> Result<&'a RendererFont, RendererBackendError> {
        match self.fonts.entry(handle.id()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let font = load_font(handle, backend, asset)?;
                Ok(e.insert(font))
            }
        }
    }

    pub(crate) fn request_mesh(
        &mut self,
        handle: StaticAsset<Mesh>,
        backend: &mut dyn RendererBackend,
        asset: &AssetManager,
    ) -> Result<&RendererMesh, RendererBackendError> {
        match self.meshes.entry(handle.id()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let mesh = load_mesh(handle, backend, asset)?;
                Ok(e.insert(mesh))
            }
        }
    }

    pub(crate) fn request_texture(
        &mut self,
        handle: StaticAsset<Texture>,
        backend: &mut dyn RendererBackend,
        asset: &AssetManager,
    ) -> Result<&RendererTexture, RendererBackendError> {
        match self.textures.entry(handle.id()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let texture = load_texture(handle, backend, asset)?;
                Ok(e.insert(texture))
            }
        }
    }

    pub(crate) fn request_material(
        &mut self,
        handle: StaticAsset<Material>,
        backend: &mut dyn RendererBackend,
        asset: &AssetManager,
    ) -> Result<&RendererMaterial, RendererBackendError> {
        match self.materials.entry(handle.id()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let material = asset.read(handle).unwrap();
                if let hash_map::Entry::Vacant(e) = self.textures.entry(material.diffuse.id()) {
                    let diffuse = load_texture(material.diffuse, backend, asset)?;
                    e.insert(diffuse);
                }
                let material = load_material(handle, &self.textures, backend, asset)?;
                Ok(e.insert(material))
            }
        }
    }
}

#[derive(Default)]
pub struct RendererManager {
    // Resources
    pub(crate) resources: RendererResourceManager,

    // Persistent data
    statistics: RendererStatistics,
    graphics: Graphics,
    clear_color: Color,

    // Components
    camera: StaticComponent<Camera>,
    static_mesh: StaticComponent<StaticMesh>,
    canvas: StaticComponent<Canvas>,
    local_to_world: StaticComponent<LocalToWorld>,
    viewport: StaticComponent<Viewport>,
    model: StaticComponent<Model>,
}

impl RendererManager {
    pub(crate) fn reset(&mut self, ecs: &mut ECSManager, backend: &mut dyn RendererBackend) {
        self.resources.reset();
    }

    pub(crate) fn reload_component_handles(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<(), RegistryError> {
        self.camera = registry
            .find(Camera::UID)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        self.static_mesh = registry
            .find(StaticMesh::UID)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        self.canvas = registry
            .find(Canvas::UID)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        self.local_to_world = registry
            .find(LocalToWorld::UID)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        self.viewport = registry
            .find(Viewport::UID)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        self.model = registry
            .find(Model::UID)
            .ok_or(RegistryError::ComponentDefinitionNotFound)?;
        Ok(())
    }

    pub(crate) fn dispatch_events(&mut self, backend: &mut impl RendererBackend) {
        for event in backend.events() {
            match event {
                RendererEvent::Statistics(stats) => self.statistics = *stats,
            }
        }
    }

    pub(crate) fn prepare(&mut self) {
        self.graphics.clear();
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.graphics.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn load_state(
        &mut self,
        decoder: &mut impl Decoder,
        backend: &mut impl RendererBackend,
    ) -> Result<(), DecoderError> {
        // Reset all previous resources
        backend.reset();
        // TODO: reset ECS resources
        self.graphics = Graphics::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub(crate) fn submit_graphics(
        &mut self,
        asset: &mut AssetManager,
        components: &ComponentTable,
        backend: &mut impl RendererBackend,
    ) {
        // Acquire active scene
        let viewports = components
            .view(self.viewport)
            .expect("Failed to acquire viewport view");
        // Render main screen
        self.graphics.submit_backend(
            None,
            Color::TRANSPARENT,
            &mut self.resources,
            asset,
            &viewports,
            backend,
        );
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
