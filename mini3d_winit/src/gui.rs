use egui::FontDefinitions;
use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use mini3d::glam::Vec4;
use mini3d_wgpu::context::WGPUContext;
use winit::{event::Event, event_loop::ControlFlow};

use crate::window::Window;

pub(crate) struct WindowGUI {
    platform: Platform,
    render_pass: RenderPass,
    central_viewport: Vec4,
    visible: bool,
}

impl WindowGUI {

    pub(crate) fn new(context: &WGPUContext, window: &winit::window::Window) -> Self {
        Self {
            platform: Platform::new(PlatformDescriptor {
                physical_width: window.inner_size().width,
                physical_height: window.inner_size().height,
                scale_factor: window.scale_factor(),
                font_definitions: FontDefinitions::default(),
                style: Default::default(),
            }),
            render_pass: RenderPass::new(
                &context.device, 
                context.config.format, 
                1
            ),
            central_viewport: Vec4::new(0.0, 0.0, window.inner_size().width as f32, window.inner_size().height as f32),
            visible: true,
        }
    }

    pub(crate) fn set_visible(&mut self, toggle: bool) {
        self.visible = toggle;
    }

    pub(crate) fn handle_event<T>(&mut self, winit_event: &Event<T>) {
        self.platform.handle_event(winit_event);
    }

    pub(crate) fn central_viewport(&self) -> Vec4 {
        self.central_viewport
    }

    pub(crate) fn ui(
        &mut self, 
        window: &mut Window, 
        control_flow: &mut ControlFlow, 
        delta_time: f64
    ) {
        self.platform.begin_frame();
        self.platform.update_time(delta_time);

        let mut menu_height = 0.0;
        {
            if self.visible {
                egui::TopBottomPanel::top("top").show(&self.platform.context(), |ui| {
                    menu_height = ui.available_height();
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Open").clicked() {}
                            if ui.button("Exit").clicked() {
                                *control_flow = ControlFlow::Exit;
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Inputs").clicked() {}
                        });
                        ui.menu_button("View", |ui| {
                            if ui.button("Fullscreen").clicked() {
                                window.set_fullscreen(true);
                                self.set_visible(false);
                            }
                        });
                        ui.menu_button("Help", |ui| {
                            if ui.button("Version").clicked() {}
                        });
                    }); 
                });
            }
        }

        // Compute central widget
        self.central_viewport = Vec4::new(
            0.0, menu_height as f32,
            window.handle.inner_size().width as f32,
            window.handle.inner_size().height as f32 - menu_height as f32,
        );
    }

    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device, 
        queue: &wgpu::Queue, 
        encoder: &mut wgpu::CommandEncoder, 
        output: &wgpu::TextureView
    ) {
        if self.visible {
            let full_output = self.platform.end_frame(None);
            let paint_jobs = self.platform.context().tessellate(full_output.shapes);
            let tdelta = full_output.textures_delta;
            let screen_descriptor = ScreenDescriptor {
                physical_width: window.inner_size().width as u32,
                physical_height: window.inner_size().height as u32,
                scale_factor: window.scale_factor() as f32,
            };
            self.render_pass.add_textures(device, queue, &tdelta)
                .expect("Failed to add egui textures");
            self.render_pass.update_buffers(device, queue, &paint_jobs, &screen_descriptor);
            self.render_pass.execute(encoder, output, &paint_jobs, &screen_descriptor, None)
                .expect("Failed to execute egui render");
            self.render_pass.remove_textures(tdelta)
                .expect("Failed to remove egui textures");
        }
    }
}