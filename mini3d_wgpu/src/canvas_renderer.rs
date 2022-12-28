use std::collections::{HashMap, hash_map};

use mini3d::{renderer::{backend::{TextureHandle, CanvasHandle, ViewportHandle}, color::Color}, anyhow::Result, math::rect::IRect, glam::IVec2};
use wgpu::{include_wgsl, vertex_attr_array};

use crate::{context::WGPUContext, texture::Texture, canvas::{Canvas, CANVAS_COLOR_FORMAT}, viewport::Viewport};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUCanvasData {
    resolution: [u32; 2],
    _pad: [u64; 31],
}

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
struct BlitBatch {
    texture: TextureHandle,
    instance_start: u32,
    instance_count: u32,
}

#[derive(Debug)]
struct ViewportBatch {
    viewport: ViewportHandle,
    blit_index: u32,
}

#[derive(Debug)]
struct PrimitiveBatch {
    vertex_start: u32,
    vertex_count: u32,
}

enum RenderCommand {
    Blit(BlitBatch),
    Viewport(ViewportBatch),
    Triangles(PrimitiveBatch),
    Lines(PrimitiveBatch),
    Scissor(IRect),
}

pub(crate) struct CanvasRenderPass {
    commands: Vec<RenderCommand>,
    depth: f32,
    clear_color: wgpu::Color,
    pub(crate) blit_transfer: Vec<GPUBlitData>,
    pub(crate) primitive_transfer: Vec<GPUPrimitiveVertexData>,
}

impl CanvasRenderPass {

    pub(crate) fn begin(&mut self, clear_color: Color) -> Result<()> {
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
        Ok(())
    }

    pub(crate) fn end(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn blit_rect(&mut self, texture: TextureHandle, extent: IRect, position: IVec2, filtering: Color, alpha_threshold: u8) -> Result<()> {
        
        // Insert in transfer buffer
        self.blit_transfer.push(GPUBlitData {
            color: filtering.into(),
            depth: (self.depth - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH),
            pos: [position.x as i16, position.y as i16],
            tex: [extent.left() as u16, extent.top() as u16],
            size: [extent.width() as u16, extent.height() as u16],
            threshold: (alpha_threshold as f32 / 255.0),
        });
        self.depth += DEPTH_INCREMENT;
        
        // Reuse command or create new one
        let mut new_command_required = true;
        if let Some(RenderCommand::Blit(blit)) = self.commands.last_mut() {
            if blit.texture == texture {
                blit.instance_count += 1;
                new_command_required = false;
            }
        }
        if new_command_required {
            self.commands.push(RenderCommand::Blit(BlitBatch { 
                texture, 
                instance_start: self.blit_transfer.len() as u32 - 1,
                instance_count: 1, 
            }));
        }
        Ok(())
    }
    pub(crate) fn blit_viewport(&mut self, viewport: ViewportHandle, extent: wgpu::Extent3d, position: IVec2) -> Result<()> {
        self.blit_transfer.push(GPUBlitData {
            color: Color::WHITE.into(),
            depth: ((self.depth as f32) - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH),
            pos: [position.x as i16, position.y as i16],
            tex: [0, 0],
            size: [extent.width as u16, extent.height as u16],
            threshold: 0.0,
        });
        self.commands.push(RenderCommand::Viewport(ViewportBatch { 
            viewport,
            blit_index: self.blit_transfer.len() as u32 - 1,
        }));
        self.depth += DEPTH_INCREMENT;
        Ok(())
    }
    fn add_triangles_primitive_command(&mut self, vertex_count: u32) {
        let mut new_command_required = true;
        if let Some(RenderCommand::Triangles(primitive)) = self.commands.last_mut() {
            primitive.vertex_count += vertex_count;
            new_command_required = false;
        }
        if new_command_required {
            self.commands.push(RenderCommand::Triangles(PrimitiveBatch { 
                vertex_start: self.primitive_transfer.len() as u32 - vertex_count,
                vertex_count,
            }));
        }
    }
    fn add_lines_primitive_command(&mut self, vertex_count: u32) {
        let mut new_command_required = true;
        if let Some(RenderCommand::Lines(primitive)) = self.commands.last_mut() {
            primitive.vertex_count += vertex_count;
            new_command_required = false;
        }
        if new_command_required {
            self.commands.push(RenderCommand::Lines(PrimitiveBatch { 
                vertex_start: self.primitive_transfer.len() as u32 - vertex_count,
                vertex_count,
            }));
        }
    }
    pub(crate) fn fill_rect(&mut self, extent: IRect, color: Color) -> Result<()> {
        let color: [f32; 4] = color.into();
        let depth = ((self.depth as f32) - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH);
        self.depth += DEPTH_INCREMENT;
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.bl().x, extent.bl().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tl().x, extent.tl().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tr().x, extent.tr().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tr().x, extent.tr().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.br().x, extent.br().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.bl().x, extent.bl().y], depth, color });
        self.add_triangles_primitive_command(6);
        Ok(())
    }
    pub(crate) fn draw_rect(&mut self, extent: IRect, color: Color) -> Result<()> {
        let color: [f32; 4] = color.into();
        let depth = ((self.depth as f32) - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH);
        self.depth += DEPTH_INCREMENT;
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tl().x, extent.tl().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tr().x, extent.tr().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.bl().x, extent.bl().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.br().x + 1, extent.br().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tl().x, extent.tl().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.bl().x, extent.bl().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.tr().x, extent.tr().y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [extent.br().x, extent.br().y + 1], depth, color });
        self.add_lines_primitive_command(8);
        Ok(())
    }
    pub(crate) fn draw_line(&mut self, x0: IVec2, x1: IVec2, color: Color) -> Result<()> {
        let color: [f32; 4] = color.into();
        let depth = ((self.depth as f32) - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH);
        self.depth += DEPTH_INCREMENT;
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [x0.x, x0.y], depth, color });
        self.primitive_transfer.push(GPUPrimitiveVertexData { pos: [x1.x, x1.y], depth, color });
        self.add_lines_primitive_command(2);
        Ok(())
    }
    pub(crate) fn draw_vline(&mut self, x: i32, y0: i32, y1: i32, color: Color) -> Result<()> {
        self.draw_line((x, y0).into(), (x, y1).into(), color)
    }
    pub(crate) fn draw_hline(&mut self, y: i32, x0: i32, x1: i32, color: Color) -> Result<()> {
        self.draw_line((x0, y).into(), (x1, y).into(), color)
    }
    pub(crate) fn scissor(&mut self, extent: IRect) -> Result<()> {
        self.commands.push(RenderCommand::Scissor(extent));
        Ok(())
    }
}

impl Default for CanvasRenderPass {
    fn default() -> Self {
        Self { 
            commands: Default::default(), 
            clear_color: wgpu::Color::TRANSPARENT, 
            depth: 0.0,
            blit_transfer: Default::default(),
            primitive_transfer: Default::default(),
        }
    }
}

const MAX_CANVAS_COUNT: usize = 32;
const MAX_DEPTH: f32 = 1000.0;
const MIN_DEPTH: f32 = -1000.0;
const DEPTH_INCREMENT: f32 = 0.05;

fn create_blit_bind_group(
    context: &WGPUContext,
    layout: &wgpu::BindGroupLayout,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}
pub(crate) struct CanvasRenderer {

    canvas_bind_group: wgpu::BindGroup,
    canvas_buffer: wgpu::Buffer,
    canvas_transfer: [GPUCanvasData; MAX_CANVAS_COUNT],

    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,

    texture_bind_groups: HashMap<TextureHandle, wgpu::BindGroup>,
    viewport_bind_groups: HashMap<ViewportHandle, wgpu::BindGroup>,

    primitive_triangles_pipeline: wgpu::RenderPipeline,
    primitive_lines_pipeline: wgpu::RenderPipeline,
}

impl CanvasRenderer {

    pub(crate) fn new(context: &WGPUContext) -> Self {
        
        // Create buffers
        let canvas_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("canvas_buffer"),
            size: std::mem::size_of::<GPUCanvasData>() as u64 * MAX_CANVAS_COUNT as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shader modules
        let blit_shader = context.device.create_shader_module(include_wgsl!("shader/canvas_blit.wgsl"));
        let primitive_shader = context.device.create_shader_module(include_wgsl!("shader/canvas_primitive.wgsl"));

        // Create bind group layouts
        let canvas_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("canvas_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: true, 
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GPUCanvasData>() as u64),
                },
                count: None,
            }],
        });
        let blit_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("blit_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        // Create pipeline layouts
        let blit_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas_blit_pipeline_layout"),
            bind_group_layouts: &[&canvas_bind_group_layout, &blit_bind_group_layout],
            push_constant_ranges: &[],
        });
        let primitive_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas_primitive_pipeline_layout"),
            bind_group_layouts: &[&canvas_bind_group_layout],
            push_constant_ranges: &[],
        });
        
        // Create pipelines
        let blit_vertex_buffer_layout = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GPUBlitData>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &vertex_attr_array![
                0 => Uint32,
                1 => Uint32,
                2 => Uint32,
                3 => Float32,
                4 => Float32x3,
                5 => Float32,
            ],
        }];
        let blit_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canvas_blit_pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vs_main",
                buffers: &blit_vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: CANVAS_COLOR_FORMAT,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let primitive_triangle_vertex_buffer_layout = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GPUPrimitiveVertexData>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Sint32x2,
                1 => Float32,
                2 => Float32x4,
            ],
        }];
        let primitive_triangles_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canvas_primitive_triangles_pipeline"),
            layout: Some(&primitive_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &primitive_shader,
                entry_point: "vs_main",
                buffers: &primitive_triangle_vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &primitive_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: CANVAS_COLOR_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let primitive_lines_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canvas_primitive_lines_pipeline"),
            layout: Some(&primitive_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &primitive_shader,
                entry_point: "vs_main",
                buffers: &primitive_triangle_vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: &primitive_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: CANVAS_COLOR_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Line,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::GreaterEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create bind groups
        let canvas_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("canvas_bind_group"),
            layout: &canvas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &canvas_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<GPUCanvasData>() as u64),
                    })
                },
            ],
        });

        Self {
            canvas_bind_group, 
            canvas_buffer, 
            canvas_transfer: [GPUCanvasData::default(); MAX_CANVAS_COUNT],
            
            blit_pipeline, 
            blit_bind_group_layout,

            texture_bind_groups: Default::default(),
            viewport_bind_groups: Default::default(),

            primitive_triangles_pipeline,
            primitive_lines_pipeline,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.texture_bind_groups.clear();
    }

    pub(crate) fn render(
        &mut self,
        context: &WGPUContext,
        textures: &HashMap<TextureHandle, Texture>,
        viewports: &HashMap<ViewportHandle, Viewport>,
        sampler: &wgpu::Sampler,
        canvases: &mut HashMap<CanvasHandle, Canvas>,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {

        // Iterate over all canvases
        for (canvas_index, canvas) in canvases.values_mut().enumerate() {

            // Write viewport transfer
            self.canvas_transfer[canvas_index] = GPUCanvasData {
                resolution: [canvas.extent.width as u32, canvas.extent.height as u32],
                _pad: [0; 31],
            };

            // Write transfer buffers
            canvas.write_transfer_buffers(context)?;

            // Build missing bind groups
            for command in &canvas.render_pass.commands {
                match command {
                    RenderCommand::Blit(cmd) => {
                        if let hash_map::Entry::Vacant(e) = self.texture_bind_groups.entry(cmd.texture) {
                            let texture = textures.get(&cmd.texture).unwrap();
                            let bind_group = create_blit_bind_group(context, &self.blit_bind_group_layout, &texture.view, sampler);
                            e.insert(bind_group);
                        }
                    },
                    RenderCommand::Viewport(cmd) => {
                        if let hash_map::Entry::Vacant(e) = self.viewport_bind_groups.entry(cmd.viewport) {
                            let viewport = viewports.get(&cmd.viewport).unwrap();
                            let bind_group = create_blit_bind_group(context, &self.blit_bind_group_layout, &viewport.color_view, sampler);
                            e.insert(bind_group);
                        }
                    },
                    _ => {},
                }
            }

            // Begin pass
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("canvas_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &canvas.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(canvas.render_pass.clear_color),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &canvas.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            // Set canvas bind group
            let offset = std::mem::size_of::<GPUCanvasData>() * canvas_index;
            render_pass.set_bind_group(0, &self.canvas_bind_group, &[offset as u32]);

            // Iterate commands
            for command in canvas.render_pass.commands.iter() {
                match command {
                    RenderCommand::Blit(cmd) => {
                        render_pass.set_pipeline(&self.blit_pipeline);
                        let start = cmd.instance_start as u64 * std::mem::size_of::<GPUBlitData>() as u64;
                        let stop = start + cmd.instance_count as u64 * std::mem::size_of::<GPUBlitData>() as u64;       
                        render_pass.set_bind_group(1, self.texture_bind_groups.get(&cmd.texture).unwrap(), &[]);
                        render_pass.set_vertex_buffer(0, canvas.blit_buffer.slice(start..stop));
                        render_pass.draw(0..6, 0..cmd.instance_count);
                    },
                    RenderCommand::Viewport(cmd) => {
                        render_pass.set_pipeline(&self.blit_pipeline);
                        let start = cmd.blit_index as u64 * std::mem::size_of::<GPUBlitData>() as u64;
                        let stop = start + std::mem::size_of::<GPUBlitData>() as u64;
                        render_pass.set_bind_group(1, self.viewport_bind_groups.get(&cmd.viewport).unwrap(), &[]);    
                        render_pass.set_vertex_buffer(0, canvas.blit_buffer.slice(start..stop));
                        render_pass.draw(0..6, 0..1);
                    },
                    RenderCommand::Triangles(cmd) => {
                        render_pass.set_pipeline(&self.primitive_triangles_pipeline);
                        render_pass.set_vertex_buffer(0, canvas.primitive_buffer.slice(..));
                        render_pass.draw(cmd.vertex_start..(cmd.vertex_start + cmd.vertex_count), 0..1);
                    },
                    RenderCommand::Lines(cmd) => {
                        render_pass.set_pipeline(&self.primitive_lines_pipeline);
                        render_pass.set_vertex_buffer(0, canvas.primitive_buffer.slice(..));
                        render_pass.draw(cmd.vertex_start..(cmd.vertex_start + cmd.vertex_count), 0..1);
                    },
                    RenderCommand::Scissor(cmd) => {
                        render_pass.set_scissor_rect(cmd.left() as u32, cmd.top() as u32, cmd.width(), cmd.height());
                    },
                }
            }
        }

        // Write buffers
        context.queue.write_buffer(&self.canvas_buffer, 0, bytemuck::cast_slice(&self.canvas_transfer[0..canvases.len()]));

        Ok(())
    }
}