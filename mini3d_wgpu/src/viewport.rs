use mini3d::{glam::UVec2, renderer::backend::CameraHandle};

use crate::context::WGPUContext;

pub(crate) const VIEWPORT_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const VIEWPORT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub(crate) struct Viewport {
    pub(crate) camera: Option<CameraHandle>,
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
}

impl Viewport {

    pub(crate) fn new(context: &WGPUContext, resolution: UVec2) -> Self {

        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };
        
        let color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: VIEWPORT_COLOR_FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("render_target_color_texture"),
        });
        let color_view = color_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: VIEWPORT_DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("render_target_depth_texture"),
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
    
        Self {
            camera: None,
            extent,
            color_view,
            depth_view,
        }
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        let camera = self.camera;
        *self = Self::new(context, resolution);
        self.camera = camera;
    }

    pub(crate) fn aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }
}