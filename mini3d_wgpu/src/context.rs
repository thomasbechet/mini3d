use futures::executor;

pub(crate) struct WGPUContext {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) config: wgpu::SurfaceConfiguration,
    pub(crate) surface: wgpu::Surface,
}

impl WGPUContext {
    pub(crate) fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W) -> Self {
        
        // Create instance
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        
        // Create surface
        let surface = unsafe { instance.create_surface(window) };
        
        // Create WGPU adapter
        let adapter = executor::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        // Request device and queue from adaptor
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

        // Configure the surface
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            // format: surface.get_supported_formats(&adapter).first().unwrap().to_owned(),
            format: wgpu::TextureFormat::Bgra8Unorm,
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