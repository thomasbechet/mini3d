use std::f32::consts;
use std::mem;

use futures::executor;
use mini3d::glam::{UVec2, Vec4, Mat4, Vec3};
use mini3d::graphics::SCREEN_ASPECT_RATIO;
use mini3d::{
    application::Application,
    graphics::{
        rasterizer::{self, Plotable},
        SCREEN_HEIGHT, SCREEN_PIXEL_COUNT, SCREEN_VIEWPORT, SCREEN_WIDTH,
    },
};
use wgpu::util::DeviceExt;
use wgpu::{include_wgsl, SurfaceError};
use winit::dpi::PhysicalSize;

#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

const PIXEL_WHITE: Pixel = Pixel {
    r: 255,
    g: 255,
    b: 255,
    a: 255,
};

struct RenderBuffer {
    buffer: Box<[Pixel]>,
}

impl RenderBuffer {
    pub fn new() -> RenderBuffer {
        RenderBuffer {
            buffer: vec![Pixel::default(); SCREEN_PIXEL_COUNT].into_boxed_slice(),
        }
    }
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.buffer)
    }
}

pub struct WGPUContext {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    texture_size: wgpu::Extent3d,

    camera_matrix: Mat4,

    // Buffers
    ui_buffer: RenderBuffer,
    vertex_buffer: wgpu::Buffer,
    indice_buffer: wgpu::Buffer,
    indice_count: usize,
    uniform_buffer: wgpu::Buffer,

    // Textures
    ui_texture: wgpu::Texture,
    render_texture: wgpu::Texture,
    render_texture_view: wgpu::TextureView,

    // Pipelines
    blit_pipeline: wgpu::RenderPipeline,
    scene_pipeline: wgpu::RenderPipeline,
    post_process_pipeline: wgpu::RenderPipeline,

    // Bind Groups
    ui_bind_group: wgpu::BindGroup,
    scene_bind_group: wgpu::BindGroup,
    render_bind_group: wgpu::BindGroup, 
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 4],
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let vertex_data = [
        // top (0, 0, 1)
        Vertex{position: [-1.0, -1.0, 1.0, 1.0]},
        Vertex{position: [1.0, -1.0, 1.0, 1.0]},
        Vertex{position: [1.0, 1.0, 1.0, 1.0]},
        Vertex{position: [-1.0, 1.0, 1.0, 1.0]},
        // bottom (0, 0, -1)
        Vertex{position: [-1.0, 1.0, -1.0, 1.0]},
        Vertex{position: [1.0, 1.0, -1.0, 1.0]},
        Vertex{position: [1.0, -1.0, -1.0, 1.0]},
        Vertex{position: [-1.0, -1.0, -1.0, 1.0]},
        // right (1, 0, 0)
        Vertex{position: [1.0, -1.0, -1.0, 1.0]},
        Vertex{position: [1.0, 1.0, -1.0, 1.0]},
        Vertex{position: [1.0, 1.0, 1.0, 1.0]},
        Vertex{position: [1.0, -1.0, 1.0, 1.0]},
        // left (-1, 0, 0)
        Vertex{position: [-1.0, -1.0, 1.0, 1.0]},
        Vertex{position: [-1.0, 1.0, 1.0, 1.0]},
        Vertex{position: [-1.0, 1.0, -1.0, 1.0]},
        Vertex{position: [-1.0, -1.0, -1.0, 1.0]},
        // front (0, 1, 0)
        Vertex{position: [1.0, 1.0, -1.0, 1.0]},
        Vertex{position: [-1.0, 1.0, -1.0, 1.0]},
        Vertex{position: [-1.0, 1.0, 1.0, 1.0]},
        Vertex{position: [1.0, 1.0, 1.0, 1.0]},
        // back (0, -1, 0)
        Vertex{position: [1.0, -1.0, 1.0, 1.0]},
        Vertex{position: [-1.0, -1.0, 1.0, 1.0]},
        Vertex{position: [-1.0, -1.0, -1.0, 1.0]},
        Vertex{position: [1.0, -1.0, -1.0, 1.0]},
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    (vertex_data.to_vec(), index_data.to_vec())
}

fn generate_matrix(aspect_ratio: f32) -> Mat4 {
    let projection = Mat4::perspective_rh(consts::FRAC_PI_4, aspect_ratio, 1.0, 10.0);
    let view = Mat4::look_at_rh(
        Vec3::new(1.5f32, -5.0, 3.0),
        Vec3::ZERO,
        Vec3::Z,
    );
    projection * view
}

impl WGPUContext {

    pub fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W) -> Self {
        let size = PhysicalSize::new(SCREEN_WIDTH, SCREEN_HEIGHT);

        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let (device, queue) = executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None,
            },
            None,
        ))
        .unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // format: surface.get_supported_formats(&adapter).first().unwrap().to_owned(),
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        // Create buffers
        let ui_buffer = RenderBuffer::new();

        let vertex_size = mem::size_of::<Vertex>();
        let (vertex_data, indice_data) = create_vertices();

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let indice_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("indice_buffer"),
            contents: bytemuck::cast_slice(&indice_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        let camera_matrix = generate_matrix(SCREEN_ASPECT_RATIO);
        let mat_ref: &[f32; 16] = camera_matrix.as_ref();
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("uniform_buffer"),
            contents: bytemuck::cast_slice(mat_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create textures
        let texture_size = wgpu::Extent3d {
            width: SCREEN_WIDTH as u32,
            height: SCREEN_HEIGHT as u32,
            depth_or_array_layers: 1,
        };
        let ui_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            label: Some("ui_texture"),
        });
        let ui_texture_view = ui_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_texture = device.create_texture(&wgpu::TextureDescriptor {
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            label: Some("render_texture"),
        });
        let render_texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        // Create pipelines
        let blit_shader = device.create_shader_module(include_wgsl!("blit.wgsl"));
        let blit_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
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
                label: None,
            });
        let blit_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("blit_pipeline_layout"),
                bind_group_layouts: &[&blit_bind_group_layout],
                push_constant_ranges: &[],
            });
        let blit_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("blit_pipeline"),
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
                    format: wgpu::TextureFormat::Rgba8Unorm,
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let scene_shader = device.create_shader_module(include_wgsl!("scene.wgsl"));
        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer { 
                            ty: wgpu::BufferBindingType::Uniform, 
                            has_dynamic_offset: false, 
                            min_binding_size: wgpu::BufferSize::new(64),
                        },
                        count: None,
                    },
                ],
                label: None,
            });
        let scene_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("scene_pipeline_layout"),
                bind_group_layouts: &[&scene_bind_group_layout],
                push_constant_ranges: &[],
            });
        let scene_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                }
            ],
        };
        let scene_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene_pipeline"),
            layout: Some(&scene_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &scene_shader,
                entry_point: "vs_main",
                buffers: &[scene_vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &scene_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let post_process_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("post_process_pipeline"),
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
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
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
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        // Create bind groups
        let ui_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&ui_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("ui_bind_group"),
        });
        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &scene_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
            label: Some("scene_bind_group"),
        });
        let render_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &blit_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&render_texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture_sampler),
                },
            ],
            label: Some("render_bind_group"),
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,

            texture_size,

            camera_matrix,

            ui_buffer,
            vertex_buffer,
            indice_buffer,
            indice_count: indice_data.len(),
            uniform_buffer,

            ui_texture,
            render_texture,
            render_texture_view,

            blit_pipeline,
            scene_pipeline,
            post_process_pipeline,

            ui_bind_group,
            scene_bind_group,
            render_bind_group,
        }
    }

    pub fn recreate(&mut self) {
        self.resize(self.size.width, self.size.height);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.size.width = width;
            self.size.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn render(&mut self, app: &Application) -> Result<(), SurfaceError> {
        self.clear();

        // Process immediate commands
        for cmd in &app.graphics.commands {
            match cmd {
                mini3d::graphics::immediate_command::ImmediateCommand::Print {
                    p,
                    text,
                    font_id,
                } => {
                    rasterizer::print(
                        self,
                        *p,
                        text.as_str(),
                        &app.asset_manager.fonts.get(font_id).unwrap().resource,
                    );
                }
                mini3d::graphics::immediate_command::ImmediateCommand::DrawLine { p0, p1 } => {
                    rasterizer::draw_line(self, *p0, *p1);
                }
                mini3d::graphics::immediate_command::ImmediateCommand::DrawVLine { x, y0, y1 } => {
                    rasterizer::draw_vline(self, *x, *y0, *y1);
                }
                mini3d::graphics::immediate_command::ImmediateCommand::DrawHLine { y, x0, x1 } => {
                    rasterizer::draw_hline(self, *y, *x0, *x1);
                }
                mini3d::graphics::immediate_command::ImmediateCommand::DrawRect { rect } => {
                    let mut rect = *rect;
                    rect.clamp(&SCREEN_VIEWPORT);
                    rasterizer::draw_rect(self, rect);
                }
                mini3d::graphics::immediate_command::ImmediateCommand::FillRect { rect } => {
                    let mut rect = *rect;
                    rect.clamp(&SCREEN_VIEWPORT);
                    rasterizer::fill_rect(self, rect);
                }
            }
        }

        // Acquire next surface texture
        let output = self.surface.get_current_texture()?;
        let target_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create frame encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("main_encoder"),
            });

        // Copy software render buffer to gpu
        {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &self.ui_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                self.ui_buffer.as_bytes(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * SCREEN_WIDTH as u32),
                    rows_per_image: std::num::NonZeroU32::new(SCREEN_HEIGHT as u32),
                },
                self.texture_size,
            );
        }

        // Compute viewport
        let viewport = compute_viewport((self.size.width, self.size.height).into());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_to_texture"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_texture_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.scene_pipeline);
            render_pass.set_bind_group(0, &self.scene_bind_group, &[]);
            render_pass.set_index_buffer(self.indice_buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.draw_indexed(0..self.indice_count as u32, 0, 0..1);

            render_pass.set_pipeline(&self.blit_pipeline);
            render_pass.set_bind_group(0, &self.ui_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("render_to_target"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 25.0 / 255.0,
                            g: 27.0 / 255.0,
                            b: 43.0 / 255.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_viewport(viewport.x, viewport.y, viewport.z, viewport.w, 0.0, 1.0);
        
            render_pass.set_pipeline(&self.post_process_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn clear(&mut self) {
        self.ui_buffer.buffer.fill(Pixel::default());
    }
}

pub fn compute_viewport(size: UVec2) -> Vec4 {
    if size.x as f32 / size.y as f32 >= (SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32) {
        let w = SCREEN_WIDTH as f32 * size.y as f32 / SCREEN_HEIGHT as f32;
        let h = size.y as f32;
        let x = (size.x / 2) as f32 - (w / 2.0);
        let y = 0.0;
        (x, y, w, h).into()
    } else {
        let w = size.x as f32;
        let h = SCREEN_HEIGHT as f32 * size.x as f32 / SCREEN_WIDTH as f32;
        let x = 0.0;
        let y = (size.y / 2) as f32 - (h / 2.0);
        (x, y, w, h).into()
    }
}

impl Plotable for WGPUContext {
    fn plot(&mut self, p: UVec2) {
        self.ui_buffer.buffer[p.y as usize * SCREEN_WIDTH as usize + p.x as usize] =
            PIXEL_WHITE;
    }
}
