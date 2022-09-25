use std::mem;

use mini3d::glam::Mat4;

use crate::context::WGPUContext;

const MAX_INSTANCE_COUNT: usize = 256;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUInstanceData {
    model: [f32; 16],
}

impl GPUInstanceData {
    pub(crate) fn size() -> usize {
        mem::size_of::<Self>()
    }
}

pub(crate) struct InstanceUniformBuffer {
    pub(crate) buffer: wgpu::Buffer,
    transfer: [GPUInstanceData; MAX_INSTANCE_COUNT],
}

pub(crate) type InstanceIndex = usize;

impl InstanceUniformBuffer {
    
    pub(crate) fn new(context: &WGPUContext) -> Self {
        Self {
            buffer: context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("instance_uniform_buffer"),
                size: (GPUInstanceData::size() * MAX_INSTANCE_COUNT) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            transfer: [GPUInstanceData::default(); MAX_INSTANCE_COUNT],
        }
    }

    pub(crate) fn set_model(&mut self, instance: InstanceIndex, mat: &Mat4) {
        self.transfer[instance].model = mat.to_cols_array();
    }

    pub(crate) fn write_buffer(&self, context: &WGPUContext) {
        context.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.transfer));
    }
}

