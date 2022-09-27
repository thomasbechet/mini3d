use mini3d::{graphics::{SCREEN_PIXEL_COUNT, rasterizer::{Plotable, self}, SCREEN_WIDTH, SCREEN_HEIGHT, SCREEN_VIEWPORT, CommandBuffer}, glam::UVec2, application::Application, asset::{AssetDatabase, font::Font}};
use wgpu::TextureViewDescriptor;

use crate::context::WGPUContext;

#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub(crate) struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub(crate) const WHITE: Color = Self::from_rgba(255, 255, 255, 255);
    pub(crate) const BLACK: Color = Self::from_rgba(0, 0, 0, 255);
    pub(crate) const fn from_rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a, }
    }
    pub(crate) const fn from_color_alpha(color: Color, a: u8) -> Self {
        Self::from_rgba(color.r, color.g, color.b, a)
    }
}

pub(crate) struct SurfaceBuffer {
    buffer: Box<[Color]>,
    texture: wgpu::Texture,
    pub(crate) texture_view: wgpu::TextureView,
}

impl SurfaceBuffer {
    
    pub(crate) fn extent() -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            depth_or_array_layers: 1
        }
    }

    pub(crate) fn new(
        context: &WGPUContext,
    ) -> SurfaceBuffer {
        let buffer = vec![Color::default(); SCREEN_PIXEL_COUNT].into_boxed_slice();
        let texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: SurfaceBuffer::extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("surface_texture"),
        });
        let texture_view = texture.create_view(&TextureViewDescriptor::default());
        SurfaceBuffer {
            buffer,
            texture,
            texture_view, 
        }
    }

    pub(crate) fn write_texture(&self, context: &WGPUContext) {
        context.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytemuck::cast_slice(&self.buffer),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * SCREEN_WIDTH as u32),
                rows_per_image: std::num::NonZeroU32::new(SCREEN_HEIGHT as u32),
            },
            wgpu::Extent3d {
                width: SCREEN_WIDTH,
                height: SCREEN_HEIGHT,
                depth_or_array_layers: 1
            },
        );
    }

    pub(crate) fn clear(&mut self, color: Color) {
        self.buffer.fill(color);
    }

    pub(crate) fn draw_command_buffer(
        &mut self,
        app: &Application,
        cb: &CommandBuffer,
    ) {
        for command in cb.iter() {
            match command {
                mini3d::graphics::command_buffer::Command::Print {
                    p,
                    text,
                    font_id,
                } => {
                    rasterizer::print(
                        self,
                        *p,
                        text.as_str(),
                        &AssetDatabase::read::<Font>(app, *font_id).expect("Invalid font id").data,
                    );
                }
                mini3d::graphics::command_buffer::Command::DrawLine { p0, p1 } => {
                    rasterizer::draw_line(self, *p0, *p1);
                }
                mini3d::graphics::command_buffer::Command::DrawVLine { x, y0, y1 } => {
                    rasterizer::draw_vline(self, *x, *y0, *y1);
                }
                mini3d::graphics::command_buffer::Command::DrawHLine { y, x0, x1 } => {
                    rasterizer::draw_hline(self, *y, *x0, *x1);
                }
                mini3d::graphics::command_buffer::Command::DrawRect { rect } => {
                    let mut rect = *rect;
                    rect.clamp(&SCREEN_VIEWPORT);
                    rasterizer::draw_rect(self, rect);
                }
                mini3d::graphics::command_buffer::Command::FillRect { rect } => {
                    let mut rect = *rect;
                    rect.clamp(&SCREEN_VIEWPORT);
                    rasterizer::fill_rect(self, rect);
                }
            }   
        }
    }
}

impl Plotable for SurfaceBuffer {
    fn plot(&mut self, p: UVec2) {
        let index = p.y as usize * SCREEN_WIDTH as usize + p.x as usize;
        self.buffer[index] = Color::WHITE;
    }
}