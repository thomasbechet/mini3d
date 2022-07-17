use winit::{window, event_loop};
use input_helper::WinitInputHelper;

use nucleus_window::Window;
use nucleus_runner::Runner;

pub struct WinitRunner {
    window: window::Window,
    event_loop: event_loop::EventLoop,
    input_helper: WinitInputHelper,
}

impl WinitRunner {
    fn new() -> Self {
        WinitRunner { event_loop::EventLoop::new() }
    }
}

impl Runner for WinitRunner {
    fn invoke(&mut self, app: App) {
        // Event loop
        self.event_loop.run(move |event, _, control_flow| {
            // Handle inputs
            if self.input.update(&event) && self.input.key_pressed(VirtualKeyCode::Escape) {
                *control_flow = ControlFlow::Exit;
            }

            // Match window events
            match event {
                Event::WindowEvent { window_id, event } => {
                    if window_id == window.id() {
                        match event {
                            WindowEvent::CloseRequested => {
                                *control_flow = ControlFlow::Exit;
                            }
                            // WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            //     renderer.resize(*new_inner_size);
                            // }
                            // WindowEvent::Resized(_) => {
                            //     renderer.resize(window.inner_size());
                            // }
                            _ => {}
                        }
                    }
                }
                // Event::RedrawRequested(window_id) => {
                //     if window_id == window.id() {
                //         renderer.update();
                //         match renderer.render() {
                //             Ok(_) => {}
                //             Err(RendererError::Lost) => renderer.recreate(),
                //             Err(RendererError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                //             Err(e) => eprintln!("{:?}", e),
                //         }
                //     }
                // }
                Event::MainEventsCleared => {
                    window.request_redraw();
                }
                _ => {}
            }
        });
    }
}