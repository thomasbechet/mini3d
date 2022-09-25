use mini3d::asset;

use crate::context::WGPUContext;

#[derive(Clone, Copy)]
pub(crate) struct VertexBufferDescriptor {
    pub(crate) vertex_count: u64,
    pub(crate) start_index: u64,
}

pub(crate) struct VertexBuffer {
    position_buffer: wgpu::Buffer,
    normal_buffer: wgpu::Buffer,
    uv_buffer: wgpu::Buffer,
    max_vertex_count: u64,
    vertex_count: u64,
}

impl VertexBuffer {
    pub fn new(
        context: &WGPUContext,
        max_vertex_count: u64,
    ) -> Self {
        // Create buffers
        let position_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: wgpu::VertexFormat::Float32x3.size() * max_vertex_count,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let normal_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: wgpu::VertexFormat::Float32x3.size() * max_vertex_count,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uv_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: wgpu::VertexFormat::Float32x2.size() * max_vertex_count,
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

    pub fn test(&mut self) {}

    pub fn add(
        &mut self,
        context: &WGPUContext,
        vertices: &Vec<asset::mesh::Vertex>,
    ) -> Option<VertexBufferDescriptor> {

        // Create the vertex descriptor
        let descriptor = VertexBufferDescriptor { 
            vertex_count: vertices.len() as u64, 
            start_index: self.vertex_count, 
        };
        self.vertex_count += descriptor.vertex_count;

        // Convert vertices
        let positions: &[f32] = &vertices.iter().map(|v| v.position.to_array()).collect::<Vec<[f32; 3]>>().concat();
        let normals: &[f32] = &vertices.iter().map(|v| v.normal.to_array()).collect::<Vec<[f32; 3]>>().concat();
        let uvs: &[f32] = &vertices.iter().map(|v| v.uv.to_array()).collect::<Vec<[f32; 2]>>().concat();
    
        // Write buffers
        context.queue.write_buffer(
            &self.position_buffer, 
            wgpu::VertexFormat::Float32x3.size() * descriptor.start_index as u64, 
            bytemuck::cast_slice(positions)
        );
        context.queue.write_buffer(
            &self.normal_buffer, 
            wgpu::VertexFormat::Float32x3.size() * descriptor.start_index as u64, 
            bytemuck::cast_slice(normals)
        );
        context.queue.write_buffer(
            &self.uv_buffer, 
            wgpu::VertexFormat::Float32x2.size() * descriptor.start_index as u64, 
            bytemuck::cast_slice(uvs)
        );

        // Return descriptor
        Some(descriptor)
    }

    pub(crate) fn position_slice(
        &self,
        descriptor: &VertexBufferDescriptor,
    ) -> wgpu::BufferSlice {
        let stride = wgpu::VertexFormat::Float32x3.size();
        let start = descriptor.start_index * stride;
        let end = start + descriptor.vertex_count * stride;
        self.position_buffer.slice(start..end)
    }

    pub(crate) fn normal_slice(
        &self,
        descriptor: &VertexBufferDescriptor,
    ) -> wgpu::BufferSlice {
        let stride = wgpu::VertexFormat::Float32x3.size();
        let start = descriptor.start_index * stride;
        let end = start + descriptor.vertex_count * stride;
        self.normal_buffer.slice(start..end)
    }

    pub(crate) fn uv_slice(
        &self,
        descriptor: &VertexBufferDescriptor,
    ) -> wgpu::BufferSlice {
        let stride = wgpu::VertexFormat::Float32x2.size();
        let start = descriptor.start_index * stride;
        let end = start + descriptor.vertex_count * stride;
        self.uv_buffer.slice(start..end)
    }
}