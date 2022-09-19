use mini3d::asset;
use wgpu::util::DeviceExt;

use crate::context::WGPUContext;

pub(crate) struct VertexBuffer {
    pub(crate) vertex_count: usize,
    pub(crate) position_buffer: wgpu::Buffer,
    pub(crate) normal_buffer: wgpu::Buffer,
    pub(crate) uv_buffer: wgpu::Buffer,
}

impl VertexBuffer {
    pub fn from_vertices(
        context: &WGPUContext,
        vertices: &Vec<asset::mesh::Vertex>,
        label: Option<&str>,
    ) -> Self {

        // Collect vertex data
        let positions: &[f32] = &vertices.iter().map(|v| v.position.to_array()).collect::<Vec<[f32; 3]>>().concat();
        let normals: &[f32] = &vertices.iter().map(|v| v.normal.to_array()).collect::<Vec<[f32; 3]>>().concat();
        let uvs: &[f32] = &vertices.iter().map(|v| v.uv.to_array()).collect::<Vec<[f32; 2]>>().concat();
        
        // Create buffers
        let position_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(positions),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let normal_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(normals),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let uv_buffer = context.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label,
            contents: bytemuck::cast_slice(uvs),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_count: vertices.len(),
            position_buffer,
            normal_buffer,
            uv_buffer,
        }
    }
}