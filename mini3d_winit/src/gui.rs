use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use mini3d::{glam::Vec4, app::App, slotmap::Key, uid::UID, feature::asset::{input_table::InputTable, input_action::InputAction, input_axis::{InputAxis, InputAxisRange}}};
use mini3d_wgpu::context::WGPUContext;
use winit::{event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, event_loop::ControlFlow};

use crate::{window::Window, mapper::{InputMapper, InputProfileId, Button, Axis, InputProfile}, DisplayMode, set_display_mode};

#[derive(PartialEq, Eq)]
enum Page {
    None,
    InputConfig,
}

enum RecordSource {
    ActionButton { uid: UID, previous: Option<Button> },
    AxisButton { uid: UID, previous: Option<(Button, f32)> },
    AxisAxis { uid: UID, previous: Option<(Axis, f32)> },
}

struct RecordRequest {
    profile_id: InputProfileId,
    source: RecordSource,
}

pub(crate) struct WindowGUI {
    platform: Platform,
    render_pass: RenderPass,
    central_viewport: Vec4,
    visible: bool,
    page: Page,
    show_uid: bool,
    show_internal_name: bool,
    show_min_max: bool,
    active_profile: InputProfileId,
    profile_rename_placeholder: String,
    elspased_time: f64,
    record_request: Option<RecordRequest>,
}

pub(crate) struct WindowControl<'a> {
    pub control_flow: &'a mut ControlFlow, 
    pub display_mode: &'a mut DisplayMode,
    pub request_save: &'a mut bool,
    pub request_load: &'a mut bool,
}

impl WindowGUI {

    pub(crate) fn new(context: &WGPUContext, window: &winit::window::Window, mapper: &InputMapper) -> Self {
        Self {
            platform: Platform::new(PlatformDescriptor {
                physical_width: window.inner_size().width,
                physical_height: window.inner_size().height,
                scale_factor: window.scale_factor(),
                font_definitions: egui::FontDefinitions::default(),
                style: Default::default(),
            }),
            render_pass: RenderPass::new(
                &context.device, 
                context.config.format, 
                1
            ),
            central_viewport: Vec4::new(0.0, 0.0, window.inner_size().width as f32, window.inner_size().height as f32),
            visible: true,
            page: Page::None,
            show_uid: false,
            show_internal_name: false,
            show_min_max: false,
            active_profile: mapper.default_profile,
            profile_rename_placeholder: Default::default(),
            elspased_time: 0.0,
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
                        input: KeyboardInput {
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
                                    mapper.profiles.get_mut(request.profile_id).unwrap()
                                        .actions.get_mut(&uid).unwrap().button = previous;
                                },
                                RecordSource::AxisButton { uid, previous } => {
                                    mapper.profiles.get_mut(request.profile_id).unwrap()
                                        .axis.get_mut(&uid).unwrap().button = previous;
                                },
                                RecordSource::AxisAxis { uid, previous } => {
                                    mapper.profiles.get_mut(request.profile_id).unwrap()
                                        .axis.get_mut(&uid).unwrap().axis = previous;
                                },
                            }
                            window.set_focus(false);
                            self.record_request = None;
                        } else if *state == ElementState::Pressed {
                            match request.source {
                                RecordSource::ActionButton { uid, .. } => {
                                    let action = &mut mapper.profiles.get_mut(request.profile_id).unwrap().actions.get_mut(&uid).unwrap();
                                    action.button = Some(Button::Keyboard { code: *keycode });
                                },
                                RecordSource::AxisButton { uid, .. } => {
                                    let axis = mapper.profiles.get_mut(request.profile_id).unwrap().axis.get_mut(&uid).unwrap();
                                    axis.button = Some((Button::Keyboard { code: *keycode }, 1.0));
                                },
                                _ => {}
                            }
                            window.set_focus(false);
                            self.record_request = None;
                        }
                    }
                    WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                        if *state == ElementState::Pressed {
                            match request.source {
                                RecordSource::ActionButton { uid, .. } => {
                                    let action = &mut mapper.profiles.get_mut(request.profile_id).unwrap().actions.get_mut(&uid).unwrap();
                                    action.button = Some(Button::Mouse { button: *button });
                                },
                                RecordSource::AxisButton { uid, .. } => {
                                    let axis = &mut mapper.profiles.get_mut(request.profile_id).unwrap().axis.get_mut(&uid).unwrap();
                                    axis.button = Some((Button::Mouse { button: *button }, 1.0));
                                },
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
            self.platform.handle_event(event);
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
                            let action = &mut mapper.profiles.get_mut(request.profile_id).unwrap().actions.get_mut(&uid).unwrap();
                            action.button = Some(Button::Controller { id, button: *button });
                        },
                        RecordSource::AxisButton { uid, .. } => {
                            let axis = &mut mapper.profiles.get_mut(request.profile_id).unwrap().axis.get_mut(&uid).unwrap();
                            axis.button = Some((Button::Controller { id, button: *button }, 1.0));
                        },
                        _ => {}
                    }
                    window.set_focus(false);
                    self.record_request = None;
                },
                gilrs::EventType::AxisChanged(ax, value, _) => {
                    if f32::abs(*value) > 0.6 {
                        if let RecordSource::AxisAxis { uid, .. } = request.source {
                            let axis = &mut mapper.profiles.get_mut(request.profile_id).unwrap().axis.get_mut(&uid).unwrap();
                            let scale: f32 = if *value > 0.0 { 1.0 } else { -1.0 };
                            axis.axis = Some((Axis::Controller { id, axis: *ax }, scale));
                        }
                        window.set_focus(false);
                        self.record_request = None;
                    }
                },
                _ => {}
            }
        }
    }

    pub(crate) fn central_viewport(&self) -> Vec4 {
        self.central_viewport
    }

    fn ui_input_table(
        &mut self,
        input_table: &InputTable,
        profile: &mut InputProfile,
        ui: &mut egui::Ui,
        app: &App,
        window: &mut Window,
    ) {
        ui.collapsing(input_table.display_name.clone(), |ui| {
            
            // Show actions
            ui.separator();
            ui.vertical_centered_justified(|ui| { ui.label(egui::RichText::new("Action").strong()); });
            ui.separator();
            ui.push_id("actions", |ui| {
                let mut table = egui_extras::TableBuilder::new(ui);
                if self.show_uid {
                    table = table.column(egui_extras::Size::initial(120.0)); // UID
                }
                if self.show_internal_name {
                    table = table.column(egui_extras::Size::initial(150.0)); // Internal Name
                }
                table = table.column(egui_extras::Size::initial(100.0)); // Name
                table = table.column(egui_extras::Size::exact(110.0)); // Button
                table = table.column(egui_extras::Size::remainder()); // Description
                table.scroll(false)
                    .clip(false)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .header(20.0, |mut header| {
                        if self.show_uid {
                            header.col(|ui| { ui.label(egui::RichText::new("UID").strong()); });
                        }
                        if self.show_internal_name {
                            header.col(|ui| { ui.label(egui::RichText::new("Internal Name").strong()); });
                        }
                        header.col(|ui| { ui.label(egui::RichText::new("Name").strong()); });
                        header.col(|ui| { ui.label(egui::RichText::new("Button").strong()); });
                        header.col(|ui| { ui.label(egui::RichText::new("Description").strong()); });
                    })
                    .body(|mut body| {
                        input_table.actions.iter().for_each(|uid| {
                            let action = app.asset.entry::<InputAction>(*uid).expect("Action UID not found");
                            if let Some(map_action) = profile.actions.get_mut(uid) {
                                body.row(20.0, |mut row| {
                                    if self.show_uid {
                                        row.col(|ui| { ui.label(uid.to_string()); });
                                    }
                                    if self.show_internal_name {
                                        row.col(|ui| { ui.label(action.name.to_string()); });
                                    }
                                    row.col(|ui| { ui.label(action.asset.display_name.clone()); });
                                    row.col(|ui| { 
                                        if ui.add_sized(ui.available_size(), egui::Button::new(
                                                if let Some(button) = &map_action.button {
                                                    match button {
                                                        Button::Keyboard { code } => format!("{:?}", code),
                                                        Button::Mouse { button } => format!("{:?}", button),
                                                        Button::Controller { id, button } => format!("{:?} ({})", button, id),
                                                    }
                                                } else {
                                                    "".to_owned()
                                                }
                                            )
                                            .stroke(egui::Stroke::none()))
                                            .context_menu(|ui| {
                                                if ui.button("Reset").clicked() {
                                                    map_action.button = None;
                                                    ui.close_menu();
                                                }
                                            })
                                        .clicked() {
                                            // Record button
                                            self.record_request = Some(RecordRequest {
                                                profile_id: self.active_profile,
                                                source: RecordSource::ActionButton { uid: *uid, previous: map_action.button }, 
                                            });
                                            map_action.button = None;
                                            window.set_focus(true);
                                        }
                                    });
                                    row.col(|ui| { ui.label(action.asset.description.clone()); });
                                });
                            }
                        });
                    });
            });
        
            // Show axis
            ui.separator();
            ui.vertical_centered_justified(|ui| { ui.label(egui::RichText::new("Axis").strong()); });
            ui.separator();
            ui.push_id("axis", |ui| {
                let mut table = egui_extras::TableBuilder::new(ui);
                if self.show_uid {
                    table = table.column(egui_extras::Size::initial(120.0)); // UID
                }
                if self.show_internal_name {
                    table = table.column(egui_extras::Size::initial(150.0)); // Internal Name
                }
                table = table.column(egui_extras::Size::initial(100.0)); // Display name
                table = table.column(egui_extras::Size::exact(110.0)); // Button
                table = table.column(egui_extras::Size::exact(60.0)); // Button Value
                table = table.column(egui_extras::Size::exact(110.0)); // Axis
                table = table.column(egui_extras::Size::exact(60.0)); // Axis Scale
                if self.show_min_max {
                    table = table.column(egui_extras::Size::exact(40.0)); // Min
                    table = table.column(egui_extras::Size::exact(40.0)); // Max
                }
                table = table.column(egui_extras::Size::remainder()); // Description
                table.scroll(false)
                    .clip(false)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .header(20.0, |mut header| {
                        if self.show_uid {
                            header.col(|ui| { ui.label(egui::RichText::new("UID").strong()); });
                        }
                        if self.show_internal_name {
                            header.col(|ui| { ui.label(egui::RichText::new("Internal Name").strong()); });
                        }
                        header.col(|ui| { ui.label(egui::RichText::new("Name").strong()); });
                        header.col(|ui| { ui.label(egui::RichText::new("Button").strong()); });
                        header.col(|ui| { ui.label(egui::RichText::new("Value").strong()); });
                        header.col(|ui| { ui.label(egui::RichText::new("Axis").strong()); });
                        header.col(|ui| { ui.label(egui::RichText::new("Scale").strong()); });
                        if self.show_min_max {
                            header.col(|ui| { ui.label(egui::RichText::new("Min").strong()); });
                            header.col(|ui| { ui.label(egui::RichText::new("Max").strong()); });
                        }
                        header.col(|ui| { ui.label(egui::RichText::new("Description").strong()); });
                    })
                    .body(|mut body| {
                        input_table.axis.iter().for_each(|uid| {
                            let axis = app.asset.entry::<InputAxis>(*uid).expect("Axis UID not found");
                            if let Some(map_axis) = profile.axis.get_mut(uid) {
                                body.row(20.0, |mut row| {
                                    if self.show_uid {
                                        row.col(|ui| { ui.label(uid.to_string()); });
                                    }
                                    if self.show_internal_name {
                                        row.col(|ui| { ui.label(axis.name.to_string()); });
                                    }
                                    row.col(|ui| { ui.label(axis.asset.display_name.clone()); });
                                    row.col(|ui| {
                                        if ui.add_sized(ui.available_size(), egui::Button::new(
                                            if let Some((button, _)) = &map_axis.button {
                                                match button {
                                                    Button::Keyboard { code } => format!("{:?}", code),
                                                    Button::Mouse { button } => format!("{:?}", button),
                                                    Button::Controller { id, button } => format!("{:?} ({})", button, id),
                                                }
                                            } else {
                                                "".to_owned()
                                            }
                                        ).stroke(egui::Stroke::none()))
                                        .context_menu(|ui| {
                                            if ui.button("Reset").clicked() {
                                                map_axis.button = None;
                                                ui.close_menu();
                                            }
                                        })
                                        .clicked() {
                                            self.record_request = Some(RecordRequest { 
                                                profile_id: self.active_profile,
                                                source: RecordSource::AxisButton { uid: *uid, previous: map_axis.button },
                                            });
                                            map_axis.button = None;
                                            window.set_focus(true);
                                        }
                                    });
                                    row.col(|ui| {
                                        if let Some((_, value)) = &mut map_axis.button {
                                            ui.add_sized(ui.available_size(), egui::DragValue::new(value).speed(0.01).fixed_decimals(3));
                                        }
                                    });
                                    row.col(|ui| {
    
                                        if ui.add_sized(ui.available_size(), egui::Button::new(
                                            if let Some((axis, _)) = &map_axis.axis {
                                                match axis {
                                                    Axis::MousePositionX => { "Mouse Position X".to_owned() },
                                                    Axis::MousePositionY => { "Mouse Position Y".to_owned() },
                                                    Axis::MouseMotionX => { "Mouse Motion X".to_owned() },
                                                    Axis::MouseMotionY => { "Mouse Motion Y".to_owned() },
                                                    Axis::Controller { id, axis } => { format!("{:?} ({})", axis, id) },
                                                }
                                            } else {
                                                "".to_owned()
                                            }
                                        ).stroke(egui::Stroke::none()))
                                        .context_menu(|ui| {
                                            if ui.button("Reset").clicked() {
                                                map_axis.axis = None;
                                                ui.close_menu();
                                            } else if ui.button("Mouse Position X").clicked() {
                                                map_axis.axis = Some((Axis::MousePositionX, 1.0));
                                                ui.close_menu();
                                            } else if ui.button("Mouse Position Y").clicked() {
                                                map_axis.axis = Some((Axis::MousePositionY, 1.0));
                                                ui.close_menu();
                                            } else if ui.button("Mouse Motion X").clicked() {
                                                map_axis.axis = Some((Axis::MouseMotionX, 1.0));
                                                ui.close_menu();
                                            } else if ui.button("Mouse Motion Y").clicked() {
                                                map_axis.axis = Some((Axis::MouseMotionY, 1.0));
                                                ui.close_menu();
                                            }
                                        })
                                        .clicked() {
                                            self.record_request = Some(RecordRequest { 
                                                profile_id: self.active_profile,
                                                source: RecordSource::AxisAxis { uid: *uid, previous: map_axis.axis },
                                            });
                                            map_axis.axis = None;
                                            window.set_focus(true);
                                        }
                                    });
                                    row.col(|ui| {
                                        if let Some((_, scale)) = &mut map_axis.axis {
                                            ui.add_sized(ui.available_size(), egui::DragValue::new(scale).speed(0.01).fixed_decimals(3));
                                        }
                                    });
                                    if self.show_min_max {
                                        row.col(|ui| {
                                            ui.centered_and_justified(|ui| {
                                                ui.label(match axis.asset.range {
                                                    InputAxisRange::Clamped { min, .. } => { min.to_string() },
                                                    InputAxisRange::Normalized { .. } => { f32::NEG_INFINITY.to_string() },
                                                    InputAxisRange::ClampedNormalized { min, .. } => { min.to_string() },
                                                    InputAxisRange::Infinite => { f32::NEG_INFINITY.to_string() },
                                                });
                                            });
                                        });
                                        row.col(|ui| {
                                            ui.centered_and_justified(|ui| {
                                                ui.label(match axis.asset.range {
                                                    InputAxisRange::Clamped { max, .. } => { max.to_string() },
                                                    InputAxisRange::Normalized { .. } => { f32::INFINITY.to_string() },
                                                    InputAxisRange::ClampedNormalized { max, .. } => { max.to_string() },
                                                    InputAxisRange::Infinite => { f32::INFINITY.to_string() },
                                                });
                                            });
                                        });
                                    }
                                    row.col(|ui| { ui.add(egui::Label::new(axis.asset.description.clone()).wrap(false)); });
                                });
                            }
                        });
                    });
            });
        });
    }

    pub(crate) fn ui(
        &mut self, 
        window: &mut Window,
        app: &App,
        mapper: &mut InputMapper,
        control: &mut WindowControl,
        delta_time: f64
    ) {
        self.platform.begin_frame();
        self.elspased_time += delta_time;
        self.platform.update_time(self.elspased_time);

        let mut menu_height = 0.0;
        {
            if self.visible {
                egui::TopBottomPanel::top("top").show(&self.platform.context(), |ui| {
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
                            if ui.button("Fullscreen").clicked() && !self.is_fullscreen() {
                                ui.close_menu();
                                *control.display_mode = set_display_mode(window, self, DisplayMode::FullscreenFocus);
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
                    egui::CentralPanel::default().show(&self.platform.context(), |ui| {
                        
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
                                mapper.refresh(app); // Update IDs
                                mapper.rebuild_cache();
                                self.page = Page::None;
                            }
                            if ui.button("   Refresh   ").clicked() {
                                mapper.refresh(app);
                            }
                            ui.separator();
                            ui.checkbox(&mut self.show_uid, "Show UID");
                            ui.checkbox(&mut self.show_internal_name, "Show Internal Name");
                            ui.checkbox(&mut self.show_min_max, "Show Min/Max");
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            for (profile_id, profile) in &mut mapper.profiles {
                                if self.active_profile == profile_id {
                                    ui.add(egui::SelectableLabel::new(true, format!("   {}   ", profile.name)));
                                } else if ui.button(format!("   {}   ", profile.name)).clicked() {
                                    self.active_profile = if self.active_profile == profile_id {
                                        InputProfileId::null()
                                    } else {
                                        profile_id
                                    };
                                }
                            }
                            ui.separator();
                            if ui.button("Add").clicked() {
                                self.active_profile = mapper.new_profile(app);
                            }
                            if ui.button("Remove").clicked() && !self.active_profile.is_null() {
                                mapper.profiles.remove(self.active_profile);
                                self.active_profile = InputProfileId::null();
                            }
                            if ui.button("Duplicate").clicked() {
                                self.active_profile = mapper.duplicate(self.active_profile, app);
                            }
                            ui.add(egui::TextEdit::singleline(&mut self.profile_rename_placeholder).desired_width(100.0));
                            if ui.button("Rename").clicked() && !mapper.profiles.iter().any(|(_, p)| p.name == self.profile_rename_placeholder) {
                                if let Some(profile) = mapper.profiles.get_mut(self.active_profile) {
                                    if !self.profile_rename_placeholder.is_empty() {
                                        profile.name = self.profile_rename_placeholder.clone();
                                        self.profile_rename_placeholder.clear();
                                    }
                                }
                            }
                        });
                        ui.separator();

                        // Get the active input profile
                        if let Some(profile) = mapper.profiles.get_mut(self.active_profile) {

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
                                    for (_, entry) in app.asset.iter::<InputTable>().unwrap() {
                                        self.ui_input_table(&entry.asset, profile, ui, app, window);
                                    }
                                });
                        }
                    });
                }
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