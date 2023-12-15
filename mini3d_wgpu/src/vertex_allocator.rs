use mini3d_core::feature::renderer;

use crate::{context::WGPUContext, error::WGPURendererError};

#[derive(Clone, Copy)]
pub(crate) struct VertexBufferDescriptor {
    pub(crate) vertex_count: u32,
    pub(crate) base_index: u32,
}

pub(crate) struct VertexAllocator {
    pub(crate) position_buffer: wgpu::Buffer,
    pub(crate) normal_buffer: wgpu::Buffer,
    pub(crate) uv_buffer: wgpu::Buffer,
    max_vertex_count: usize,
    vertex_count: usize,
}

impl VertexAllocator {
    pub fn new(context: &WGPUContext, max_vertex_count: usize) -> Self {
        // Create buffers
        let position_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: wgpu::VertexFormat::Float32x3.size() * max_vertex_count as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let normal_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: wgpu::VertexFormat::Float32x3.size() * max_vertex_count as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uv_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: wgpu::VertexFormat::Float32x2.size() * max_vertex_count as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // Create vertex buffer
        Self {
            position_buffer,
            normal_buffer,
            uv_buffer,
            max_vertex_count,
            vertex_count: 0,
        }
    }

    pub fn add(
        &mut self,
        context: &WGPUContext,
        vertices: &Vec<renderer::mesh::Vertex>,
    ) -> Result<VertexBufferDescriptor, WGPURendererError> {
        // Create the vertex descriptor
        let descriptor = VertexBufferDescriptor {
            vertex_count: vertices.len() as u32,
            base_index: self.vertex_count as u32,
        };

        // Check vertex count
        if self.vertex_count + (descriptor.vertex_count as usize) > self.max_vertex_count {
            return Err(WGPURendererError::MaxVertexCountReached);
        }

        // Increment vertex count
        self.vertex_count += descriptor.vertex_count as usize;

        // Convert vertices
        let positions: &[f32] = &vertices
            .iter()
            .map(|v| v.position.to_array())
            .collect::<Vec<[f32; 3]>>()
            .concat();
        let normals: &[f32] = &vertices
            .iter()
            .map(|v| v.normal.to_array())
            .collect::<Vec<[f32; 3]>>()
            .concat();
        let uvs: &[f32] = &vertices
            .iter()
            .map(|v| v.uv.to_array())
            .collect::<Vec<[f32; 2]>>()
            .concat();

        // Write buffers
        context.queue.write_buffer(
            &self.position_buffer,
            wgpu::VertexFormat::Float32x3.size() * descriptor.base_index as u64,
            bytemuck::cast_slice(positions),
        );
        context.queue.write_buffer(
            &self.normal_buffer,
            wgpu::VertexFormat::Float32x3.size() * descriptor.base_index as u64,
            bytemuck::cast_slice(normals),
        );
        context.queue.write_buffer(
            &self.uv_buffer,
            wgpu::VertexFormat::Float32x2.size() * descriptor.base_index as u64,
            bytemuck::cast_slice(uvs),
        );

        // Return descriptor
        Ok(descriptor)
    }

    pub fn clear(&mut self) {
        self.vertex_count = 0;
    }
}
