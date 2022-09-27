use mini3d::glam::Mat4;

use crate::context::WGPUContext;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUGlobalData {
    world_to_clip: [f32; 16],
}

pub(crate) struct GlobalBuffer {
    pub(crate) buffer: wgpu::Buffer,
    local: GPUGlobalData,
}

impl GlobalBuffer {
    pub(crate) fn new(context: &WGPUContext) -> Self {
        Self {
            buffer: context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("global_buffer"),
                size: GlobalBuffer::size() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            local: Default::default(),
        }
    }

    pub(crate) fn size() -> usize {
        std::mem::size_of::<GPUGlobalData>()
    }

    pub(crate) fn set_world_to_clip(&mut self, mat: &Mat4) {
        self.local.world_to_clip = mat.to_cols_array();
    }

    pub(crate) fn write_buffer(&self, context: &WGPUContext) {
        context.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.local));
    }
}

