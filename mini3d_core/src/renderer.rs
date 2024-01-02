use crate::ecs::component::ComponentError;
use crate::ecs::entity::Entity;
use crate::ecs::view::native::single::NativeSingleViewMut;
use crate::math::fixed::{FixedPoint, I32F16};
use crate::math::vec::{V2, V2U32};
use crate::serialize::{Decoder, DecoderError};
use crate::{
    math::rect::IRect,
    serialize::{Encoder, EncoderError},
};
use alloc::boxed::Box;
use alloc::vec::Vec;
use mini3d_derive::Serialize;

use self::component::{
    Font, Material, MaterialData, Mesh, MeshData, RenderTransform, Texture, TextureData,
};
use self::event::RendererEvent;
use self::provider::{ProviderMaterialInfo, RendererProviderHandle};
use self::{color::Color, provider::RendererProvider};

pub mod color;
pub mod component;
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

pub const SCREEN_PIXEL_COUNT: u32 = SCREEN_WIDTH * SCREEN_HEIGHT;
pub const SCREEN_RESOLUTION: V2U32 = V2::new(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_CENTER: V2U32 = V2::new(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
pub const SCREEN_VIEWPORT: IRect = IRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_ASPECT_RATIO: I32F16 =
    I32F16::from_int(SCREEN_WIDTH as i32).div(I32F16::from_int(SCREEN_HEIGHT as i32));
pub const SCREEN_INV_ASPECT_RATIO: I32F16 = I32F16::ONE.div(SCREEN_ASPECT_RATIO);

pub const TILE_SIZE: u32 = 8;
pub const TILE_HCOUNT: u32 = SCREEN_WIDTH / TILE_SIZE;
pub const TILE_VCOUNT: u32 = SCREEN_HEIGHT / TILE_SIZE;

#[derive(Default, Clone, Copy, Serialize)]
pub struct RendererStatistics {
    pub triangle_count: u32,
    pub draw_count: u32,
}

#[derive(Default, Clone, Copy, Serialize)]
pub struct RendererFeatures {
    forward: bool,
    deferred: bool,
    shadow: bool,
}

#[derive(Default)]
pub(crate) struct RendererViews {
    pub(crate) texture: NativeSingleViewMut<Texture>,
    pub(crate) material: NativeSingleViewMut<Material>,
    pub(crate) mesh: NativeSingleViewMut<Mesh>,
    pub(crate) font: NativeSingleViewMut<Font>,
    pub(crate) transform: NativeSingleViewMut<RenderTransform>,
}

#[derive(Default)]
pub struct RendererManager {
    pub(crate) provider: Box<dyn RendererProvider>,
    statistics: RendererStatistics,
    features: RendererFeatures,
    clear_color: Color,
    textures: Vec<(Entity, RendererProviderHandle)>,
    meshes: Vec<(Entity, RendererProviderHandle)>,
    materials: Vec<(Entity, RendererProviderHandle)>,
    transforms: Vec<(Entity, RendererProviderHandle)>,
    pub(crate) handles: RendererViews,
}

impl RendererManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn RendererProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }

    pub(crate) fn dispatch_events(&mut self) {
        while let Some(event) = self.provider.next_event() {
            match event {
                RendererEvent::Statistics(statistics) => self.statistics = statistics,
            }
        }
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // TODO: reset ECS resources
        Ok(())
    }

    pub(crate) fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub(crate) fn statistics(&self) -> RendererStatistics {
        self.statistics
    }

    pub(crate) fn features(&self) -> RendererFeatures {
        self.features
    }

    pub(crate) fn add_transform(
        &mut self,
        entity: Entity,
    ) -> Result<RendererProviderHandle, ComponentError> {
        // Add transform to provider
        let handle = self
            .provider
            .add_transform()
            .map_err(|_| ComponentError::ProviderError)?;
        // Register transform
        self.transforms.push((entity, handle));
        Ok(handle)
    }

    pub(crate) fn remove_transform(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), ComponentError> {
        // Remove transform from provider
        self.provider
            .remove_transform(handle)
            .map_err(|_| ComponentError::ProviderError)?;
        // Unregister transform
        self.transforms.retain(|(_, h)| *h != handle);
        Ok(())
    }

    pub(crate) fn add_font(
        &mut self,
        entity: Entity,
        data: &Font,
    ) -> Result<RendererProviderHandle, ComponentError> {
        // Add font to provider
        let handle = self
            .provider
            .add_font(data)
            .map_err(|_| ComponentError::ProviderError)?;
        // Register font
        self.fonts.push((entity, handle));
        Ok(handle)
    }

    pub(crate) fn add_texture(
        &mut self,
        entity: Entity,
        data: &TextureData,
    ) -> Result<RendererProviderHandle, ComponentError> {
        // Add texture to provider
        let handle = self
            .provider
            .add_texture(data)
            .map_err(|_| ComponentError::ProviderError)?;
        // Register texture
        self.textures.push((entity, handle));
        Ok(handle)
    }

    pub(crate) fn remove_texure(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), ComponentError> {
        // Remove texture from provider
        self.provider
            .remove_texture(handle)
            .map_err(|_| ComponentError::ProviderError)?;
        // Unregister texture
        self.textures.retain(|(_, h)| *h != handle);
        Ok(())
    }

    pub(crate) fn add_mesh(
        &mut self,
        entity: Entity,
        data: &MeshData,
    ) -> Result<RendererProviderHandle, ComponentError> {
        // Add mesh to provider
        let handle = self
            .provider
            .add_mesh(data)
            .map_err(|_| ComponentError::ProviderError)?;
        // Register mesh
        self.meshes.push((entity, handle));
        Ok(handle)
    }

    pub(crate) fn add_material(
        &mut self,
        entity: Entity,
        data: &MaterialData,
    ) -> Result<RendererProviderHandle, ComponentError> {
        // Resolve resources
        let diffuse = self
            .textures
            .iter()
            .find(|(e, _)| *e == data.tex0)
            .map(|(_, h)| *h)
            .ok_or(ComponentError::UnresolvedReference)?;
        // Add material to provider
        let handle = self
            .provider
            .add_material(ProviderMaterialInfo {
                diffuse: RendererProviderHandle::null(),
            })
            .map_err(|_| ComponentError::ProviderError)?;
        // Register material
        self.materials.push((entity, handle));
        Ok(handle)
    }

    pub(crate) fn remove_material(
        &mut self,
        handle: RendererProviderHandle,
    ) -> Result<(), ComponentError> {
        // Remove material from provider
        self.provider
            .remove_material(handle)
            .map_err(|_| ComponentError::ProviderError)?;
        // Unregister material
        self.materials.retain(|(_, h)| *h != handle);
        Ok(())
    }
}
