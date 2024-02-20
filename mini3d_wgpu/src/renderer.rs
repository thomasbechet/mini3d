use wgpu::StoreOp;

use crate::{
    context::WGPUContext,
    error::WGPURendererError,
    viewport::{Viewport, ViewportMode},
};

pub struct WGPURenderer {
    context: WGPUContext,
    viewport: Viewport,
}

pub fn srgb_to_linear(c: [f32; 3]) -> [f32; 3] {
    let f = |x: f32| -> f32 {
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c[0]), f(c[1]), f(c[2])]
}

impl WGPURenderer {
    pub fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
    ) -> Self {
        Self {
            context: WGPUContext::new(window),
            viewport: Viewport::new(ViewportMode::FixedBestFit, (800, 450), (0, 0, 800, 450)),
        }
    }

    pub fn recreate(&mut self) {
        self.context.recreate();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.context.resize(width, height);
            self.viewport.set_extent(0, 0, width, height);
        }
    }

    pub fn cursor_position(&self, x: f32, y: f32) -> (f32, f32) {
        self.viewport.cursor_position((x, y))
    }

    pub fn set_viewport_mode(&mut self, mode: ViewportMode) {
        self.viewport.set_mode(mode);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.context.device
    }

    pub fn render(&mut self) -> Result<(), WGPURendererError> {
        // Acquire next surface texture
        let output = self
            .context
            .surface
            .get_current_texture()
            .map_err(|_| WGPURendererError::SurfaceAcquisition)?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create frame encoder
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

        // Clear screen
        {
            let mut clear_color = [25.0 / 255.0, 27.0 / 255.0, 43.0 / 255.0];
            if self.context.config.format.is_srgb() {
                clear_color = srgb_to_linear(clear_color);
            }

            let mut clear_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("clear_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: clear_color[0] as f64,
                            g: clear_color[1] as f64,
                            b: clear_color[2] as f64,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Setup viewport
            let (x, y, w, h) = self.viewport.extent();
            clear_render_pass.set_viewport(x as f32, y as f32, w as f32, h as f32, 0.0, 1.0);
        }

        // Submit queue and present
        self.context.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}

impl RendererProvider for WGPURenderer {
    fn on_connect(&mut self) {
        todo!()
    }

    fn on_disconnect(&mut self) {
        todo!()
    }

    fn next_event(&mut self) -> Option<mini3d_core::renderer::event::RendererEvent> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn add_mesh(
        &mut self,
        mesh: &mini3d_core::renderer::resource::Mesh,
    ) -> Result<
        mini3d_core::renderer::provider::RendererProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        todo!()
    }

    fn remove_mesh(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn add_texture(
        &mut self,
        texture: &mini3d_core::renderer::resource::Texture,
    ) -> Result<
        mini3d_core::renderer::provider::RendererProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        todo!()
    }

    fn remove_texture(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn add_material(
        &mut self,
        desc: mini3d_core::renderer::provider::ProviderMaterialInfo,
    ) -> Result<
        mini3d_core::renderer::provider::RendererProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        todo!()
    }

    fn remove_material(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn add_transform(
        &mut self,
    ) -> Result<
        mini3d_core::renderer::provider::RendererProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        todo!()
    }

    fn remove_transform(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn update_transform(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
        mat: mini3d_core::math::mat::M4I32F16,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn add_diffuse_pass(
        &mut self,
        info: &mini3d_core::renderer::resource::diffuse::DiffusePassInfo,
    ) -> Result<
        mini3d_core::renderer::provider::RendererProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        todo!()
    }

    fn remove_diffuse_pass(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn submit_diffuse_pass(
        &mut self,
        pass: mini3d_core::renderer::provider::RendererProviderHandle,
        command: &mini3d_core::renderer::resource::diffuse::DiffusePassCommand,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn render_diffuse_pass(
        &mut self,
        pass: mini3d_core::renderer::provider::RendererProviderHandle,
        info: &mini3d_core::renderer::resource::diffuse::DiffusePassRenderInfo,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn add_canvas_pass(
        &mut self,
        info: &mini3d_core::renderer::resource::renderpass::canvas::CanvasPassInfo,
    ) -> Result<
        mini3d_core::renderer::provider::RendererProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        todo!()
    }

    fn remove_canvas_pass(
        &mut self,
        handle: mini3d_core::renderer::provider::RendererProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn submit_canvas_pass(
        &mut self,
        pass: mini3d_core::renderer::provider::RendererProviderHandle,
        command: &mini3d_core::renderer::resource::renderpass::canvas::CanvasPassCommand,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }

    fn render_canvas_pass(
        &mut self,
        pass: mini3d_core::renderer::provider::RendererProviderHandle,
        info: &mini3d_core::renderer::resource::renderpass::canvas::CanvasPassRenderInfo,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        todo!()
    }
}
