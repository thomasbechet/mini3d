use mini3d_core::{glam::UVec2, renderer::provider::SceneCameraProviderHandle};

use crate::context::WGPUContext;

pub(crate) const VIEWPORT_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const VIEWPORT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

fn create_color_view(context: &WGPUContext, extent: &wgpu::Extent3d) -> wgpu::TextureView {
    let color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: VIEWPORT_COLOR_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("viewport_color_texture"),
    });
    color_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_depth_view(context: &WGPUContext, extent: &wgpu::Extent3d) -> wgpu::TextureView {
    let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: VIEWPORT_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("viewport_depth_texture"),
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) struct Viewport {
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) camera: Option<SceneCameraProviderHandle>,
}

impl Viewport {
    pub(crate) fn new(context: &WGPUContext, resolution: UVec2) -> Self {
        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };

        Self {
            extent,
            color_view: create_color_view(context, &extent),
            depth_view: create_depth_view(context, &extent),
            camera: None,
        }
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        self.extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };
        self.color_view = create_color_view(context, &self.extent);
        self.depth_view = create_depth_view(context, &self.extent);
    }

    pub(crate) fn aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }
}
