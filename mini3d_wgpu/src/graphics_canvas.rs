use mini3d_core::glam::UVec2;

use crate::{
    context::WGPUContext,
    graphics_render_pass::GraphicsRenderPass,
    graphics_renderer::{GPUGlobalData, GraphicsRenderer},
};

pub(crate) const CANVAS_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const CANVAS_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

fn create_color_view(context: &WGPUContext, extent: &wgpu::Extent3d) -> wgpu::TextureView {
    let color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CANVAS_COLOR_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("canvas_color_texture"),
    });
    color_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) fn create_depth_view(
    context: &WGPUContext,
    extent: &wgpu::Extent3d,
) -> wgpu::TextureView {
    let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: CANVAS_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("canvas_depth_texture"),
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) struct GraphicsCanvas {
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) render_pass: GraphicsRenderPass,
    pub(crate) global_buffer: wgpu::Buffer,
    pub(crate) global_bind_group: wgpu::BindGroup,
}

impl GraphicsCanvas {
    fn write_global_buffer(&self, context: &WGPUContext) {
        let global_data = GPUGlobalData {
            resolution: [self.extent.width, self.extent.height],
        };
        context
            .queue
            .write_buffer(&self.global_buffer, 0, bytemuck::bytes_of(&global_data));
    }

    pub(crate) fn new(
        context: &WGPUContext,
        graphics_renderer: &GraphicsRenderer,
        resolution: UVec2,
    ) -> Self {
        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };
        let global_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("global_buffer"),
            size: std::mem::size_of::<GPUGlobalData>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let global_bind_group = graphics_renderer.create_global_bind_group(context, &global_buffer);
        let canvas = Self {
            extent,
            color_view: create_color_view(context, &extent),
            depth_view: create_depth_view(context, &extent),
            render_pass: GraphicsRenderPass::new(context),
            global_buffer,
            global_bind_group,
        };
        canvas.write_global_buffer(context);
        canvas
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        self.extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };
        self.color_view = create_color_view(context, &self.extent);
        self.depth_view = create_depth_view(context, &self.extent);
        self.write_global_buffer(context);
    }
}
