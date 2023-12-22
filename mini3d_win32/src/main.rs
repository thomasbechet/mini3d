use std::time::Duration;
use std::{cell::RefCell, rc::Rc};

use helper::Win32ControlHandle;
use input::Win32InputProvider;
use mini3d_core::simulation::{Simulation, SimulationConfig};
use mini3d_input::mapper::InputMapper;
use mini3d_wgpu::renderer::WGPURenderer;
use native_windows_derive as nwd;
use native_windows_gui as nwg;

use nwd::NwgUi;
use nwg::NativeUi;

use crate::helper::{get_window_long, toggle_fullscreen};

pub mod helper;
pub mod input;
pub mod mouse;

pub struct AppData {
    input_mapper: Win32InputProvider,
    renderer: Option<WGPURenderer>,
    simulation: Option<Simulation>,
    mouse_grab: bool,
}

impl Default for AppData {
    fn default() -> Self {
        let input_mapper = Win32InputProvider::new(Rc::new(RefCell::new(InputMapper::new())));
        let simulation = Simulation::new(SimulationConfig::default());
        Self {
            input_mapper,
            renderer: None,
            simulation: Some(simulation),
            mouse_grab: false,
        }
    }
}

#[derive(Default, NwgUi)]
pub struct App {
    #[nwg_control(size: (600, 400), position: (300, 300), title: "mini3d", flags: "WINDOW|VISIBLE|RESIZABLE")]
    #[nwg_events(
        OnInit: [App::on_init],
        OnKeyPress: [App::on_key_press(SELF, EVT_DATA)],
        OnKeyRelease: [App::on_key_release(SELF, EVT_DATA)],
        OnMouseMove: [App::on_mouse_move],
        OnMousePress: [App::on_mouse_press(SELF, EVT)],
        OnWindowClose: [App::on_close],
    )]
    window: nwg::Window,

    #[nwg_control(text: "File")]
    menu_file: nwg::Menu,
    #[nwg_control(parent: menu_file, text: "Open")]
    menu_open_item: nwg::MenuItem,

    #[nwg_control(text: "Configuration")]
    menu_configuration: nwg::Menu,
    #[nwg_control(parent: menu_configuration, text: "Graphics")]
    menu_graphics_item: nwg::MenuItem,
    #[nwg_control(parent: menu_configuration, text: "Input")]
    menu_input_item: nwg::MenuItem,

    #[nwg_layout(parent: window, margin: [0, 0, 0, 0], spacing: 0)]
    grid: nwg::GridLayout,

    #[nwg_control(parent: Some(&data.window))]
    #[nwg_events(OnResize: [App::resize_canvas])]
    #[nwg_layout_item(layout: grid, row: 0, col: 0, margin: [0, 0, 0, 0])]
    canvas: nwg::ExternCanvas,

    #[nwg_control(parent: window, interval: Duration::from_millis(1000/60))]
    #[nwg_events(OnTimerTick: [App::animate])]
    timer: nwg::AnimationTimer,

    data: RefCell<AppData>,
}

impl App {
    unsafe fn set_menu_visible(&self, visible: bool) {
        if visible {
            self.menu_file.set_enabled(true);
            self.menu_configuration.set_enabled(true);
            winapi::um::winuser::SetMenu(
                self.window.handle.hwnd().unwrap() as _,
                self.menu_file.handle.hmenu().unwrap().0 as _,
            );
            winapi::um::winuser::SetMenu(
                self.window.handle.hwnd().unwrap() as _,
                self.menu_configuration.handle.hmenu().unwrap().0 as _,
            );
        } else {
            self.menu_file.set_enabled(false);
            self.menu_configuration.set_enabled(false);
            winapi::um::winuser::SetMenu(
                self.window.handle.hwnd().unwrap() as _,
                winapi::um::winuser::WM_NULL as _,
            );
        }
    }

    fn set_cursor_capture(&self, capture: bool) {
        unsafe {
            if capture {
                nwg::GlobalCursor::set_capture(&self.window.handle);
                winapi::um::winuser::SetCursor(winapi::um::winuser::WM_NULL as _);
                let rect = Default::default();
                winapi::um::winuser::GetWindowRect(
                    self.window.handle.hwnd().unwrap() as _,
                    &rect as *const _ as _,
                );
                winapi::um::winuser::ClipCursor(&rect);
            } else {
                nwg::GlobalCursor::release();
                nwg::GlobalCursor::set(&nwg::Cursor::default());
                winapi::um::winuser::ClipCursor(std::ptr::null());
            }
        }
    }

    fn on_init(&self) {
        let mut data = self.data.borrow_mut();
        // Initialize renderer
        let mut renderer = WGPURenderer::new(&Win32ControlHandle::new(self.canvas.handle));
        let (width, height) = self.canvas.size();
        renderer.resize(width, height);
        data.renderer = Some(renderer);
        // Start main loop
        self.timer.start();
        // Capture mouse
        self.set_cursor_capture(true);
    }

    fn animate(&self) {
        let (width, height) = self.canvas.size();
        if width > 0 && height > 0 {
            self.data
                .borrow_mut()
                .renderer
                .as_mut()
                .unwrap()
                .render()
                .expect("Failed to render");
        }
    }

    fn on_mouse_move(&self) {
        let (x, y) = nwg::GlobalCursor::position();
        if self.data.borrow().mouse_grab {
            nwg::GlobalCursor::set_position(x, y);
        }
    }

    fn on_mouse_press(&self, event: nwg::Event) {
        match event {
            nwg::Event::OnMousePress(event) => match event {
                nwg::MousePressEvent::MousePressLeftDown => {
                    println!("Left mouse button pressed");
                }
                nwg::MousePressEvent::MousePressRightDown => {
                    println!("Right mouse button pressed");
                }
                nwg::MousePressEvent::MousePressLeftUp => {
                    println!("Middle mouse button released");
                }
                nwg::MousePressEvent::MousePressRightUp => {
                    println!("Right mouse button released");
                }
            },
            _ => unreachable!(),
        }
    }

    fn on_key_press(&self, event: &nwg::EventData) {
        if event.on_key() == nwg::keys::_K {
            println!("F10 pressed");
            unsafe {
                let hwnd = self.window.handle.hwnd().unwrap();
                let fullscreen = (get_window_long(hwnd, winapi::um::winuser::GWL_STYLE)
                    & winapi::um::winuser::WS_POPUP as isize)
                    == 0;
                toggle_fullscreen(self.window.handle, fullscreen);
                self.set_menu_visible(!fullscreen);
                self.set_cursor_capture(fullscreen);
                self.window.focus();
            }
            // self.canvas.
            let mut data = self.data.borrow_mut();
            let (width, height) = self.canvas.size();
            println!("resize {} {}", width, height);
            data.renderer.as_mut().unwrap().resize(width, height);
        }
    }

    fn on_key_release(&self, event: &nwg::EventData) {}

    fn resize_canvas(&self) {
        let mut data = self.data.borrow_mut();
        let (width, height) = self.canvas.size();
        println!("resize {} {}", width, height);
        if let Some(renderer) = &mut data.renderer {
            renderer.resize(width, height);
        }
    }

    fn on_close(&self) {
        nwg::stop_thread_dispatch();
    }
}

fn main() {
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
    let _app = App::build_ui(Default::default()).expect("Failed to build UI");
    nwg::dispatch_thread_events();
}
