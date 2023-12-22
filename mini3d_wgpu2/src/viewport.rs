use glam::Vec2;

use crate::context::WGPUContext;

#[derive(Debug, Clone, Copy)]
pub enum ViewportMode {
    Fixed(f32),
    FixedBestFit,
    StretchKeepAspect,
    Stretch,
}

pub struct ViewportExtent {
    mode: ViewportMode,
    screen: (u16, u16),                    // Size of the core engine render target
    global_viewport: (u32, u32, u32, u32), // Global viewport
    viewport: (f32, f32, f32, f32),        // Final viewport
}

impl ViewportExtent {
    fn update(&mut self) {
        let global_pos =
            Vec2::new(self.global_viewport.0 as f32, self.global_viewport.1 as f32).floor();
        let global_size =
            Vec2::new(self.global_viewport.2 as f32, self.global_viewport.3 as f32).floor();

        let screen_aspect_ratio = self.screen.0 as f32 / self.screen.1 as f32;

        let size = match self.mode {
            ViewportMode::Fixed(factor) => {
                (factor * self.screen.0 as f32, factor * self.screen.1 as f32)
            }
            ViewportMode::FixedBestFit => {
                let w_factor = global_size.x / self.screen.0 as f32;
                let h_factor = global_size.y / self.screen.1 as f32;
                let min = f32::floor(w_factor.min(h_factor)).max(1.0);
                (min * self.screen.0 as f32, min * self.screen.1 as f32)
            }
            ViewportMode::StretchKeepAspect => {
                if global_size.x / global_size.y >= screen_aspect_ratio {
                    let w = global_size.y * screen_aspect_ratio;
                    let h = global_size.y;
                    (w.floor(), h.floor())
                } else {
                    let w = global_size.x;
                    let h = global_size.x / screen_aspect_ratio;
                    (w.floor(), h.floor())
                }
            }
            ViewportMode::Stretch => (global_size.x, global_size.y),
        };

        let x = (global_size.x / 2.0) - (size.0 / 2.0);
        let y = (global_size.y / 2.0) - (size.1 / 2.0);
        self.viewport = (global_pos.x + x, global_pos.y + y, size.0, size.1);
    }

    pub fn new(mode: ViewportMode, screen: (u16, u16), viewport: (u32, u32, u32, u32)) -> Self {
        let mut viewport = Self {
            mode,
            screen,
            global_viewport: viewport,
            viewport: (0.0, 0.0, 0.0, 0.0),
        };
        viewport.update();
        viewport
    }

    pub fn set_mode(&mut self, mode: ViewportMode) {
        self.mode = mode;
        self.update();
    }

    pub fn set_global_viewport(&mut self, posx: u32, posy: u32, width: u32, height: u32) {
        self.global_viewport = (posx, posy, width, height);
        self.update();
    }

    pub fn set_screen_size(&mut self, width: u16, height: u16) {
        self.screen = (width, height);
        self.update();
    }

    pub fn cursor_position(&self, pos: (f32, f32)) -> (f32, f32) {
        let relative_position =
            Vec2::new(pos.0, pos.1) - Vec2::new(self.viewport.0, self.viewport.1);
        let final_position = (relative_position / Vec2::new(self.viewport.2, self.viewport.3))
            * Vec2::new(self.viewport.2, self.viewport.3);
        (final_position.x, final_position.y)
    }
}

pub(crate) const VIEWPORT_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const VIEWPORT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

fn create_color_view(context: &WGPUContext, extent: &wgpu::Extent3d) -> wgpu::TextureView {
    let color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: VIEWPORT_COLOR_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("viewport_color_texture"),
    });
    color_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_depth_view(context: &WGPUContext, extent: &wgpu::Extent3d) -> wgpu::TextureView {
    let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: VIEWPORT_DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("viewport_depth_texture"),
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) struct Viewport {
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) camera: Option<SceneCameraProviderHandle>,
}

impl Viewport {
    pub(crate) fn new(context: &WGPUContext, resolution: UVec2) -> Self {
        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };

        Self {
            extent,
            color_view: create_color_view(context, &extent),
            depth_view: create_depth_view(context, &extent),
            camera: None,
        }
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        self.extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1,
        };
        self.color_view = create_color_view(context, &self.extent);
        self.depth_view = create_depth_view(context, &self.extent);
    }

    pub(crate) fn aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }
}
