use mini3d::glam::Mat4;

use crate::context::WGPUContext;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUViewportData {
    world_to_clip: [f32; 16],
    _pad: [u64; 24],
}

pub(crate) const MAX_VIEWPORT_COUNT: usize = 32; 

pub(crate) struct ViewportBuffer {
    pub(crate) buffer: wgpu::Buffer,
    transfer_buffer: GPUViewportData,
}

impl ViewportBuffer {
    pub(crate) fn new(context: &WGPUContext) -> Self {
        Self {
            buffer: context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("viewport_buffer"),
                size: ViewportBuffer::size() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            transfer_buffer: Default::default(),
        }
    }

    pub(crate) fn size() -> usize {
        std::mem::size_of::<GPUViewportData>()
    }

    pub(crate) fn set_world_to_clip(&mut self, mat: &Mat4) {
        self.transfer_buffer.world_to_clip = mat.to_cols_array();
    }

    pub(crate) fn write_buffer(&self, context: &WGPUContext) {
        context.queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&self.transfer_buffer));
    }
}

