use crate::ecs::container::ContainerTable;
use crate::feature::common::local_to_world::LocalToWorld;
use crate::feature::renderer::camera::Camera;
use crate::feature::renderer::canvas::Canvas;
use crate::feature::renderer::font::{Font, FontAtlas};
use crate::feature::renderer::material::Material;
use crate::feature::renderer::mesh::Mesh;
use crate::feature::renderer::model::Model;
use crate::feature::renderer::static_mesh::StaticMesh;
use crate::feature::renderer::texture::Texture;
use crate::feature::renderer::viewport::Viewport;
use crate::registry::component::StaticComponentType;
use crate::registry::error::RegistryError;
use crate::registry::resource::StaticResourceType;
use crate::registry::RegistryManager;
use crate::resource::handle::ResourceHandle;
use crate::resource::ResourceManager;
use crate::serialize::{Decoder, DecoderError, Serialize};
use crate::utils::slotmap::SecondaryMap;
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

#[derive(Default)]
pub(crate) struct RendererFont {
    pub(crate) atlas: FontAtlas,
    pub(crate) handle: TextureHandle,
}

#[derive(Default)]
pub(crate) struct RendererTexture {
    pub(crate) handle: TextureHandle,
}

#[derive(Default)]
pub(crate) struct RendererMesh {
    pub(crate) handle: MeshHandle,
}

#[derive(Default)]
pub(crate) struct RendererMaterial {
    pub(crate) handle: MaterialHandle,
}

#[derive(Default)]
pub(crate) struct RendererResourceManager {
    fonts: SecondaryMap<RendererFont>,
    textures: SecondaryMap<RendererTexture>,
    meshes: SecondaryMap<RendererMesh>,
    materials: SecondaryMap<RendererMaterial>,
}

fn load_font(
    handle: ResourceHandle,
    provider: &mut dyn RendererProvider,
    resource: &ResourceManager,
) -> Result<RendererFont, RendererProviderError> {
    let font = resource.read::<Font>(handle).unwrap();
    let atlas = FontAtlas::new(font);
    let handle = provider.texture_add(&atlas.texture)?;
    Ok(RendererFont { atlas, handle })
}

fn load_mesh(
    handle: ResourceHandle,
    provider: &mut dyn RendererProvider,
    resource: &ResourceManager,
) -> Result<RendererMesh, RendererProviderError> {
    let mesh = resource.read::<Mesh>(handle).unwrap();
    let handle = provider.mesh_add(mesh)?;
    Ok(RendererMesh { handle })
}

fn load_texture(
    handle: ResourceHandle,
    provider: &mut dyn RendererProvider,
    resource: &ResourceManager,
) -> Result<RendererTexture, RendererProviderError> {
    let texture = resource.read::<Texture>(handle).unwrap();
    let handle = provider.texture_add(texture)?;
    Ok(RendererTexture { handle })
}

fn load_material(
    handle: ResourceHandle,
    textures: &SecondaryMap<RendererTexture>,
    provider: &mut dyn RendererProvider,
    resource: &ResourceManager,
) -> Result<RendererMaterial, RendererProviderError> {
    let material = resource.read::<Material>(handle).unwrap();
    let info = resource.info(handle).unwrap();
    let diffuse = textures.get(material.diffuse.id).unwrap().handle;
    let handle = provider.material_add(ProviderMaterialDescriptor {
        diffuse,
        name: info.path,
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

    pub(crate) fn request_font(
        &mut self,
        handle: ResourceHandle,
        provider: &mut dyn RendererProvider,
        resource: &ResourceManager,
    ) -> Result<&RendererFont, RendererProviderError> {
        if self.fonts.contains(handle.id) {
            return Ok(self.fonts.get(handle.id).unwrap());
        }
        let font = load_font(handle, provider, resource)?;
        self.fonts.insert(handle.id, font);
        Ok(self.fonts.get(handle.id).unwrap())
    }

    pub(crate) fn request_mesh(
        &mut self,
        handle: ResourceHandle,
        provider: &mut dyn RendererProvider,
        resource: &ResourceManager,
    ) -> Result<&RendererMesh, RendererProviderError> {
        if self.meshes.contains(handle.id) {
            return Ok(self.meshes.get(handle.id).unwrap());
        }
        self.meshes
            .insert(handle.id, load_mesh(handle, provider, resource)?);
        Ok(self.meshes.get(handle.id).unwrap())
    }

    pub(crate) fn request_texture(
        &mut self,
        handle: ResourceHandle,
        provider: &mut dyn RendererProvider,
        resource: &ResourceManager,
    ) -> Result<&RendererTexture, RendererProviderError> {
        if self.textures.contains(handle.id) {
            return Ok(self.textures.get(handle.id).unwrap());
        }
        self.textures
            .insert(handle.id, load_texture(handle, provider, resource)?);
        Ok(self.textures.get(handle.id).unwrap())
    }

    pub(crate) fn request_material(
        &mut self,
        handle: ResourceHandle,
        provider: &mut dyn RendererProvider,
        resource: &ResourceManager,
    ) -> Result<&RendererMaterial, RendererProviderError> {
        if self.materials.contains(handle.id) {
            return Ok(self.materials.get(handle.id).unwrap());
        }
        let material = resource.read::<Material>(handle).unwrap();
        if !self.textures.contains(material.diffuse.id) {
            let diffuse = load_texture(material.diffuse, provider, resource)?;
            self.textures.insert(material.diffuse.id, diffuse);
        }
        self.materials.insert(
            handle.id,
            load_material(handle, &self.textures, provider, resource)?,
        );
        Ok(self.materials.get(handle.id).unwrap())
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
    camera: StaticComponentType<Camera>,
    static_mesh: StaticComponentType<StaticMesh>,
    canvas: StaticComponentType<Canvas>,
    local_to_world: StaticComponentType<LocalToWorld>,
    viewport: StaticComponentType<Viewport>,
    model: StaticResourceType<Model>,
}

impl RendererManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn RendererProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    pub(crate) fn reload_components_and_resources(
        &mut self,
        registry: &RegistryManager,
    ) -> Result<(), RegistryError> {
        self.camera = registry.component.find(Camera::NAME).unwrap_or_default();
        self.static_mesh = registry
            .component
            .find(StaticMesh::NAME)
            .unwrap_or_default();
        self.canvas = registry.component.find(Canvas::NAME).unwrap_or_default();
        self.local_to_world = registry
            .component
            .find(LocalToWorld::NAME)
            .unwrap_or_default();
        self.viewport = registry.component.find(Viewport::NAME).unwrap_or_default();
        self.model = registry.resource.find(Model::NAME).unwrap_or_default();
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
        resource: &mut ResourceManager,
        containers: &ContainerTable,
    ) -> Result<(), RendererProviderError> {
        // Acquire active scene
        let viewports = containers
            .view(self.viewport)
            .expect("Failed to acquire viewport view");
        // Render main screen
        self.graphics.submit_provider(
            None,
            Color::TRANSPARENT,
            &mut self.resources,
            resource,
            &viewports,
            self.provider.as_mut(),
        )
    }

    pub(crate) fn graphics(&mut self) -> &mut Graphics {
        &mut self.graphics
    }

    pub(crate) fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub(crate) fn statistics(&self) -> RendererStatistics {
        self.statistics
    }
}
