use futures::executor;
use wgpu::{RequestAdapterOptions, PowerPreference};

pub struct WGPUContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface,
}

impl WGPUContext {
    pub(crate) fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W) -> Self {
        
        let backend = wgpu::util::backend_bits_from_env().unwrap_or_else(wgpu::Backends::all);

        // Create instance
        let instance = wgpu::Instance::new(backend);
        
        // Create surface
        let surface = unsafe { instance.create_surface(window) };

        // Build the adaptor based on backend environment
        let adapter = executor::block_on(
            instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            // wgpu::util::initialize_adapter_from_env_or_default(&instance, backend, Some(&surface))
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
            &wgpu::DeviceDescriptor { features, limits, label: None },
            None,
        ))
        .expect("Failed to find suitable GPU adapter");

        println!("Supported formats: ");
        for format in surface.get_supported_formats(&adapter) {
            println!("- {:?}", format);
        }

        // Configure the surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter).first().unwrap().to_owned(),
            // format: wgpu::TextureFormat::Rgb10a2Unorm,
            width: 1600,
            height: 900,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

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