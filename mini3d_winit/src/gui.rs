use mini3d_core::{
    feature::input::axis::InputAxisRange,
    glam::Vec4,
    utils::uid::{ToUID, UID},
};
use mini3d_wgpu::context::WGPUContext;
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};

use crate::{
    mapper::{Axis, Button, InputMapper, InputProfile},
    window::Window,
    DisplayMode,
};

#[derive(PartialEq, Eq)]
enum Page {
    None,
    InputConfig,
}

enum RecordSource {
    ActionButton {
        uid: UID,
        previous: Option<Button>,
    },
    AxisButton {
        uid: UID,
        previous: Option<(Button, f32)>,
    },
    AxisAxis {
        uid: UID,
        previous: Option<(Axis, f32)>,
    },
}

struct RecordRequest {
    profile: UID,
    source: RecordSource,
}

pub(crate) struct WindowGUI {
    egui_context: egui::Context,
    egui_state: egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,
    central_viewport: Vec4,
    visible: bool,
    page: Page,
    show_uid: bool,
    show_internal_name: bool,
    show_min_max: bool,
    active_profile: UID,
    profile_rename_placeholder: String,
    record_request: Option<RecordRequest>,
}

pub(crate) struct WindowControl<'a> {
    pub control_flow: &'a mut ControlFlow,
    pub display_mode: &'a mut DisplayMode,
    pub request_save: &'a mut bool,
    pub request_load: &'a mut bool,
}

impl WindowGUI {
    pub(crate) fn new(
        context: &WGPUContext,
        window: &winit::window::Window,
        event_loop: &EventLoop<()>,
        mapper: &InputMapper,
    ) -> Self {
        // Create the egui resources
        let egui_context = egui::Context::default();
        let egui_state = egui_winit::State::new(event_loop);
        let egui_renderer =
            egui_wgpu::Renderer::new(&context.device, context.config.format, None, 1);

        Self {
            egui_context,
            egui_state,
            egui_renderer,
            central_viewport: Vec4::new(
                0.0,
                0.0,
                window.inner_size().width as f32,
                window.inner_size().height as f32,
            ),
            visible: true,
            page: Page::None,
            show_uid: false,
            show_internal_name: false,
            show_min_max: false,
            active_profile: mapper.default_profile,
            profile_rename_placeholder: Default::default(),
            record_request: None,
        }
    }

    pub(crate) fn set_visible(&mut self, toggle: bool) {
        self.visible = toggle;
    }

    pub(crate) fn is_fullscreen(&self) -> bool {
        self.page != Page::None
    }

    pub(crate) fn is_recording(&self) -> bool {
        self.record_request.is_some()
    }

    pub(crate) fn handle_event<T>(
        &mut self,
        event: &Event<T>,
        mapper: &mut InputMapper,
        window: &mut Window,
    ) {
        // Handle record request
        if let Some(request) = &self.record_request {
            if let Event::WindowEvent { event, .. } = event {
                match event {
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                        ..
                    } => {
                        if *state == ElementState::Pressed && *keycode == VirtualKeyCode::Escape {
                            // Cancel recording
                            match request.source {
                                RecordSource::ActionButton { uid, previous } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .actions
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .button = previous;
                                }
                                RecordSource::AxisButton { uid, previous } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .axis
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .button = previous;
                                }
                                RecordSource::AxisAxis { uid, previous } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .axis
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .axis = previous;
                                }
                            }
                            window.set_focus(false);
                            self.record_request = None;
                        } else if *state == ElementState::Pressed {
                            match request.source {
                                RecordSource::ActionButton { uid, .. } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .actions
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .button = Some(Button::Keyboard { code: *keycode });
                                }
                                RecordSource::AxisButton { uid, .. } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .axis
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .button = Some((Button::Keyboard { code: *keycode }, 1.0));
                                }
                                _ => {}
                            }
                            window.set_focus(false);
                            self.record_request = None;
                        }
                    }
                    WindowEvent::MouseInput {
                        device_id: _,
                        state,
                        button,
                        ..
                    } => {
                        if *state == ElementState::Pressed {
                            match request.source {
                                RecordSource::ActionButton { uid, .. } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .actions
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .button = Some(Button::Mouse { button: *button });
                                }
                                RecordSource::AxisButton { uid, .. } => {
                                    mapper
                                        .profiles
                                        .get_mut(&request.profile)
                                        .unwrap()
                                        .axis
                                        .iter_mut()
                                        .find(|a| a.name.to_uid() == uid)
                                        .unwrap()
                                        .button = Some((Button::Mouse { button: *button }, 1.0));
                                }
                                _ => {}
                            }
                            window.set_focus(false);
                            self.record_request = None;
                        }
                    }
                    _ => {}
                }
            }
        } else {
            // No record, continue
            if let Event::WindowEvent { window_id, event } = event {
                if *window_id == window.handle.id() {
                    _ = self.egui_state.on_event(&self.egui_context, event);
                }
            }
        }
    }

    pub(crate) fn handle_controller_event(
        &mut self,
        event: &gilrs::EventType,
        id: gilrs::GamepadId,
        mapper: &mut InputMapper,
        window: &mut Window,
    ) {
        if let Some(request) = &self.record_request {
            match event {
                gilrs::EventType::ButtonPressed(button, _) => {
                    match request.source {
                        RecordSource::ActionButton { uid, .. } => {
                            mapper
                                .profiles
                                .get_mut(&request.profile)
                                .unwrap()
                                .actions
                                .iter_mut()
                                .find(|a| a.name.to_uid() == uid)
                                .unwrap()
                                .button = Some(Button::Controller {
                                id,
                                button: *button,
                            });
                        }
                        RecordSource::AxisButton { uid, .. } => {
                            mapper
                                .profiles
                                .get_mut(&request.profile)
                                .unwrap()
                                .axis
                                .iter_mut()
                                .find(|a| a.name.to_uid() == uid)
                                .unwrap()
                                .button = Some((
                                Button::Controller {
                                    id,
                                    button: *button,
                                },
                                1.0,
                            ));
                        }
                        _ => {}
                    }
                    window.set_focus(false);
                    self.record_request = None;
                }
                gilrs::EventType::AxisChanged(ax, value, _) => {
                    if f32::abs(*value) > 0.6 {
                        if let RecordSource::AxisAxis { uid, .. } = request.source {
                            let axis = &mut mapper
                                .profiles
                                .get_mut(&request.profile)
                                .unwrap()
                                .axis
                                .iter_mut()
                                .find(|a| a.name.to_uid() == uid)
                                .unwrap();
                            let scale: f32 = if *value > 0.0 { 1.0 } else { -1.0 };
                            axis.axis = Some((Axis::Controller { id, axis: *ax }, scale));
                        }
                        window.set_focus(false);
                        self.record_request = None;
                    }
                }
                _ => {}
            }
        }
    }

    pub(crate) fn central_viewport(&self) -> Vec4 {
        self.central_viewport
    }

    pub(crate) fn ui(
        &mut self,
        window: &mut Window,
        mapper: &mut InputMapper,
        control: &mut WindowControl,
    ) {
        // Begin a new frame
        let raw_input = self.egui_state.take_egui_input(&window.handle);
        self.egui_context.begin_frame(raw_input);

        let mut menu_height = 0.0;
        {
            if self.visible {
                let is_fullscreen = self.is_fullscreen();
                egui::TopBottomPanel::top("top").show(&self.egui_context, |ui| {
                    menu_height = ui.available_height();
                    egui::menu::bar(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Exit").clicked() {
                                *control.control_flow = ControlFlow::Exit;
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Input Settings").clicked() {
                                ui.close_menu();
                                self.page = Page::InputConfig;
                            }
                        });
                        ui.menu_button("State", |ui| {
                            if ui.button("Save").clicked() {
                                ui.close_menu();
                                *control.request_save = true;
                            }
                            if ui.button("Load").clicked() {
                                ui.close_menu();
                                *control.request_load = true;
                            }
                        });
                        ui.menu_button("View", |ui| {
                            if ui.button("Fullscreen").clicked() && !is_fullscreen {
                                ui.close_menu();
                                *control.display_mode = DisplayMode::FullscreenFocus;
                            }
                        });
                        ui.menu_button("Help", |ui| {
                            if ui.button("About").clicked() {
                                ui.close_menu();
                            }
                        });
                    });
                });

                if self.page == Page::InputConfig {
                    egui::CentralPanel::default().show(&self.egui_context, |ui| {
                        ui.horizontal(|ui| {
                            if ui.button("   Save & Exit   ").clicked() {
                                mapper.rebuild_cache();
                                mapper.save().ok();
                                self.page = Page::None;
                            }
                            if ui.button("   Save   ").clicked() {
                                mapper.rebuild_cache();
                                mapper.save().ok();
                            }
                            if ui.button("   Cancel   ").clicked() {
                                mapper.load().ok();
                                mapper.rebuild_cache(); // Update IDs
                                self.page = Page::None;
                            }
                            if ui.button("   Refresh   ").clicked() {
                                mapper.rebuild_cache();
                            }
                            ui.separator();
                            ui.checkbox(&mut self.show_uid, "Show UID");
                            ui.checkbox(&mut self.show_internal_name, "Show Internal Name");
                            ui.checkbox(&mut self.show_min_max, "Show Min/Max");
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            for (profile_uid, profile) in &mut mapper.profiles {
                                if self.active_profile == *profile_uid {
                                    ui.add(egui::SelectableLabel::new(
                                        true,
                                        format!("   {}   ", profile.name),
                                    ));
                                } else if ui.button(format!("   {}   ", profile.name)).clicked() {
                                    self.active_profile = if self.active_profile == *profile_uid {
                                        UID::null()
                                    } else {
                                        *profile_uid
                                    };
                                }
                            }
                            ui.separator();
                            if ui.button("Add").clicked() {
                                self.active_profile = mapper.new_profile();
                            }
                            if ui.button("Remove").clicked() && !self.active_profile.is_null() {
                                mapper.profiles.remove(&self.active_profile);
                                self.active_profile = UID::null();
                            }
                            if ui.button("Duplicate").clicked() {
                                self.active_profile = mapper.duplicate(self.active_profile);
                            }
                            ui.add(
                                egui::TextEdit::singleline(&mut self.profile_rename_placeholder)
                                    .desired_width(100.0),
                            );
                            if ui.button("Rename").clicked()
                                && !mapper
                                    .profiles
                                    .iter()
                                    .any(|(_, p)| p.name == self.profile_rename_placeholder)
                            {
                                if let Some(profile) = mapper.profiles.get_mut(&self.active_profile)
                                {
                                    if !self.profile_rename_placeholder.is_empty() {
                                        profile.name = self.profile_rename_placeholder.clone();
                                        self.profile_rename_placeholder.clear();
                                    }
                                }
                            }
                        });
                        ui.separator();

                        // Get the active input profile
                        if let Some(profile) = mapper.profiles.get_mut(&self.active_profile) {
                            // Profile section
                            ui.checkbox(&mut profile.active, "Active");
                            ui.separator();

                            // Pre-compute total scroll height
                            let mut total_height = 0.0;
                            total_height += 200.0;
                            total_height += profile.actions.len() as f32 * 30.0;
                            total_height += profile.axis.len() as f32 * 30.0;
                            egui::ScrollArea::both()
                                .auto_shrink([false, false])
                                .max_height(total_height)
                                .show_rows(ui, total_height, 1, |ui, _| {
                                    ui_input_table(
                                        profile,
                                        ui,
                                        window,
                                        &mut self.record_request,
                                        UIInputTableDescriptor {
                                            active_profile: self.active_profile,
                                            show_min_max: self.show_min_max,
                                        },
                                    );
                                });
                        }
                    });
                }
            }
        }

        // Compute central widget
        self.central_viewport = Vec4::new(
            0.0,
            menu_height,
            window.handle.inner_size().width as f32,
            window.handle.inner_size().height as f32 - menu_height,
        );
    }

    pub(crate) fn render(
        &mut self,
        window: &winit::window::Window,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        output: &wgpu::TextureView,
    ) {
        if self.visible {
            // End the frame
            let full_output = self.egui_context.end_frame();
            self.egui_state.handle_platform_output(
                window,
                &self.egui_context,
                full_output.platform_output,
            );

            // Render the UI
            let paint_jobs = self.egui_context.tessellate(full_output.shapes);

            let screen_descriptor = {
                let size = window.inner_size();
                egui_wgpu::renderer::ScreenDescriptor {
                    size_in_pixels: [size.width, size.height],
                    pixels_per_point: window.scale_factor() as f32,
                }
            };
            self.egui_renderer.update_buffers(
                device,
                queue,
                encoder,
                &paint_jobs,
                &screen_descriptor,
            );
            for (tex_id, img_delta) in full_output.textures_delta.set {
                self.egui_renderer
                    .update_texture(device, queue, tex_id, &img_delta);
            }
            for tex_id in full_output.textures_delta.free {
                self.egui_renderer.free_texture(&tex_id);
            }

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("eguirender_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: output,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            self.egui_renderer
                .render(&mut render_pass, &paint_jobs, &screen_descriptor);

            // render_state.renderer.read().fre
            // self.render_pass.add_textures(device, queue, &tdelta)
            //     .expect("Failed to add egui textures");
            // self.render_pass.update_buffers(device, queue, &clipped_primitives, &screen_descriptor);
            // self.render_pass.execute(encoder, full_output, &clipped_primitives, &screen_descriptor, None)
            //     .expect("Failed to execute egui render");
            // self.render_pass.remove_textures(tdelta)
            //     .expect("Failed to remove egui textures");
        }
    }
}

struct UIInputTableDescriptor {
    active_profile: UID,
    show_min_max: bool,
}

fn ui_input_table(
    profile: &mut InputProfile,
    ui: &mut egui::Ui,
    window: &mut Window,
    record_request: &mut Option<RecordRequest>,
    desc: UIInputTableDescriptor,
) {
    // Show actions
    ui.separator();
    ui.vertical_centered_justified(|ui| {
        ui.label(egui::RichText::new("Action").strong());
    });
    ui.separator();
    ui.push_id("actions", |ui| {
        let mut table = egui_extras::TableBuilder::new(ui);
        table = table.column(egui_extras::Column::initial(100.0)); // Name
        table = table.column(egui_extras::Column::exact(110.0)); // Button
        table
            .vscroll(false)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(egui::RichText::new("Name").strong());
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("Button").strong());
                });
            })
            .body(|mut body| {
                profile.actions.iter_mut().for_each(|action| {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(action.name.clone());
                        });
                        row.col(|ui| {
                            if ui
                                .add_sized(
                                    ui.available_size(),
                                    egui::Button::new(if let Some(button) = &action.button {
                                        match button {
                                            Button::Keyboard { code } => {
                                                format!("{:?}", code)
                                            }
                                            Button::Mouse { button } => {
                                                format!("{:?}", button)
                                            }
                                            Button::Controller { id, button } => {
                                                format!("{:?} ({})", button, id)
                                            }
                                        }
                                    } else {
                                        "".to_owned()
                                    })
                                    .stroke(egui::Stroke::NONE),
                                )
                                .context_menu(|ui| {
                                    if ui.button("Reset").clicked() {
                                        action.button = None;
                                        ui.close_menu();
                                    }
                                })
                                .clicked()
                            {
                                // Record button
                                *record_request = Some(RecordRequest {
                                    profile: desc.active_profile,
                                    source: RecordSource::ActionButton {
                                        uid: action.name.to_uid(),
                                        previous: action.button,
                                    },
                                });
                                action.button = None;
                                window.set_focus(true);
                            }
                        });
                    });
                });
            });
    });

    // Show axis
    ui.separator();
    ui.vertical_centered_justified(|ui| {
        ui.label(egui::RichText::new("Axis").strong());
    });
    ui.separator();
    ui.push_id("axis", |ui| {
        let mut table = egui_extras::TableBuilder::new(ui);
        table = table.column(egui_extras::Column::initial(100.0)); // Display name
        table = table.column(egui_extras::Column::exact(110.0)); // Button
        table = table.column(egui_extras::Column::exact(60.0)); // Button Value
        table = table.column(egui_extras::Column::exact(110.0)); // Axis
        table = table.column(egui_extras::Column::exact(60.0)); // Axis Scale
        if desc.show_min_max {
            table = table.column(egui_extras::Column::exact(40.0)); // Min
            table = table.column(egui_extras::Column::exact(40.0)); // Max
        }
        table
            .vscroll(false)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(egui::RichText::new("Name").strong());
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("Button").strong());
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("Value").strong());
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("Axis").strong());
                });
                header.col(|ui| {
                    ui.label(egui::RichText::new("Scale").strong());
                });
                if desc.show_min_max {
                    header.col(|ui| {
                        ui.label(egui::RichText::new("Min").strong());
                    });
                    header.col(|ui| {
                        ui.label(egui::RichText::new("Max").strong());
                    });
                }
            })
            .body(|mut body| {
                profile.axis.iter_mut().for_each(|axis| {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.label(axis.name.clone());
                        });
                        row.col(|ui| {
                            if ui
                                .add_sized(
                                    ui.available_size(),
                                    egui::Button::new(if let Some((button, _)) = &axis.button {
                                        match button {
                                            Button::Keyboard { code } => {
                                                format!("{:?}", code)
                                            }
                                            Button::Mouse { button } => {
                                                format!("{:?}", button)
                                            }
                                            Button::Controller { id, button } => {
                                                format!("{:?} ({})", button, id)
                                            }
                                        }
                                    } else {
                                        "".to_owned()
                                    })
                                    .stroke(egui::Stroke::NONE),
                                )
                                .context_menu(|ui| {
                                    if ui.button("Reset").clicked() {
                                        axis.button = None;
                                        ui.close_menu();
                                    }
                                })
                                .clicked()
                            {
                                *record_request = Some(RecordRequest {
                                    profile: desc.active_profile,
                                    source: RecordSource::AxisButton {
                                        uid: axis.name.to_uid(),
                                        previous: axis.button,
                                    },
                                });
                                axis.button = None;
                                window.set_focus(true);
                            }
                        });
                        row.col(|ui| {
                            if let Some((_, value)) = &mut axis.button {
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::DragValue::new(value).speed(0.01).fixed_decimals(3),
                                );
                            }
                        });
                        row.col(|ui| {
                            if ui
                                .add_sized(
                                    ui.available_size(),
                                    egui::Button::new(if let Some((axis, _)) = &axis.axis {
                                        match axis {
                                            Axis::MousePositionX => "Mouse Position X".to_owned(),
                                            Axis::MousePositionY => "Mouse Position Y".to_owned(),
                                            Axis::MouseMotionX => "Mouse Motion X".to_owned(),
                                            Axis::MouseMotionY => "Mouse Motion Y".to_owned(),
                                            Axis::MouseWheelX => "Mouse Wheel X".to_owned(),
                                            Axis::MouseWheelY => "Mouse Wheel Y".to_owned(),
                                            Axis::Controller { id, axis } => {
                                                format!("{:?} ({})", axis, id)
                                            }
                                        }
                                    } else {
                                        "".to_owned()
                                    })
                                    .stroke(egui::Stroke::NONE),
                                )
                                .context_menu(|ui| {
                                    if ui.button("Reset").clicked() {
                                        axis.axis = None;
                                        ui.close_menu();
                                    } else if ui.button("Mouse Position X").clicked() {
                                        axis.axis = Some((Axis::MousePositionX, 1.0));
                                        ui.close_menu();
                                    } else if ui.button("Mouse Position Y").clicked() {
                                        axis.axis = Some((Axis::MousePositionY, 1.0));
                                        ui.close_menu();
                                    } else if ui.button("Mouse Motion X").clicked() {
                                        axis.axis = Some((Axis::MouseMotionX, 1.0));
                                        ui.close_menu();
                                    } else if ui.button("Mouse Motion Y").clicked() {
                                        axis.axis = Some((Axis::MouseMotionY, 1.0));
                                        ui.close_menu();
                                    } else if ui.button("Mouse Wheel X").clicked() {
                                        axis.axis = Some((Axis::MouseWheelX, 1.0));
                                        ui.close_menu();
                                    } else if ui.button("Mouse Wheel Y").clicked() {
                                        axis.axis = Some((Axis::MouseWheelY, 1.0));
                                        ui.close_menu();
                                    }
                                })
                                .clicked()
                            {
                                *record_request = Some(RecordRequest {
                                    profile: desc.active_profile,
                                    source: RecordSource::AxisAxis {
                                        uid: axis.name.to_uid(),
                                        previous: axis.axis,
                                    },
                                });
                                axis.axis = None;
                                window.set_focus(true);
                            }
                        });
                        row.col(|ui| {
                            if let Some((_, scale)) = &mut axis.axis {
                                ui.add_sized(
                                    ui.available_size(),
                                    egui::DragValue::new(scale).speed(0.01).fixed_decimals(3),
                                );
                            }
                        });
                        if desc.show_min_max {
                            row.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(match axis.range {
                                        InputAxisRange::Clamped { min, .. } => min.to_string(),
                                        InputAxisRange::Normalized { .. } => {
                                            f32::NEG_INFINITY.to_string()
                                        }
                                        InputAxisRange::ClampedNormalized { min, .. } => {
                                            min.to_string()
                                        }
                                        InputAxisRange::Infinite => f32::NEG_INFINITY.to_string(),
                                    });
                                });
                            });
                            row.col(|ui| {
                                ui.centered_and_justified(|ui| {
                                    ui.label(match axis.range {
                                        InputAxisRange::Clamped { max, .. } => max.to_string(),
                                        InputAxisRange::Normalized { .. } => {
                                            f32::INFINITY.to_string()
                                        }
                                        InputAxisRange::ClampedNormalized { max, .. } => {
                                            max.to_string()
                                        }
                                        InputAxisRange::Infinite => f32::INFINITY.to_string(),
                                    });
                                });
                            });
                        }
                    });
                });
            });
    });
}
