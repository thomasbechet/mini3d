use crate::math::fixed::{FixedPoint, I32F16};
use crate::math::vec::{V2, V2U32};
use crate::resource::ResourceTypeHandle;
use crate::serialize::{Decoder, DecoderError};
use crate::{
    math::rect::IRect,
    serialize::{Encoder, EncoderError},
};
use alloc::boxed::Box;
use mini3d_derive::Serialize;

use self::event::RendererEvent;
use self::resource::{Font, FontHandle, Mesh, MeshHandle, Texture, TextureHandle};
use self::{color::Color, provider::RendererProvider};

pub mod color;
pub mod component;
pub mod event;
pub mod graphics;
pub mod provider;
pub mod rasterizer;
pub mod resource;

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
pub(crate) struct RendererHandles {
    pub(crate) texture: ResourceTypeHandle,
    pub(crate) material: ResourceTypeHandle,
    pub(crate) mesh: ResourceTypeHandle,
    pub(crate) font: ResourceTypeHandle,
    pub(crate) graph: ResourceTypeHandle,
    pub(crate) model: ResourceTypeHandle,
    pub(crate) transform: ResourceTypeHandle,
    pub(crate) renderpass: ResourceTypeHandle,
}

#[derive(Default)]
pub struct RendererManager {
    pub(crate) provider: Box<dyn RendererProvider>,
    statistics: RendererStatistics,
    features: RendererFeatures,
    clear_color: Color,
    pub(crate) handles: RendererHandles,
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

    pub(crate) fn on_texture_added_hook(&mut self, texture: &mut Texture, handle: TextureHandle) {}
    pub(crate) fn on_texture_removed_hook(&mut self, texture: &mut Texture, handle: TextureHandle) {
    }
    pub(crate) fn on_mesh_added_hook(&mut self, mesh: &mut Mesh, handle: MeshHandle) {}
    pub(crate) fn on_mesh_removed_hook(&mut self, mesh: &mut Mesh, handle: MeshHandle) {}
    pub(crate) fn on_font_added_hook(&mut self, font: &mut Font, handle: FontHandle) {}
    pub(crate) fn on_font_removed_hook(&mut self, font: &mut Font, handle: FontHandle) {}
}
