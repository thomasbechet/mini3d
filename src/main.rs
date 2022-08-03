use mini3d_core::app::App;
use mini3d_wgpu::WGPUContext;
use mini3d_winit::WinitContext;

fn main() {
    let winit_context = WinitContext::new();
    let wgpu_context = WGPUContext::new(&winit_context.window);
    let app = App::new();
    winit_context.run(app, wgpu_context);
}
