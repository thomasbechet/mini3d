use winit::{event_loop::EventLoop, dpi::PhysicalSize, window::{WindowBuilder, CursorGrabMode, Fullscreen}};

pub(crate) struct Window {
    pub(crate) handle: winit::window::Window,
    fullscreen: bool,
    focus: bool,
}

impl Window {

    pub(crate) fn new(event_loop: &EventLoop<()>) -> Self {
        let window = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1024 - 30, 640))
            .with_resizable(true)
            .build(event_loop)
            .unwrap();
        if let Some(monitor) = window.current_monitor() {
            let screen_size = monitor.size();
            let window_size = window.outer_size();
            window.set_outer_position(winit::dpi::PhysicalPosition {
                x: screen_size.width.saturating_sub(window_size.width) as f64 / 2.
                    + monitor.position().x as f64,
                y: screen_size.height.saturating_sub(window_size.height) as f64 / 2.
                    + monitor.position().y as f64,
            });
        }

        Self { handle: window, fullscreen: false, focus: false }
    }

    pub(crate) fn set_focus(&mut self, toggle: bool) {
        self.handle.set_cursor_grab(if toggle { CursorGrabMode::Confined } else { CursorGrabMode::None })
            .expect("Failed to change cursor mode");
        self.handle.set_cursor_visible(!toggle);
        self.focus = toggle;
    }

    pub(crate) fn is_focus(&self) -> bool {
        self.focus
    }

    pub(crate) fn set_fullscreen(&mut self, toggle: bool) {
        self.handle.set_fullscreen(if toggle {
            Some(Fullscreen::Borderless(None))
        } else {
            None
        });
        self.fullscreen = toggle;
    }
}