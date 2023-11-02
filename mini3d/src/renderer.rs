use crate::ecs::container::ContainerTable;
use crate::resource::ResourceManager;
use crate::serialize::{Decoder, DecoderError, Serialize};
use crate::{
    math::rect::IRect,
    serialize::{Encoder, EncoderError},
};
use glam::{uvec2, UVec2};
use mini3d_derive::Serialize;

use self::event::RendererEvent;
use self::resource::RendererResources;
use self::{
    color::Color,
    provider::{RendererProvider, RendererProviderError},
};

pub mod color;
pub mod command;
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

pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;
pub const SCREEN_RESOLUTION: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_CENTER: UVec2 = uvec2(SCREEN_WIDTH / 2, SCREEN_HEIGHT / 2);
pub const SCREEN_VIEWPORT: IRect = IRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;
pub const SCREEN_INV_ASPECT_RATIO: f32 = 1.0 / SCREEN_ASPECT_RATIO;

pub const TILE_SIZE: u32 = 8;
pub const TILE_HCOUNT: u32 = SCREEN_WIDTH / TILE_SIZE;
pub const TILE_VCOUNT: u32 = SCREEN_HEIGHT / TILE_SIZE;

#[derive(Default, Clone, Copy, Serialize)]
pub struct RendererStatistics {
    pub triangle_count: usize,
    pub draw_count: usize,
}

#[derive(Default)]
pub struct RendererManager {
    pub(crate) provider: Box<dyn RendererProvider>,

    // Persistent data
    statistics: RendererStatistics,
    clear_color: Color,
    resources: RendererResources,
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

    pub(crate) fn prepare(&mut self) {}

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // TODO: reset ECS resources
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
            &mut self.types,
            resource,
            &viewports,
            self.provider.as_mut(),
        )
    }

    pub(crate) fn set_clear_color(&mut self, color: Color) {
        self.clear_color = color;
    }

    pub(crate) fn statistics(&self) -> RendererStatistics {
        self.statistics
    }

    pub(crate) fn on_texture_added_hook(&mut self, texture: &mut Texture, handle: TextureHandle) {}
    pub(crate) fn on_texture_removed_hook(&mut self, texture: &mut Texture, handle: FontHandle) {}
    pub(crate) fn on_mesh_added_hook(&mut self, mesh: &mut Mesh, handle: FontHandle) {}
    pub(crate) fn on_mesh_removed_hook(&mut self, mesh: &mut Mesh, handle: FontHandle) {}
    pub(crate) fn on_font_added_hook(&mut self, font: &mut Font, handle: FontHandle) {}
    pub(crate) fn on_font_removed_hook(&mut self, font: &mut Font, handle: FontHandle) {}
}
