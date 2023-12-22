use futures::executor;
use wgpu::{
    Dx12Compiler, Gles3MinorVersion, InstanceDescriptor, InstanceFlags, PowerPreference,
    RequestAdapterOptions,
};

pub(crate) struct WGPUContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) surface: wgpu::Surface,
}

impl WGPUContext {
    pub(crate) fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
    ) -> Self {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

        // Create instance
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends,
            flags: InstanceFlags::default(),
            dx12_shader_compiler: Dx12Compiler::default(),
            gles_minor_version: Gles3MinorVersion::Automatic,
        });

        // Create surface
        let surface = unsafe { instance.create_surface(window) }.expect("Failed to create surface");

        // Build the adaptor based on backend environment
        let adapter = executor::block_on(
            instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            }), // wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
        )
        .expect("Failed to create adapter");

        // Display adaptor info
        let adapter_info = adapter.get_info();
        println!("Using {} ({:?})", adapter_info.name, adapter_info.backend);

        // Find features
        let mut features = adapter.features();
        features |= wgpu::Features::INDIRECT_FIRST_INSTANCE;
        features |= wgpu::Features::MULTI_DRAW_INDIRECT;

        // Find correct limit levels
        // let limits = wgpu::Limits::default();
        // let limits = wgpu::Limits::downlevel_defaults();
        let limits = wgpu::Limits::downlevel_webgl2_defaults();

        // Request device and queue from adaptor
        let (device, queue) = executor::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features,
                limits,
                label: None,
            },
            None,
        ))
        .expect("Failed to find suitable GPU adapter");

        println!("Supported formats: ");
        for format in surface.get_capabilities(&adapter).formats {
            println!("- {:?}", format);
        }

        // Configure the surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface
                .get_capabilities(&adapter)
                .formats
                .first()
                .unwrap()
                .to_owned(),
            // format: wgpu::TextureFormat::Bgra8UnormSrgb,
            // format: wgpu::TextureFormat::Bgra8Unorm,
            width: 1600,
            height: 900,
            present_mode: wgpu::PresentMode::Fifo,
            // present_mode: wgpu::PresentMode::Mailbox,
            alpha_mode: wgpu::CompositeAlphaMode::Opaque,
            view_formats: surface.get_capabilities(&adapter).formats,
        };
        surface.configure(&device, &config);

        println!("Using format: {:?}", config.format);

        Self {
            device,
            queue,
            config,
            surface,
        }
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub(crate) fn recreate(&mut self) {
        self.surface.configure(&self.device, &self.config);
    }
}
