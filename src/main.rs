use futures::executor::block_on;
use nucleus_wgpu::{Renderer, RendererError};
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use winit_input_helper::WinitInputHelper;
use nucleus_app::App;

pub async fn run() {
    // Window setup
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(600, 400))
        .with_resizable(true)
        .build(&event_loop)
        .unwrap();
    let mut input = WinitInputHelper::new();

    let mut renderer = Renderer::new(&window).await;

    
}

fn main() {
    App::new(PlatformProvider {

    }).run();
        .with_window(WinitWindow::new())
        .with_renderer(WGPURenderer::new())
        .run();

    block_on(run());
}
