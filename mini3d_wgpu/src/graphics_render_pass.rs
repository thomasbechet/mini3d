use mini3d::{
    glam::{IVec2, UVec2},
    math::rect::IRect,
    renderer::{
        color::Color,
        graphics::TextureWrapMode,
        provider::{TextureHandle, ViewportHandle},
        rasterizer,
    },
};

use crate::context::WGPUContext;

const MAX_DEPTH: f32 = 1000.0;
const MIN_DEPTH: f32 = -1000.0;
const DEPTH_INCREMENT: f32 = 0.05;

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUBlitData {
    pos: [i16; 2],
    tex: [u16; 2],
    size: [u16; 2],
    depth: f32,
    color: [f32; 3],
    threshold: f32,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUPrimitiveVertexData {
    pos: [i32; 2],
    depth: f32,
    color: [f32; 4],
}

#[derive(Debug)]
pub(crate) struct BlitBatch {
    pub(crate) texture: TextureHandle,
    pub(crate) instance_start: u32,
    pub(crate) instance_count: u32,
}

#[derive(Debug)]
pub(crate) struct ViewportBatch {
    pub(crate) viewport: ViewportHandle,
    pub(crate) blit_index: u32,
}

#[derive(Debug)]
pub(crate) struct PrimitiveBatch {
    pub(crate) vertex_start: u32,
    pub(crate) vertex_count: u32,
}

pub(crate) enum GraphicsCommand {
    Blit(BlitBatch),
    Viewport(ViewportBatch),
    Triangles(PrimitiveBatch),
    Lines(PrimitiveBatch),
    Points(PrimitiveBatch),
    Scissor(IRect),
}

fn create_vertex_buffer<T>(context: &WGPUContext, vertex_count: usize) -> wgpu::Buffer {
    context.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: (std::mem::size_of::<T>() * vertex_count) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

pub(crate) struct GraphicsRenderPass {
    pub(crate) commands: Vec<GraphicsCommand>,
    pub(crate) clear_color: wgpu::Color,
    pub(crate) blit_transfer: Vec<GPUBlitData>,
    pub(crate) primitive_transfer: Vec<GPUPrimitiveVertexData>,
    pub(crate) blit_buffer: wgpu::Buffer,
    pub(crate) primitive_buffer: wgpu::Buffer,
    depth: f32,
}

impl GraphicsRenderPass {
    pub(crate) fn new(context: &WGPUContext) -> Self {
        Self {
            commands: Default::default(),
            clear_color: wgpu::Color::TRANSPARENT,
            blit_transfer: Default::default(),
            primitive_transfer: Default::default(),
            blit_buffer: create_vertex_buffer::<GPUBlitData>(context, 512),
            primitive_buffer: create_vertex_buffer::<GPUPrimitiveVertexData>(context, 512),
            depth: 0.0,
        }
    }

    pub(crate) fn write_buffers(&mut self, context: &WGPUContext) {
        if self.blit_buffer.size()
            < (std::mem::size_of::<GPUBlitData>() * self.blit_transfer.len()) as u64
        {
            self.blit_buffer =
                create_vertex_buffer::<GPUBlitData>(context, self.blit_transfer.len() * 2);
        }
        if self.primitive_buffer.size()
            < (std::mem::size_of::<GPUPrimitiveVertexData>() * self.primitive_transfer.len()) as u64
        {
            self.primitive_buffer = create_vertex_buffer::<GPUPrimitiveVertexData>(
                context,
                self.primitive_transfer.len() * 2,
            );
        }
        context.queue.write_buffer(
            &self.blit_buffer,
            0,
            bytemuck::cast_slice(&self.blit_transfer),
        );
        context.queue.write_buffer(
            &self.primitive_buffer,
            0,
            bytemuck::cast_slice(&self.primitive_transfer),
        );
    }

    pub(crate) fn begin(&mut self, clear_color: Color) {
        self.commands.clear();
        self.blit_transfer.clear();
        self.primitive_transfer.clear();
        self.depth = 0.0;
        let clear_color: [f64; 4] = clear_color.into();
        self.clear_color = wgpu::Color {
            r: clear_color[0],
            g: clear_color[1],
            b: clear_color[2],
            a: clear_color[3],
        };
    }
    pub(crate) fn end(&mut self) {}

    fn add_blit(
        &mut self,
        pos: IVec2,
        tex: UVec2,
        size: UVec2,
        filtering: Color,
        alpha_threshold: u8,
    ) {
        self.blit_transfer.push(GPUBlitData {
            color: filtering.into(),
            depth: (self.depth - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH),
            pos: [pos.x as i16, pos.y as i16],
            tex: [tex.x as u16, tex.y as u16],
            size: [size.x as u16, size.y as u16],
            threshold: (alpha_threshold as f32 / 255.0),
        });
    }

    pub(crate) fn blit_rect(
        &mut self,
        texture: TextureHandle,
        extent: IRect,
        texture_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) {
        let mut blit_count;

        match wrap_mode {
            TextureWrapMode::Clamp => {
                self.add_blit(
                    extent.tl(),
                    texture_extent.tl().as_uvec2(),
                    texture_extent.size().min(extent.size()),
                    filtering,
                    alpha_threshold,
                );
                blit_count = 1;
            }
            TextureWrapMode::Repeat => {
                // Calculate blit count
                let full_hblit_count = extent.width() / texture_extent.width();
                let full_vblit_count = extent.height() / texture_extent.height();
                blit_count = full_vblit_count * full_hblit_count;

                // Insert full blits in transfer buffer
                for y in 0..full_vblit_count {
                    for x in 0..full_hblit_count {
                        let pos_x = extent.tl().x + (x * texture_extent.width()) as i32;
                        let pos_y = extent.tl().y + (y * texture_extent.height()) as i32;
                        self.add_blit(
                            IVec2::new(pos_x, pos_y),
                            texture_extent.tl().as_uvec2(),
                            texture_extent.size(),
                            filtering,
                            alpha_threshold,
                        );
                    }
                }

                // Insert partial blits in transfer buffer
                let partial_hblit_size = extent.width() % texture_extent.width();
                let partial_vblit_size = extent.height() % texture_extent.height();

                if partial_hblit_size > 0 {
                    for y in 0..full_vblit_count {
                        let pos_x =
                            extent.tl().x + (full_hblit_count * texture_extent.width()) as i32;
                        let pos_y = extent.tl().y + (y * texture_extent.height()) as i32;
                        let size = UVec2::new(partial_hblit_size, texture_extent.height());
                        self.add_blit(
                            IVec2::new(pos_x, pos_y),
                            texture_extent.tl().as_uvec2(),
                            size,
                            filtering,
                            alpha_threshold,
                        );
                        blit_count += 1;
                    }
                }

                if partial_vblit_size > 0 {
                    for x in 0..full_hblit_count {
                        let pos_x = extent.tl().x + (x * texture_extent.width()) as i32;
                        let pos_y =
                            extent.tl().y + (full_vblit_count * texture_extent.height()) as i32;
                        let size = UVec2::new(texture_extent.width(), partial_vblit_size);
                        self.add_blit(
                            IVec2::new(pos_x, pos_y),
                            texture_extent.tl().as_uvec2(),
                            size,
                            filtering,
                            alpha_threshold,
                        );
                        blit_count += 1;
                    }
                }

                if partial_hblit_size > 0 && partial_vblit_size > 0 {
                    let pos_x = extent.tl().x + (full_hblit_count * texture_extent.width()) as i32;
                    let pos_y = extent.tl().y + (full_vblit_count * texture_extent.height()) as i32;
                    let size = UVec2::new(partial_hblit_size, partial_vblit_size);
                    self.add_blit(
                        IVec2::new(pos_x, pos_y),
                        texture_extent.tl().as_uvec2(),
                        size,
                        filtering,
                        alpha_threshold,
                    );
                    blit_count += 1;
                }
            }
            TextureWrapMode::Mirror => {
                todo!();
            }
        }

        self.depth += DEPTH_INCREMENT;

        // Reuse command or create new one
        let mut new_command_required = true;
        if let Some(GraphicsCommand::Blit(blit)) = self.commands.last_mut() {
            if blit.texture == texture {
                blit.instance_count += blit_count;
                new_command_required = false;
            }
        }
        if new_command_required {
            self.commands.push(GraphicsCommand::Blit(BlitBatch {
                texture,
                instance_start: self.blit_transfer.len() as u32 - blit_count,
                instance_count: blit_count,
            }));
        }
    }
    pub(crate) fn blit_viewport(
        &mut self,
        viewport: ViewportHandle,
        extent: wgpu::Extent3d,
        position: IVec2,
    ) {
        let size = UVec2::new(extent.width, extent.height);
        self.add_blit(position, UVec2::ZERO, size, Color::WHITE, 0);
        self.commands.push(GraphicsCommand::Viewport(ViewportBatch {
            viewport,
            blit_index: self.blit_transfer.len() as u32 - 1,
        }));
        self.depth += DEPTH_INCREMENT;
    }
    fn add_triangles_primitive_command(&mut self, vertex_count: u32) {
        let mut new_command_required = true;
        if let Some(GraphicsCommand::Triangles(primitive)) = self.commands.last_mut() {
            primitive.vertex_count += vertex_count;
            new_command_required = false;
        }
        if new_command_required {
            self.commands
                .push(GraphicsCommand::Triangles(PrimitiveBatch {
                    vertex_start: self.primitive_transfer.len() as u32 - vertex_count,
                    vertex_count,
                }));
        }
    }
    fn add_lines_primitive_command(&mut self, vertex_count: u32) {
        let mut new_command_required = true;
        if let Some(GraphicsCommand::Lines(primitive)) = self.commands.last_mut() {
            primitive.vertex_count += vertex_count;
            new_command_required = false;
        }
        if new_command_required {
            self.commands.push(GraphicsCommand::Lines(PrimitiveBatch {
                vertex_start: self.primitive_transfer.len() as u32 - vertex_count,
                vertex_count,
            }));
        }
    }
    fn add_points_primitive_command(&mut self, vertex_count: u32) {
        let mut new_command_required = true;
        if let Some(GraphicsCommand::Points(primitive)) = self.commands.last_mut() {
            primitive.vertex_count += vertex_count;
            new_command_required = false;
        }
        if new_command_required {
            self.commands.push(GraphicsCommand::Points(PrimitiveBatch {
                vertex_start: self.primitive_transfer.len() as u32 - vertex_count,
                vertex_count,
            }));
        }
    }
    pub(crate) fn fill_rect(&mut self, extent: IRect, color: Color) {
        let color: [f32; 4] = color.into();
        let depth = (self.depth - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH);
        self.depth += DEPTH_INCREMENT;
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.bl().x, extent.bl().y + 1],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tl().x, extent.tl().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tr().x + 1, extent.tr().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tr().x + 1, extent.tr().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.br().x + 1, extent.br().y + 1],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.bl().x, extent.bl().y + 1],
            depth,
            color,
        });
        self.add_triangles_primitive_command(6);
    }
    pub(crate) fn draw_rect(&mut self, extent: IRect, color: Color) {
        let color: [f32; 4] = color.into();
        let depth = (self.depth - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH);
        self.depth += DEPTH_INCREMENT;
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tl().x, extent.tl().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tr().x, extent.tr().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.bl().x, extent.bl().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.br().x + 1, extent.br().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tl().x, extent.tl().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.bl().x, extent.bl().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.tr().x, extent.tr().y],
            depth,
            color,
        });
        self.primitive_transfer.push(GPUPrimitiveVertexData {
            pos: [extent.br().x, extent.br().y + 1],
            depth,
            color,
        });
        self.add_lines_primitive_command(8);
    }
    pub(crate) fn draw_line(&mut self, x0: IVec2, x1: IVec2, color: Color) {
        let color: [f32; 4] = color.into();
        let depth = (self.depth - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH);
        self.depth += DEPTH_INCREMENT;
        // self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [x0.x, x0.y], depth, color });
        // self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [x1.x, x1.y], depth, color });
        // self.add_lines_primitive_command(2);
        let mut vertex_count = 0;
        rasterizer::draw_line(x0, x1, |p| {
            self.primitive_transfer.push(GPUPrimitiveVertexData {
                pos: [p.x, p.y],
                depth,
                color,
            });
            vertex_count += 1;
        });
        self.add_points_primitive_command(vertex_count);
    }
    pub(crate) fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) {
        self.draw_line((x, y0).into(), (x, y1).into(), color)
    }
    pub(crate) fn draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) {
        self.draw_line((x0, y).into(), (x1, y).into(), color)
    }
    pub(crate) fn scissor(&mut self, extent: IRect) {
        self.commands.push(GraphicsCommand::Scissor(extent));
    }
}
