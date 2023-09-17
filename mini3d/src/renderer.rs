use std::collections::{hash_map, HashMap};

use crate::asset::handle::{Asset, StaticAsset};
use crate::asset::AssetManager;
use crate::ecs::container::ContainerTable;
use crate::ecs::ECSManager;
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
use crate::registry::component::{ComponentRegistry, StaticComponent};
use crate::registry::error::RegistryError;
use crate::serialize::{Decoder, DecoderError, Serialize};
use crate::utils::uid::UID;
use crate::{
    math::rect::IRect,
    serialize::{Encoder, EncoderError},
};
use glam::{uvec2, UVec2};
use mini3d_derive::Serialize;

use self::event::RendererEvent;
use self::{
    color::Color,
    graphics::Graphics,
    provider::{
        MaterialHandle, MeshHandle, ProviderMaterialDescriptor, RendererProvider,
        RendererProviderError, TextureHandle,
    },
};

pub mod color;
pub mod event;
pub mod graphics;
pub mod provider;
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
    fonts: HashMap<Asset, RendererFont>,
    textures: HashMap<Asset, RendererTexture>,
    meshes: HashMap<Asset, RendererMesh>,
    materials: HashMap<Asset, RendererMaterial>,
}

fn load_font(
    handle: StaticAsset<Font>,
    provider: &mut dyn RendererProvider,
    asset: &AssetManager,
) -> Result<RendererFont, RendererProviderError> {
    let font = asset.read(handle).unwrap();
    let atlas = FontAtlas::new(font);
    let handle = provider.texture_add(&atlas.texture)?;
    Ok(RendererFont { atlas, handle })
}

fn load_mesh(
    handle: StaticAsset<Mesh>,
    provider: &mut dyn RendererProvider,
    asset: &AssetManager,
) -> Result<RendererMesh, RendererProviderError> {
    let mesh = asset.read(handle).unwrap();
    let handle = provider.mesh_add(mesh)?;
    Ok(RendererMesh { handle })
}

fn load_texture(
    handle: StaticAsset<Texture>,
    provider: &mut dyn RendererProvider,
    asset: &AssetManager,
) -> Result<RendererTexture, RendererProviderError> {
    let texture = asset.read(handle).unwrap();
    let handle = provider.texture_add(texture)?;
    Ok(RendererTexture { handle })
}

fn load_material(
    handle: StaticAsset<Material>,
    textures: &HashMap<Asset, RendererTexture>,
    provider: &mut dyn RendererProvider,
    asset: &AssetManager,
) -> Result<RendererMaterial, RendererProviderError> {
    let material = asset.read(handle).unwrap();
    let info = asset.info(handle).unwrap();
    let diffuse = textures.get(&material.diffuse.into()).unwrap().handle;
    let handle = provider.material_add(ProviderMaterialDescriptor {
        diffuse,
        name: info.name,
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
        provider: &mut dyn RendererProvider,
        asset: &AssetManager,
    ) -> Result<&'a RendererFont, RendererProviderError> {
        match self.fonts.entry(handle.into()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let font = load_font(handle, provider, asset)?;
                Ok(e.insert(font))
            }
        }
    }

    pub(crate) fn request_mesh(
        &mut self,
        handle: StaticAsset<Mesh>,
        provider: &mut dyn RendererProvider,
        asset: &AssetManager,
    ) -> Result<&RendererMesh, RendererProviderError> {
        match self.meshes.entry(handle.into()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let mesh = load_mesh(handle, provider, asset)?;
                Ok(e.insert(mesh))
            }
        }
    }

    pub(crate) fn request_texture(
        &mut self,
        handle: StaticAsset<Texture>,
        provider: &mut dyn RendererProvider,
        asset: &AssetManager,
    ) -> Result<&RendererTexture, RendererProviderError> {
        match self.textures.entry(handle.into()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let texture = load_texture(handle, provider, asset)?;
                Ok(e.insert(texture))
            }
        }
    }

    pub(crate) fn request_material(
        &mut self,
        handle: StaticAsset<Material>,
        provider: &mut dyn RendererProvider,
        asset: &AssetManager,
    ) -> Result<&RendererMaterial, RendererProviderError> {
        match self.materials.entry(handle.into()) {
            hash_map::Entry::Occupied(e) => Ok(&*e.into_mut()),
            hash_map::Entry::Vacant(e) => {
                let material = asset.read(handle).unwrap();
                if let hash_map::Entry::Vacant(e) = self.textures.entry(material.diffuse.into()) {
                    let diffuse = load_texture(material.diffuse, provider, asset)?;
                    e.insert(diffuse);
                }
                let material = load_material(handle, &self.textures, provider, asset)?;
                Ok(e.insert(material))
            }
        }
    }
}

#[derive(Default)]
pub struct RendererManager {
    pub(crate) provider: Box<dyn RendererProvider>,

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
    pub(crate) fn set_provider(&mut self, provider: Box<dyn RendererProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    pub(crate) fn reset(&mut self, ecs: &mut ECSManager) {
        self.resources.reset();
    }

    pub(crate) fn reload_component_handles(
        &mut self,
        registry: &ComponentRegistry,
    ) -> Result<(), RegistryError> {
        self.camera = registry
            .find(Camera::NAME)
            .ok_or(RegistryError::ComponentNotFound)?;
        self.static_mesh = registry
            .find(StaticMesh::NAME)
            .ok_or(RegistryError::ComponentNotFound)?;
        self.canvas = registry
            .find(Canvas::NAME)
            .ok_or(RegistryError::ComponentNotFound)?;
        self.local_to_world = registry
            .find(LocalToWorld::NAME)
            .ok_or(RegistryError::ComponentNotFound)?;
        self.viewport = registry
            .find(Viewport::NAME)
            .ok_or(RegistryError::ComponentNotFound)?;
        self.model = registry
            .find(Model::NAME)
            .ok_or(RegistryError::ComponentNotFound)?;
        Ok(())
    }

    pub(crate) fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                RendererEvent::Statistics(statistics) => self.statistics = statistics,
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

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // TODO: reset ECS resources
        self.graphics = Graphics::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub(crate) fn submit_graphics(
        &mut self,
        asset: &mut AssetManager,
        containers: &ContainerTable,
    ) {
        // Acquire active scene
        let viewports = containers
            .view(self.viewport)
            .expect("Failed to acquire viewport view");
        // Render main screen
        self.graphics.submit_provider(
            None,
            Color::TRANSPARENT,
            &mut self.resources,
            asset,
            &viewports,
            self.provider.as_mut(),
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
