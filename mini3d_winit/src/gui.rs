use egui_wgpu_backend::{RenderPass, ScreenDescriptor};
use egui_winit_platform::{Platform, PlatformDescriptor};
use mini3d::{glam::Vec4, app::App, slotmap::{KeyData, Key}, input::axis::AxisKind};
use mini3d_wgpu::context::WGPUContext;
use winit::{event::{Event, WindowEvent, KeyboardInput, ElementState, VirtualKeyCode}, event_loop::ControlFlow};

use crate::{window::Window, mapper::{InputMapper, InputConfigId, Button, Axis, InputConfig}, DisplayMode, set_display_mode};

#[derive(PartialEq, Eq)]
enum Page {
    None,
    InputConfig,
}

enum RecordKind {
    Button { previous: Option<Button> },
    Axis { previous: Option<Axis> },
}

enum RecordSource {
    Action { index: usize },
    Axis { index: usize },
}

struct RecordRequest {
    config_id: InputConfigId,
    group_index: usize,
    kind: RecordKind,
    source: RecordSource,
}

pub(crate) struct WindowGUI {
    platform: Platform,
    render_pass: RenderPass,
    central_viewport: Vec4,
    visible: bool,
    page: Page,
    show_internal_name: bool,
    show_id: bool,
    show_min_max: bool,
    active_config: InputConfigId,
    config_rename_placeholder: String,
    elspased_time: f64,
    record_request: Option<RecordRequest>,
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
            show_internal_name: false,
            show_id: false,
            show_min_max: false,
            active_config: mapper.default_config,
            config_rename_placeholder: Default::default(),
            elspased_time: 0.0,
            record_request: None,
        }
    }

    pub(crate) fn set_visible(&mut self, toggle: bool) {
        self.visible = toggle;
        self.page = Page::None;
    }

    pub(crate) fn is_fullscreen(&self) -> bool {
        self.page != Page::None
    }

    pub(crate) fn handle_event<T>(
        &mut self, 
        event: &Event<T>,
        mapper: &mut InputMapper,
        window: &mut Window,
    ) {
        // Handle record request
        if let Some(request) = &self.record_request {
            match request.kind {
                RecordKind::Button { previous } => {
                    match event {
                        Event::WindowEvent { event, .. } => {
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
                                            RecordSource::Action { index } => {
                                                mapper.configs.get_mut(request.config_id).unwrap()
                                                    .groups[request.group_index].actions[index].button = previous;
                                            },
                                            RecordSource::Axis { index } => {
                                                mapper.configs.get_mut(request.config_id).unwrap()
                                                    .groups[request.group_index].axis[index].button = previous;
                                            },
                                        }
                                        window.set_focus(false);
                                        self.record_request = None;
                                    } else if *state == ElementState::Released {
                                        match request.source {
                                            RecordSource::Action { index } => {
                                                let action = &mut mapper.configs.get_mut(request.config_id).unwrap().groups[request.group_index].actions[index];
                                                action.button = Some(Button::Keyboard { code: *keycode });
                                            },
                                            RecordSource::Axis { index } => {
                                                let axis = &mut mapper.configs.get_mut(request.config_id).unwrap().groups[request.group_index].axis[index];
                                                axis.button = Some(Button::Keyboard { code: *keycode });
                                                axis.button_value = 1.0;
                                            },
                                        }
                                        window.set_focus(false);
                                        self.record_request = None;
                                    }
                                }
                                WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                                    if *state == ElementState::Released {
                                        match request.source {
                                            RecordSource::Action { index } => {
                                                let action = &mut mapper.configs.get_mut(request.config_id).unwrap().groups[request.group_index].actions[index];
                                                action.button = Some(Button::Mouse { button: *button });
                                            },
                                            RecordSource::Axis { index } => {
                                                let axis = &mut mapper.configs.get_mut(request.config_id).unwrap().groups[request.group_index].axis[index];
                                                axis.button = Some(Button::Mouse { button: *button });
                                                axis.button_value = 1.0;
                                            },
                                        }
                                        window.set_focus(false);
                                        self.record_request = None;
                                    }
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                },
                RecordKind::Axis { .. } => {

                },
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
            match request.kind {
                RecordKind::Button { .. } => {
                    match event {
                        gilrs::EventType::ButtonPressed(button, _) => {
                            match request.source {
                                RecordSource::Action { index } => {
                                    let action = &mut mapper.configs.get_mut(request.config_id).unwrap().groups[request.group_index].actions[index];
                                    action.button = Some(Button::Controller { id, button: *button });
                                },
                                RecordSource::Axis { index } => {
                                    let axis = &mut mapper.configs.get_mut(request.config_id).unwrap().groups[request.group_index].axis[index];
                                    axis.button = Some(Button::Controller { id, button: *button });
                                    axis.button_value = 1.0;
                                },
                            }
                            window.set_focus(false);
                            self.record_request = None;
                        },
                        _ => {}
                    }
                },
                RecordKind::Axis { .. } => {},
            }
        }
    }

    pub(crate) fn central_viewport(&self) -> Vec4 {
        self.central_viewport
    }

    pub(crate) fn ui(
        &mut self, 
        window: &mut Window,
        app: &App,
        mapper: &mut InputMapper,
        control_flow: &mut ControlFlow, 
        display_mode: &mut DisplayMode,
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
                                *control_flow = ControlFlow::Exit;
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Input").clicked() {
                                ui.close_menu();
                                self.page = Page::InputConfig;
                            }
                        });
                        ui.menu_button("View", |ui| {
                            if ui.button("Fullscreen").clicked() && !self.is_fullscreen() {
                                ui.close_menu();
                                *display_mode = set_display_mode(window, self, DisplayMode::FullscreenFocus);
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
                    use egui_extras::{TableBuilder, Size};
                    egui::CentralPanel::default().show(&self.platform.context(), |ui| {
                        
                        ui.horizontal(|ui| {
                            if ui.button("   Close   ").clicked() {
                                self.page = Page::None;
                                mapper.rebuild_cache();
                            }
                            if ui.button("   Reload   ").clicked() {
                                mapper.reload(app);
                            }
                            ui.separator();
                            ui.checkbox(&mut self.show_internal_name, "Show Internal Name");
                            ui.checkbox(&mut self.show_id, "Show Debug ID");
                            ui.checkbox(&mut self.show_min_max, "Show Min/Max");
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            for (config_id, config) in &mut mapper.configs {
                                if self.active_config == config_id {
                                    ui.add(egui::SelectableLabel::new(true, format!("   {}   ", config.name)));
                                } else {
                                    if ui.button(format!("   {}   ", config.name)).clicked() {
                                        self.active_config = if self.active_config == config_id {
                                            InputConfigId::null()
                                        } else {
                                            config_id  
                                        };
                                    }
                                }
                            }
                            ui.separator();
                            if ui.button("Add").clicked() {
                                let name = format!("New Config {}", mapper.configs.len() + 1);
                                let key = mapper.configs.insert(InputConfig { name, ..Default::default() });
                                self.active_config = key;
                                mapper.reload(app);
                            }
                            if ui.button("Remove").clicked() {
                                if self.active_config != InputConfigId::null() {
                                    mapper.configs.remove(self.active_config);
                                    self.active_config = InputConfigId::null();
                                }
                            }
                            ui.add(egui::TextEdit::singleline(&mut self.config_rename_placeholder).desired_width(100.0));
                            if ui.button("Rename").clicked() {
                                if !self.config_rename_placeholder.is_empty() {
                                    if let Some(config) = mapper.configs.get_mut(self.active_config) {
                                        config.name = self.config_rename_placeholder.clone();
                                        self.config_rename_placeholder.clear();
                                    }
                                }
                            }
                        });
                        ui.separator();

                        // Get the active input configuration
                        if let Some(config) = mapper.configs.get_mut(self.active_config) {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                for (group_index, group) in config.groups.iter_mut().enumerate() {
                                    ui.collapsing(group.name.clone(), |ui| {
        
                                        // Show actions
                                        ui.separator();
                                        ui.vertical_centered_justified(|ui| { ui.label(egui::RichText::new("Action").strong()); });
                                        ui.separator();
                                        ui.push_id("actions", |ui| {
                                            let mut table = TableBuilder::new(ui);
                                            if self.show_id {
                                                table = table.column(Size::initial(100.0)); // Id
                                            }
                                            table = table.column(Size::initial(100.0)); // Name
                                            if self.show_internal_name {
                                                table = table.column(Size::initial(100.0)); // Internal Name
                                            }
                                            table = table.column(Size::exact(100.0)); // Button
                                            table = table.column(Size::remainder()); // Description
                                            table.scroll(false)
                                                .striped(true)
                                                .resizable(true)
                                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                                .header(20.0, |mut header| {
                                                    if self.show_id {
                                                        header.col(|ui| { ui.label(egui::RichText::new("Debug ID").strong()); });
                                                    }
                                                    header.col(|ui| { ui.label(egui::RichText::new("Name").strong()); });
                                                    if self.show_internal_name {
                                                        header.col(|ui| { ui.label(egui::RichText::new("Internal Name").strong()); });
                                                    }
                                                    header.col(|ui| { ui.label(egui::RichText::new("Button").strong()); });
                                                    header.col(|ui| { ui.label(egui::RichText::new("Description").strong()); });
                                                })
                                                .body(|mut body| {
                                                    for (action_index, action) in group.actions.iter_mut().enumerate() {
                                                        body.row(20.0, |mut row| {
                                                            if self.show_id {
                                                                row.col(|ui| { ui.label(KeyData::as_ffi(action.id.data()).to_string()); });
                                                            }
                                                            row.col(|ui| { ui.label(action.descriptor.display_name.clone()); });
                                                            if self.show_internal_name {
                                                                row.col(|ui| { ui.label(action.descriptor.name.clone()); });
                                                            }
                                                            row.col(|ui| { 
                                                                if ui.add_sized(ui.available_size(), egui::Button::new(
                                                                        if let Some(button) = &action.button {
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
                                                                            action.button = None;
                                                                            ui.close_menu();
                                                                        }
                                                                    })
                                                                .clicked() {
                                                                    // Record button
                                                                    self.record_request = Some(RecordRequest { 
                                                                        config_id: self.active_config,
                                                                        group_index, 
                                                                        kind: RecordKind::Button { previous: action.button }, 
                                                                        source: RecordSource::Action { index: action_index }, 
                                                                    });
                                                                    action.button = None;
                                                                    window.set_focus(true);
                                                                }
                                                            });
                                                            row.col(|ui| { ui.label(action.descriptor.description.clone()); });
                                                        });
                                                    }
                                                });
                                        });
        
                                        
                                        // Show axis
                                        ui.separator();
                                        ui.vertical_centered_justified(|ui| { ui.label(egui::RichText::new("Axis").strong()); });
                                        ui.separator();
                                        ui.push_id("axis", |ui| {
                                            let mut table = TableBuilder::new(ui);
                                            if self.show_id {
                                                table = table.column(Size::initial(100.0)); // Id
                                            }
                                            table = table.column(Size::initial(100.0)); // Display name
                                            if self.show_internal_name {
                                                table = table.column(Size::initial(100.0)); // Name
                                            }
                                            table = table.column(Size::exact(100.0)); // Button
                                            table = table.column(Size::exact(60.0)); // Button Value
                                            table = table.column(Size::exact(110.0)); // Axis
                                            table = table.column(Size::exact(60.0)); // Axis Sensibility
                                            if self.show_min_max {
                                                table = table.column(Size::exact(40.0)); // Min
                                                table = table.column(Size::exact(40.0)); // Max
                                            }
                                            table = table.column(Size::remainder()); // Description
                                            table.scroll(false)
                                                .striped(true)
                                                .resizable(true)
                                                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                                                .header(20.0, |mut header| {
                                                    if self.show_id {
                                                        header.col(|ui| { ui.label(egui::RichText::new("Debug ID").strong()); });
                                                    }
                                                    header.col(|ui| { ui.label(egui::RichText::new("Name").strong()); });
                                                    if self.show_internal_name {
                                                        header.col(|ui| { ui.label(egui::RichText::new("Internal Name").strong()); });
                                                    }
                                                    header.col(|ui| { ui.label(egui::RichText::new("Button").strong()); });
                                                    header.col(|ui| { ui.label(egui::RichText::new("Value").strong()); });
                                                    header.col(|ui| { ui.label(egui::RichText::new("Axis").strong()); });
                                                    header.col(|ui| { ui.label(egui::RichText::new("Sensibility").strong()); });
                                                    if self.show_min_max {
                                                        header.col(|ui| { ui.label(egui::RichText::new("Min").strong()); });
                                                        header.col(|ui| { ui.label(egui::RichText::new("Max").strong()); });
                                                    }
                                                    header.col(|ui| { ui.label(egui::RichText::new("Description").strong()); });
                                                })
                                                .body(|mut body| {
                                                    for (axis_index, axis) in group.axis.iter_mut().enumerate() {
                                                        body.row(20.0, |mut row| {
                                                            if self.show_id {
                                                                row.col(|ui| { ui.label(KeyData::as_ffi(axis.id.data()).to_string()); });
                                                            }
                                                            row.col(|ui| { ui.label(axis.descriptor.display_name.clone()); });
                                                            if self.show_internal_name {
                                                                row.col(|ui| { ui.label(axis.descriptor.name.clone()); });
                                                            }
                                                            row.col(|ui| { 
                                                                if ui.add_sized(ui.available_size(), egui::Button::new(
                                                                    if let Some(button) = &axis.button {
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
                                                                        axis.button = None;
                                                                        ui.close_menu();
                                                                    }
                                                                })
                                                                .clicked() {
                                                                    self.record_request = Some(RecordRequest { 
                                                                        config_id: self.active_config,
                                                                        group_index, 
                                                                        kind: RecordKind::Button { previous: axis.button }, 
                                                                        source: RecordSource::Axis { index: axis_index },
                                                                    });
                                                                    axis.button = None;
                                                                    window.set_focus(true);
                                                                }
                                                            });
                                                            row.col(|ui| {
                                                                if axis.button.is_some() {
                                                                    ui.add_sized(ui.available_size(), egui::DragValue::new(&mut axis.button_value).fixed_decimals(3));
                                                                }
                                                            });
                                                            row.col(|ui| { 
                                                                egui::ComboBox::from_id_source(format!("{}_combobox", axis.descriptor.name))
                                                                    .selected_text(
                                                                        if let Some(axis) = &axis.axis {
                                                                            match axis {
                                                                                Axis::CursorX => "Cursor X",
                                                                                Axis::CursorY => "Cursor Y",
                                                                                Axis::MotionX => "Motion X",
                                                                                Axis::MotionY => "Motion Y",
                                                                            }
                                                                        } else {
                                                                            ""
                                                                        }
                                                                    )
                                                                    .width(100.0)
                                                                    .show_ui(ui, |ui| {
                                                                        ui.selectable_value(&mut axis.axis, None, "");
                                                                        ui.selectable_value(&mut axis.axis, Some(Axis::CursorX), "Cursor X");
                                                                        ui.selectable_value(&mut axis.axis, Some(Axis::CursorY), "Cursor Y");
                                                                        ui.selectable_value(&mut axis.axis, Some(Axis::MotionX), "Motion X");
                                                                        ui.selectable_value(&mut axis.axis, Some(Axis::MotionY), "Motion Y");
                                                                    }
                                                                );
                                                            });
                                                            row.col(|ui| {
                                                                if axis.axis.is_some() {
                                                                    ui.add_sized(ui.available_size(), egui::DragValue::new(&mut axis.axis_sensibility).fixed_decimals(3));
                                                                }
                                                            });
                                                            if self.show_min_max {
                                                                row.col(|ui| {
                                                                    ui.centered_and_justified(|ui| {
                                                                        ui.label(match axis.descriptor.kind {
                                                                            AxisKind::Clamped { min, .. } => { min.to_string() },
                                                                            AxisKind::Normalized { .. } => { f32::NEG_INFINITY.to_string() },
                                                                            AxisKind::ClampedNormalized { min, .. } => { min.to_string() },
                                                                            AxisKind::Infinite => { f32::NEG_INFINITY.to_string() },
                                                                        });
                                                                    });
                                                                });
                                                                row.col(|ui| {
                                                                    ui.centered_and_justified(|ui| {
                                                                        ui.label(match axis.descriptor.kind {
                                                                            AxisKind::Clamped { max, .. } => { max.to_string() },
                                                                            AxisKind::Normalized { .. } => { f32::INFINITY.to_string() },
                                                                            AxisKind::ClampedNormalized { max, .. } => { max.to_string() },
                                                                            AxisKind::Infinite => { f32::INFINITY.to_string() },
                                                                        });
                                                                    });
                                                                });
                                                            }
                                                            row.col(|ui| { ui.add(egui::Label::new(axis.descriptor.description.clone()).wrap(false)); });
                                                        });
                                                    }
                                                });
                                        });
                                    });
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