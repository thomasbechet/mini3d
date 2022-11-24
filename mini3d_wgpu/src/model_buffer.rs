use std::mem;

use mini3d::glam::Mat4;

use crate::context::WGPUContext;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUModelData {
    // Object transform matrix
    transform: [f32; 16],
}

impl GPUModelData {
    pub(crate) fn size() -> usize {
        mem::size_of::<Self>()
    }
}

pub(crate) type ModelIndex = usize;

pub(crate) struct ModelBuffer {
    pub(crate) buffer: wgpu::Buffer,
    local: Box<[GPUModelData]>,
    model_count: usize,
    free_model_indices: Vec<ModelIndex>,
}

impl ModelBuffer {
    
    pub(crate) fn new(context: &WGPUContext, max_model_count: usize) -> Self {
        Self {
            buffer: context.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("model_buffer"),
                size: (GPUModelData::size() * max_model_count) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            local: vec![GPUModelData::default(); max_model_count].into_boxed_slice(),
            model_count: 0,
            free_model_indices: Default::default(),
        }
    }

    pub(crate) fn add(&mut self) -> ModelIndex {
        // Find free index or create a new one
        match self.free_model_indices.pop() {
            Some(index) => index,
            None => {
                let index: ModelIndex = self.model_count;
                self.model_count += 1;
                index
            },
        }
    }

    pub(crate) fn remove(&mut self, index: ModelIndex) {
        self.free_model_indices.push(index);
    }

    pub(crate) fn set_transform(&mut self, index: ModelIndex, mat: &Mat4) {
        self.local[index].transform = mat.to_cols_array();
    }

    pub(crate) fn write_buffer(&self, context: &WGPUContext) {
        context.queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&self.local));
    }
}

