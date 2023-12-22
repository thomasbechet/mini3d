use glam::Vec2;

#[derive(Debug, Clone, Copy)]
pub enum ViewportMode {
    Fixed(f32),
    FixedBestFit,
    StretchKeepAspect,
    Stretch,
}

pub struct Viewport {
    mode: ViewportMode,             // Viewport mode inside the extent
    screen: (u16, u16),             // Size of the core simulation render target
    extent: (u32, u32, u32, u32),   // Global allowed viewport
    viewport: (f32, f32, f32, f32), // Final viewport in global coordinates
}

impl Viewport {
    fn update(&mut self) {
        let global_pos = Vec2::new(self.extent.0 as f32, self.extent.1 as f32).floor();
        let global_size = Vec2::new(self.extent.2 as f32, self.extent.3 as f32).floor();

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
            extent: viewport,
            viewport: (0.0, 0.0, 0.0, 0.0),
        };
        viewport.update();
        viewport
    }

    pub fn set_mode(&mut self, mode: ViewportMode) {
        self.mode = mode;
        self.update();
    }

    pub fn set_extent(&mut self, posx: u32, posy: u32, width: u32, height: u32) {
        self.extent = (posx, posy, width, height);
        self.update();
    }

    pub fn set_screen_size(&mut self, width: u16, height: u16) {
        self.screen = (width, height);
        self.update();
    }

    pub fn extent(&self) -> (u32, u32, u32, u32) {
        self.extent
    }

    pub fn cursor_position(&self, pos: (f32, f32)) -> (f32, f32) {
        let relative_position =
            Vec2::new(pos.0, pos.1) - Vec2::new(self.viewport.0, self.viewport.1);
        let final_position = (relative_position / Vec2::new(self.viewport.2, self.viewport.3))
            * Vec2::new(self.viewport.2, self.viewport.3);
        (final_position.x, final_position.y)
    }
}
