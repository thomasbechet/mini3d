use std::collections::{HashMap, hash_map};

use mini3d::{renderer::{backend::{TextureHandle, ViewportHandle}}, anyhow::Result};
use wgpu::{include_wgsl, vertex_attr_array};

use crate::{context::WGPUContext, texture::Texture, graphics_canvas::{GraphicsCanvas, CANVAS_COLOR_FORMAT, CANVAS_DEPTH_FORMAT}, viewport::Viewport, graphics_render_pass::GraphicsCommand};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUGlobalData {
    pub(crate) resolution: [u32; 2],
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

const MAX_GLOBAL_DATA_COUNT: usize = 32;

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

struct GraphicsPipelines {
    blit: wgpu::RenderPipeline,
    primitive_triangles: wgpu::RenderPipeline,
    primitive_lines: wgpu::RenderPipeline,
}

impl GraphicsPipelines {
    fn new(
        context: &WGPUContext,
        blit_pipeline_layout: &wgpu::PipelineLayout,
        blit_shader: &wgpu::ShaderModule,
        primitive_pipeline_layout: &wgpu::PipelineLayout,
        primitive_shader: &wgpu::ShaderModule,
        color_format: wgpu::TextureFormat,
        depth_format: wgpu::TextureFormat,
    ) -> Self {
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
            layout: Some(blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: blit_shader,
                entry_point: "vs_main",
                buffers: &blit_vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: blit_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: color_format,
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
            label: Some("primitive_triangles_pipeline"),
            layout: Some(primitive_pipeline_layout),
            vertex: wgpu::VertexState {
                module: primitive_shader,
                entry_point: "vs_main",
                buffers: &primitive_triangle_vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: primitive_shader,
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
            label: Some("primitive_lines_pipeline"),
            layout: Some(primitive_pipeline_layout),
            vertex: wgpu::VertexState {
                module: primitive_shader,
                entry_point: "vs_main",
                buffers: &primitive_triangle_vertex_buffer_layout,
            },
            fragment: Some(wgpu::FragmentState {
                module: primitive_shader,
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
        Self { blit: blit_pipeline, primitive_triangles: primitive_triangles_pipeline, primitive_lines: primitive_lines_pipeline }
    }
}

pub(crate) struct GraphicsRenderer {

    global_bind_group_layout: wgpu::BindGroupLayout,

    blit_bind_group_layout: wgpu::BindGroupLayout,

    texture_bind_groups: HashMap<TextureHandle, wgpu::BindGroup>,
    viewport_bind_groups: HashMap<ViewportHandle, wgpu::BindGroup>,

    pipelines: GraphicsPipelines,
    
    sampler: wgpu::Sampler,
}

impl GraphicsRenderer {

    pub(crate) fn new(context: &WGPUContext) -> Self {
        
        // Create buffers
        let global_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("global_buffer"),
            size: std::mem::size_of::<GPUGlobalData>() as u64 * MAX_GLOBAL_DATA_COUNT as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shader modules
        let blit_shader = context.device.create_shader_module(include_wgsl!("shader/graphics_blit.wgsl"));
        let primitive_shader = context.device.create_shader_module(include_wgsl!("shader/graphics_primitive.wgsl"));

        // Create bind group layouts
        let global_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("global_bind_group_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GPUGlobalData>() as u64),
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
            bind_group_layouts: &[&global_bind_group_layout, &blit_bind_group_layout],
            push_constant_ranges: &[],
        });
        let primitive_pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("canvas_primitive_pipeline_layout"),
            bind_group_layouts: &[&global_bind_group_layout],
            push_constant_ranges: &[],
        });

        // Create pipelines
        let pipelines = GraphicsPipelines::new(context, &blit_pipeline_layout, &blit_shader, 
            &primitive_pipeline_layout, &primitive_shader, CANVAS_COLOR_FORMAT, CANVAS_DEPTH_FORMAT);

        // Create other resources
        let sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("graphics_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Self {
            global_bind_group_layout,
            
            blit_bind_group_layout,

            texture_bind_groups: Default::default(),
            viewport_bind_groups: Default::default(),

            pipelines,

            sampler,
        }
    }

    pub(crate) fn reset(&mut self) {
        self.texture_bind_groups.clear();
        self.viewport_bind_groups.clear();
    }

    pub(crate) fn create_global_bind_group(
        &self,
        context: &WGPUContext,
        buffer: &wgpu::Buffer,
    ) -> wgpu::BindGroup {
        context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        })
    }

    pub(crate) fn render_canvas(
        &mut self,
        context: &WGPUContext,
        textures: &HashMap<TextureHandle, Texture>,
        viewports: &HashMap<ViewportHandle, Viewport>,
        canvas: &mut GraphicsCanvas,
        encoder: &mut wgpu::CommandEncoder,
    ) -> Result<()> {

        // Transfer buffers
        canvas.render_pass.write_buffers(context)?;
        
        // Build missing bind groups
        for command in &canvas.render_pass.commands {
            match command {
                GraphicsCommand::Blit(cmd) => {
                    if let hash_map::Entry::Vacant(e) = self.texture_bind_groups.entry(cmd.texture) {
                        let texture = textures.get(&cmd.texture).unwrap();
                        let bind_group = create_blit_bind_group(context, &self.blit_bind_group_layout, &texture.view, &self.sampler);
                        e.insert(bind_group);
                    }
                },
                GraphicsCommand::Viewport(cmd) => {
                    if let hash_map::Entry::Vacant(e) = self.viewport_bind_groups.entry(cmd.viewport) {
                        let viewport = viewports.get(&cmd.viewport).unwrap();
                        let bind_group = create_blit_bind_group(context, &self.blit_bind_group_layout, &viewport.color_view, &self.sampler);
                        e.insert(bind_group);
                    }
                },
                _ => {},
            }
        }

        // Begin pass
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("canvas_graphics_render_pass"),
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

        // Set global bind group
        render_pass.set_bind_group(0, &canvas.global_bind_group, &[]);

        // Iterate commands
        for command in canvas.render_pass.commands.iter() {
            match command {
                GraphicsCommand::Blit(cmd) => {
                    render_pass.set_pipeline(&self.pipelines.blit);
                    let start = cmd.instance_start as u64 * std::mem::size_of::<GPUBlitData>() as u64;
                    let stop = start + cmd.instance_count as u64 * std::mem::size_of::<GPUBlitData>() as u64;       
                    render_pass.set_bind_group(1, self.texture_bind_groups.get(&cmd.texture).unwrap(), &[]);
                    render_pass.set_vertex_buffer(0, canvas.render_pass.blit_buffer.slice(start..stop));
                    render_pass.draw(0..6, 0..cmd.instance_count);
                },
                GraphicsCommand::Viewport(cmd) => {
                    render_pass.set_pipeline(&self.pipelines.blit);
                    let start = cmd.blit_index as u64 * std::mem::size_of::<GPUBlitData>() as u64;
                    let stop = start + std::mem::size_of::<GPUBlitData>() as u64;
                    render_pass.set_bind_group(1, self.viewport_bind_groups.get(&cmd.viewport).unwrap(), &[]);    
                    render_pass.set_vertex_buffer(0, canvas.render_pass.blit_buffer.slice(start..stop));
                    render_pass.draw(0..6, 0..1);
                },
                GraphicsCommand::Triangles(cmd) => {
                    render_pass.set_pipeline(&self.pipelines.primitive_triangles);
                    render_pass.set_vertex_buffer(0, canvas.render_pass.primitive_buffer.slice(..));
                    render_pass.draw(cmd.vertex_start..(cmd.vertex_start + cmd.vertex_count), 0..1);
                },
                GraphicsCommand::Lines(cmd) => {
                    render_pass.set_pipeline(&self.pipelines.primitive_lines);
                    render_pass.set_vertex_buffer(0, canvas.render_pass.primitive_buffer.slice(..));
                    render_pass.draw(cmd.vertex_start..(cmd.vertex_start + cmd.vertex_count), 0..1);
                },
                GraphicsCommand::Scissor(cmd) => {
                    render_pass.set_scissor_rect(cmd.left() as u32, cmd.top() as u32, cmd.width(), cmd.height());
                },
            }
        }

        Ok(())
    }
}