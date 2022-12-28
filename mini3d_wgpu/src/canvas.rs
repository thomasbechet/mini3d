use mini3d::{glam::{UVec2, IVec2}, renderer::backend::CanvasHandle, anyhow::Result};

use crate::{context::WGPUContext, blit_bind_group::create_blit_bind_group, canvas_renderer::{CanvasRenderPass, GPUBlitData, GPUPrimitiveVertexData}};

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

fn create_depth_view(context: &WGPUContext, extent: &wgpu::Extent3d) -> wgpu::TextureView {
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

pub(crate) struct Canvas {
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) render_pass: CanvasRenderPass,
    pub(crate) blit_buffer: wgpu::Buffer,
    pub(crate) primitive_buffer: wgpu::Buffer,
}

fn create_vertex_buffer<T>(
    context: &WGPUContext,
    vertex_count: usize,
) -> wgpu::Buffer {
    context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (std::mem::size_of::<T>() * vertex_count) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}


impl Canvas {

    pub(crate) fn new(context: &WGPUContext, resolution: UVec2) -> Self {

        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };
    
        Self {
            extent,
            color_view: create_color_view(context, &extent),
            depth_view: create_depth_view(context, &extent),
            render_pass: Default::default(),
            blit_buffer: create_vertex_buffer::<GPUBlitData>(context, 512),
            primitive_buffer: create_vertex_buffer::<GPUPrimitiveVertexData>(context, 512),
        }
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        self.extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };
        self.color_view = create_color_view(context, &self.extent);
        self.depth_view = create_depth_view(context, &self.extent);
    }

    pub(crate) fn write_transfer_buffers(&mut self, context: &WGPUContext) -> Result<()> {
        if self.blit_buffer.size() < (std::mem::size_of::<GPUBlitData>() * self.render_pass.blit_transfer.len()) as u64 {
            self.blit_buffer = create_vertex_buffer::<GPUBlitData>(context, self.render_pass.blit_transfer.len() * 2);
        }
        if self.primitive_buffer.size() < (std::mem::size_of::<GPUPrimitiveVertexData>() * self.render_pass.primitive_transfer.len()) as u64 {
            self.primitive_buffer = create_vertex_buffer::<GPUPrimitiveVertexData>(context, self.render_pass.primitive_transfer.len() * 2);
        }
        context.queue.write_buffer(&self.blit_buffer, 0, bytemuck::cast_slice(&self.render_pass.blit_transfer));
        context.queue.write_buffer(&self.primitive_buffer, 0, bytemuck::cast_slice(&self.render_pass.primitive_transfer));
        Ok(())
    }
}

pub(crate) struct SurfaceCanvas {
    pub(crate) canvas: CanvasHandle,
    pub(crate) position: IVec2,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) z_index: i32,
}

impl SurfaceCanvas {
    
    pub(crate) fn new(
        context: &WGPUContext, 
        position: IVec2,
        blit_bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        canvas_handle: CanvasHandle, 
        canvas: &Canvas,
        z_index: i32,
    ) -> Self {
        Self {
            canvas: canvas_handle,
            position,
            bind_group: create_blit_bind_group(context, blit_bind_group_layout, 
                &canvas.color_view, sampler, Some("canvas_blit_bind_group")),
            z_index,
        }
    }

    pub(crate) fn recreate(
        &mut self, 
        context: &WGPUContext,
        blit_bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler, 
        canvas: &Canvas,
    ) {
        self.bind_group = create_blit_bind_group(context, blit_bind_group_layout,
            &canvas.color_view, sampler, Some("canvas_blit_bind_group"))
    }
}