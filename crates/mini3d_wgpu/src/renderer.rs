use mini3d_core::service::renderer::{RendererService, DISPLAY_PIXEL_COUNT, RendererError, DISPLAY_WIDTH, DISPLAY_HEIGHT};
use rand::RngCore;
use wgpu::include_wgsl;
use winit::{window::Window};
use futures::executor;

#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

struct RenderBuffer {
    buffer: Box<[Pixel]>
}

impl RenderBuffer {
    pub fn new() -> RenderBuffer {
        RenderBuffer {
            buffer: vec![Pixel::default(); DISPLAY_PIXEL_COUNT].into_boxed_slice()
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
    render_texture: wgpu::Texture,
    render_texture_size: wgpu::Extent3d,
    render_texture_bind_group: wgpu::BindGroup,
    render_pipeline: wgpu::RenderPipeline,
    render_buffer: RenderBuffer
}

fn map_surface_to_renderer_error(error: wgpu::SurfaceError) -> RendererError {
    match error {
        wgpu::SurfaceError::Timeout => RendererError::Timeout,
        wgpu::SurfaceError::Outdated => RendererError::Outdated,
        wgpu::SurfaceError::Lost => RendererError::Lost,
        wgpu::SurfaceError::OutOfMemory => RendererError::OutOfMemory
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3]
}

impl WGPUContext {
    pub fn new(window: &Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::VULKAN);
        let surface = unsafe { instance.create_surface(window) };
        let adapter = executor::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false
            },
        )).unwrap();

        let (device, queue) = executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: if cfg!(target_arch = "wasm32") {
                    wgpu::Limits::downlevel_webgl2_defaults()
                } else {
                    wgpu::Limits::default()
                },
                label: None
            },
            None
        )).unwrap();

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // format: surface.get_supported_formats(&adapter).first().unwrap().to_owned(),
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo
        };
        surface.configure(&device, &config);

        let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

        let render_texture_size = wgpu::Extent3d {
            width: DISPLAY_WIDTH as u32,
            height: DISPLAY_HEIGHT as u32,
            depth_or_array_layers: 1
        };
        let render_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: render_texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: None
            }
        );

        let render_texture_view = render_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_texture_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let render_texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
                    count: None
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None
                }
            ],
            label: None
        });
        let render_texture_bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                layout: &render_texture_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&render_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&render_texture_sampler)
                    }
                ],
                label: None
            }
        );

        let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[&render_texture_bind_group_layout],
            push_constant_ranges: &[]
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[]
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false
            },
            multiview: None
        });

        let render_buffer = RenderBuffer::new();

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_texture,
            render_texture_size,
            render_texture_bind_group,
            render_pipeline,
            render_buffer,
        }
    }

    pub fn recreate(&mut self) {
        self.resize(self.size.width, self.size.height);
    }

}

impl RendererService for WGPUContext {
    fn render(&mut self) -> Result<(), RendererError> {
        let output = self.surface.get_current_texture()
            .map_err(map_surface_to_renderer_error)?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder")
        });

        {
            let index = rand::thread_rng().next_u32() % self.render_buffer.buffer.len() as u32;
            self.render_buffer.buffer[index as usize].r = 255u8;
        }

        {
            self.queue.write_texture(
                wgpu::ImageCopyTexture {
                    texture: &&self.render_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All
                },
                self.render_buffer.as_bytes(),
                wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: std::num::NonZeroU32::new(4 * DISPLAY_WIDTH as u32),
                    rows_per_image: std::num::NonZeroU32::new(DISPLAY_HEIGHT as u32)
                },
                self.render_texture_size
            );
        }

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[
                    Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color { r: 25.0 / 255.0, g: 27.0 / 255.0, b: 43.0 / 255.0, a: 1.0 }),
                            store: true
                        }
                    })
                ],
                depth_stencil_attachment: None
            });

            // Compute viewport
            let (x, y, w, h) = {
                if self.size.width as f32 / self.size.height as f32 >= (DISPLAY_WIDTH as f32 / DISPLAY_HEIGHT as f32) {
                    let w = DISPLAY_WIDTH as f32 * self.size.height as f32 / DISPLAY_HEIGHT as f32;
                    let h = self.size.height as f32;
                    let x = (self.size.width / 2) as f32 - (w / 2.0);
                    let y = 0.0;
                    (x, y, w, h)
                } else {
                    let w = self.size.width as f32;
                    let h = DISPLAY_HEIGHT as f32 * self.size.width as f32 / DISPLAY_WIDTH as f32;
                    let x = 0.0;
                    let y = (self.size.height / 2) as f32 - (h / 2.0);
                    (x, y, w, h)
                }
            };
        
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_viewport(x, y, w, h, 0.0, 1.0);
            render_pass.set_bind_group(0, &self.render_texture_bind_group, &[]);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        if width > 0 && height > 0 {
            self.size.width = width;
            self.size.height = height;
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
        Ok(())
    }
}