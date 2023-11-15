use glam::{IVec2, Mat4, Vec2, Vec3, Vec4};
use mini3d_derive::Error;

use crate::{
    define_resource_handle,
    feature::renderer::texture::{TextureHandle, TextureWrapMode},
    math::rect::IRect,
};

use super::{color::Color, pipeline::GraphicsPipelineHandle};

pub struct PipelineLayout {}

pub struct GraphicsQueueHandle(pub(crate) u32);
pub struct ComputeQueueHandle(pub(crate) u32);
pub struct CanvasQueueHandle(pub(crate) u32);
pub struct CopyQueueHandle(pub(crate) u32);

pub enum ShaderResource {
    Buffer {
        handle: RenderBufferHandle,
        offset: usize,
    },
    Texture {
        handle: TextureHandle,
    },
}

#[derive(Default)]
pub(crate) struct DrawCommand {
    pub pipeline: GraphicsPipelineHandle,
    pub vertex_buffer: RenderBufferHandle,
    pub first: u16,
    pub count: u16,
    pub bind_group: BindGroupHandle,
    pub variable_start: u16,
    pub variable_count: u16,
    pub key: u32,
}

#[derive(Error)]
pub enum DrawCommandError {
    #[error("Invalid pipeline")]
    InvalidPipeline,
    #[error("Invalid vertex buffer")]
    InvalidVertexBuffer,
    #[error("Invalid bind group")]
    InvalidBindGroup,
    #[error("Invalid key")]
    InvalidKey,
}

impl DrawCommand {
    pub(crate) fn reset(&mut self, variable_start: usize) {
        *self = Default::default();
        self.variable_start = variable_start as u16;
    }

    pub(crate) fn validate(&mut self) -> Result<(), DrawCommandError> {
        if self.pipeline.is_null() {
            return Err(DrawCommandError::InvalidPipeline);
        }
        if self.vertex_buffer.is_null() {
            return Err(DrawCommandError::InvalidVertexBuffer);
        }
        if self.bind_group.is_null() {
            return Err(DrawCommandError::InvalidBindGroup);
        }
        Ok(())
    }
}

pub(crate) struct DrawInstancedCommand {}

pub(crate) enum GraphicsCommand {
    Draw(DrawCommand),
    DrawInstanced(DrawInstancedCommand),
}

pub struct GraphicsQueue {
    pub(crate) draw: DrawCommand,
    pub(crate) draw_instanced: DrawInstancedCommand,
    pub(crate) commands: Vec<GraphicsCommand>,
    pub(crate) push_constants: Vec<PushConstant>,
}

pub struct CanvasQueue {}

impl CanvasQueue {}

pub struct ComputeQueue {}

impl ComputeQueue {
    pub fn dispatch(&mut self, x: u32, y: u32, z: u32) {}
}

pub struct CopyQueue {}

impl CopyQueue {}
