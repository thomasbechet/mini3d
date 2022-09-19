use mini3d::graphics::{SCREEN_WIDTH, SCREEN_HEIGHT};

use crate::context::WGPUContext;

pub(crate) struct RenderTarget {
    pub(crate) render_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
}

impl RenderTarget {
    pub(crate) fn extent() -> wgpu::Extent3d {
        wgpu::Extent3d {
            width: SCREEN_WIDTH,
            height: SCREEN_HEIGHT,
            depth_or_array_layers: 1
        }
    }

    pub(crate) fn format() -> wgpu::TextureFormat {
        wgpu::TextureFormat::Rgba8Unorm
    }
 
    pub(crate) fn new(context: &WGPUContext) -> Self {
        let render_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: RenderTarget::extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: RenderTarget::format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("render_target_render_texture"),
        });
        let render_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: RenderTarget::extent(),
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("render_target_depth_texture"),
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        Self {
            render_view,
            depth_view,
        }
    }
}