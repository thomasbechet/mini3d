use std::collections::{HashMap, hash_map};

use mini3d::renderer::{backend::{TextureHandle, CanvasHandle, CanvasViewportHandle}, color::Color};
use wgpu::include_wgsl;

use crate::{context::WGPUContext, texture::Texture, canvas::{Canvas, CanvasSprite, CANVAS_COLOR_FORMAT}};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUCanvasData {
    resolution: [u32; 2],
    _pad: [u64; 31],
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct GPUBlitData {
    color: [f32; 3],
    depth: f32,
    pos: [i16; 2],
    tex: [u16; 2],
    size: [u16; 2],
    _pad: u32,
}

#[derive(Debug)]
struct CanvasSpriteBatch {
    texture: TextureHandle,
    blit_start: usize,
    blit_count: usize,
}

struct CanvasViewportBatch {
    viewport: CanvasViewportHandle,
    blit_start: usize,
}

const MAX_CANVAS_COUNT: usize = 32;
const MAX_BLIT_COUNT: usize = 512;
const MAX_DEPTH: f32 = 1000.0;
const MIN_DEPTH: f32 = -1000.0;

fn create_blit_bind_group(
    context: &WGPUContext,
    layout: &wgpu::BindGroupLayout,
    buffer: &wgpu::Buffer,
    texture: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            },	
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(texture),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

pub(crate) struct CanvasRenderer {

    canvas_bind_group: wgpu::BindGroup,
    canvas_buffer: wgpu::Buffer,
    canvas_transfer: [GPUCanvasData; MAX_CANVAS_COUNT],
    canvas_offsets: HashMap<CanvasHandle, u32>, 

    blit_pipeline: wgpu::RenderPipeline,
    blit_bind_group_layout: wgpu::BindGroupLayout,

    blit_buffer: wgpu::Buffer,
    blit_transfer: [GPUBlitData; MAX_BLIT_COUNT],

    sprite_bind_groups: HashMap<TextureHandle, wgpu::BindGroup>,
    sprite_batches: HashMap<CanvasHandle, Vec<CanvasSpriteBatch>>,
    viewport_bind_groups: HashMap<CanvasViewportHandle, wgpu::BindGroup>,
    viewport_batches: HashMap<CanvasHandle, Vec<CanvasViewportBatch>>,
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
        let blit_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("blit_buffer"),
            size: std::mem::size_of::<GPUBlitData>() as u64 * MAX_BLIT_COUNT as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Create shader modules
        let blit_shader = context.device.create_shader_module(include_wgsl!("shader/canvas_blit.wgsl"));
        // let rect_shader = context.device.create_shader_module(include_wgsl!("shader/canvas_rect.wgsl"));

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
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new(64), 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
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
        
        // Create pipelines
        let blit_pipeline = context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("canvas_blit_pipeline"),
            layout: Some(&blit_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &blit_shader,
                entry_point: "vs_main",
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &blit_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: CANVAS_COLOR_FORMAT,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    // blend: Some(wgpu::BlendState::REPLACE),
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

        // Create bind groups
        let canvas_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("canvas_bind_group"),
            layout: &canvas_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    // resource: canvas_buffer.as_entire_binding(),
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
            canvas_offsets: Default::default(),
            
            blit_pipeline, 
            blit_bind_group_layout,

            blit_buffer,
            blit_transfer: [GPUBlitData::default(); MAX_BLIT_COUNT],

            sprite_bind_groups: Default::default(),
            sprite_batches: Default::default(),
            viewport_bind_groups: Default::default(),
            viewport_batches: Default::default(),
        }
    }

    pub(crate) fn write_buffers(
        &mut self,
        context: &WGPUContext,
        sampler: &wgpu::Sampler,
        textures: &HashMap<TextureHandle, Texture>,
        canvases: &HashMap<CanvasHandle, Canvas>,
    ) {

        // Initialize buffer pointer
        let mut current_blit_index = 0;

        // Build canvas batches
        for (canvas_index, (handle, canvas)) in canvases.iter().enumerate() {

            // Create and clear the canvas blit batches
            let sprite_batches = self.sprite_batches.entry(*handle).or_default();
            sprite_batches.clear();
            let viewport_batches = self.viewport_batches.entry(*handle).or_default();
            viewport_batches.clear();

            // Obtain list of items reference and sort by texture
            let mut sprite_items: Vec<&CanvasSprite> = canvas.sprites.values().collect();
            sprite_items.sort_by(|a, b| a.texture.cmp(&b.texture));
            
            // Build blit batches
            for sprite in sprite_items {

                // Append the blit to the transfer buffer
                self.blit_transfer[current_blit_index] = GPUBlitData {
                    color: sprite.color.into(),
                    depth: ((sprite.z_index as f32) - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH),
                    pos: [sprite.position.x as i16, sprite.position.y as i16],
                    tex: [sprite.extent.left() as u16, sprite.extent.top() as u16],
                    size: [sprite.extent.width() as u16, sprite.extent.height() as u16],
                    _pad: 0,
                };

                // Insert the first batch
                if sprite_batches.is_empty() {
                    sprite_batches.push(CanvasSpriteBatch {
                        texture: sprite.texture,
                        blit_start: current_blit_index,
                        blit_count: 0,
                    });
                }

                // Check if we need to create a new batch
                if let Some(batch) = sprite_batches.last_mut() {
                    if batch.texture == sprite.texture {
                        batch.blit_count += 1;
                    } else {
                        sprite_batches.push(CanvasSpriteBatch {
                            texture: sprite.texture,
                            blit_start: current_blit_index,
                            blit_count: 1,
                        });
                    }
                }

                current_blit_index += 1;
            }

            // Build viewport batches
            for (handle, viewport) in &canvas.viewports {
                
                // Append the blit to the transfer buffer
                let extent = viewport.extent;
                self.blit_transfer[current_blit_index] = GPUBlitData {
                    color: Color::WHITE.into(),
                    depth: ((viewport.z_index as f32) - MIN_DEPTH) / (MAX_DEPTH - MIN_DEPTH),
                    pos: [viewport.position.x as i16, viewport.position.y as i16],
                    tex: [0, 0],
                    size: [extent.width as u16, extent.height as u16],
                    _pad: 0,
                };

                viewport_batches.push(CanvasViewportBatch { viewport: *handle, blit_start: current_blit_index });

                current_blit_index += 1;
            }

            // Build sprite bind groups
            for batch in sprite_batches {
                if let hash_map::Entry::Vacant(e) = self.sprite_bind_groups.entry(batch.texture) {
                    let texture = textures.get(&batch.texture).unwrap();
                    let bind_group = create_blit_bind_group(context, &self.blit_bind_group_layout, &self.blit_buffer, &texture.view, sampler);
                    e.insert(bind_group);
                }
            }
            
            // Build viewport bind groups
            for (handle, viewport) in &canvas.viewports {
                if let hash_map::Entry::Vacant(e) = self.viewport_bind_groups.entry(*handle) {
                    let bind_group = create_blit_bind_group(context, &self.blit_bind_group_layout, &self.blit_buffer, &viewport.color_view, sampler);
                    e.insert(bind_group);
                }
            }

            // Write canvas buffer
            self.canvas_transfer[canvas_index] = GPUCanvasData {
                resolution: [canvas.extent.width as u32, canvas.extent.height as u32],
                _pad: [0; 31],
            };

            // Save canvas buffer offset
            let offset = std::mem::size_of::<GPUCanvasData>() * canvas_index;
            self.canvas_offsets.insert(*handle, offset as u32);
        }

        // Write buffer
        context.queue.write_buffer(&self.canvas_buffer, 0, bytemuck::cast_slice(&self.canvas_transfer[0..canvases.len()]));
        context.queue.write_buffer(&self.blit_buffer,0, bytemuck::cast_slice(&self.blit_transfer[0..current_blit_index]));

    }

    pub(crate) fn render(
        &mut self, 
        canvases: &HashMap<CanvasHandle, Canvas>,
        encoder: &mut wgpu::CommandEncoder,
    ) {

        // Iterate over all canvases
        for (canvas_handle, canvas) in canvases.iter() {

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("canvas_blit_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &canvas.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(canvas.clear_color),
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

            // Set pipeline and bind group
            render_pass.set_pipeline(&self.blit_pipeline);
            let offset = self.canvas_offsets.get(canvas_handle).unwrap();
            render_pass.set_bind_group(0, &self.canvas_bind_group, &[*offset]);
            
            // Render sprites
            for batch in self.sprite_batches.get(canvas_handle).unwrap() {

                // Bind group
                let bind_group = self.sprite_bind_groups.get(&batch.texture).unwrap();
                render_pass.set_bind_group(1, bind_group, &[]);
                
                // Draw
                render_pass.draw(0..6, batch.blit_start as u32..(batch.blit_start + batch.blit_count) as u32);
            }

            // Render viewports
            for batch in self.viewport_batches.get(canvas_handle).unwrap() {

                // Bind group
                let bind_group = self.viewport_bind_groups.get(&batch.viewport).unwrap();
                render_pass.set_bind_group(1, bind_group, &[]);
                
                // Draw
                render_pass.draw(0..6, batch.blit_start as u32..(batch.blit_start + 1) as u32);
            }
        }
    }
}