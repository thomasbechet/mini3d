use std::{collections::HashMap, path::Path};

use mini3d::{application::{Application}, glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION, input::{range::InputName, binding::{Button, Axis}, button::ButtonState}, event::{input::{AxisEvent, InputEvent, ButtonEvent, TextEvent, CursorEvent}, system::SystemEvent}};
use mini3d_utils::{image::ImageImporter, model::ModelImporter};
use mini3d_wgpu::{compute_fixed_viewport, WGPURenderer};
use wgpu::SurfaceError;
use winit::{window, event_loop::{self, ControlFlow}, dpi::PhysicalSize, event::{Event, WindowEvent, VirtualKeyCode, ElementState}};
use winit_input_helper::WinitInputHelper;

struct WinitInput {
    pub input_helper: WinitInputHelper,
    pub button_mapping: HashMap<VirtualKeyCode, Vec<InputName>>,
}

impl WinitInput {
    pub fn new() -> Self {
        WinitInput { 
            input_helper: WinitInputHelper::new(),
            button_mapping: HashMap::from([
                (VirtualKeyCode::Z, vec![Button::UP]),
                (VirtualKeyCode::S, vec![Button::DOWN]),
                (VirtualKeyCode::D, vec![Button::RIGHT]),
                (VirtualKeyCode::Q, vec![Button::LEFT]),
                (VirtualKeyCode::M, vec![Button::SWITCH_SELECTION_MODE]),
                (VirtualKeyCode::Space, vec![Button::CLICK]),
            ])
        }
    }
}

pub struct WinitContext {
    pub window: window::Window,
    pub event_loop: event_loop::EventLoop<()>,
    input: WinitInput,
}

impl Default for WinitContext {
    fn default() -> Self {
        let event_loop = event_loop::EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1024, 640))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();
        window.set_cursor_visible(false);
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
        let input = WinitInput::new();
        Self { window, event_loop, input }
    }
}

impl WinitContext {
    pub fn run(
        mut self, 
        mut app: Application, 
        mut renderer: WGPURenderer,
    ) {
        let event_loop = self.event_loop;
        event_loop.run(move |event, _, control_flow| {

            // Handle inputs
            if self.input.input_helper.update(&event) {
                if self.input.input_helper.key_pressed(VirtualKeyCode::Escape) {
                    app.events.push_system(SystemEvent::CloseRequested);
                }
                app.events.push_input(InputEvent::Axis(AxisEvent {
                    name: Axis::CURSOR_X,
                    value: self.input.input_helper.mouse_diff().0,
                }));
                app.events.push_input(InputEvent::Axis(AxisEvent {
                    name: Axis::CURSOR_Y,
                    value: self.input.input_helper.mouse_diff().1,
                }));
            }

            // Match window events
            match event {
                Event::WindowEvent { window_id, event } => {
                    if window_id == self.window.id() {
                        match event {
                            WindowEvent::KeyboardInput {
                                input: winit::event::KeyboardInput {
                                    virtual_keycode: Some(keycode),
                                    state,
                                    ..
                                },
                                ..
                            } => {
                                let action_state = match state {
                                    ElementState::Pressed => ButtonState::Pressed,
                                    ElementState::Released => ButtonState::Released,
                                };
                                if let Some(names) = self.input.button_mapping.get(&keycode) {
                                    for name in names {
                                        app.events.push_input(InputEvent::Button(ButtonEvent {
                                            name,
                                            state: action_state
                                        }));
                                    }
                                }
                            }
                            WindowEvent::CloseRequested => {
                                app.events.push_system(SystemEvent::CloseRequested);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                renderer.resize(new_inner_size.width, new_inner_size.height);
                            }
                            WindowEvent::Resized(_) => {
                                let inner_size = self.window.inner_size(); 
                                renderer.resize(inner_size.width, inner_size.height);
                            }
                            WindowEvent::ReceivedCharacter(c) => {
                                app.events.push_input(InputEvent::Text(TextEvent::Character(c)));
                            }
                            WindowEvent::CursorMoved { device_id: _, position, .. } => {
                                let p = Vec2::new(position.x as f32, position.y as f32);
                                let wsize: UVec2 = (self.window.inner_size().width, self.window.inner_size().height).into();
                                let viewport = compute_fixed_viewport(wsize);
                                let relp = p - Vec2::new(viewport.x, viewport.y);

                                app.events.push_input(InputEvent::Cursor(CursorEvent::Update { 
                                    position: ((relp / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2())
                                }));
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if window_id == self.window.id() {
                        match renderer.render(&app) {
                            Ok(_) => {}
                            Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                _ => {}
            }

            // Progress app and check close requested event
            app.progress();
            if app.close_requested() {
                *control_flow = ControlFlow::Exit;
            } else {
                self.window.request_redraw();
            }
        });
    }
}

fn main() {
    let winit_context = WinitContext::default();
    let wgpu_context = WGPURenderer::new(&winit_context.window);
    let mut app = Application::default();
    
    let texture = ImageImporter::new()
        .from_source(Path::new("car.png"))
        .with_name("car".into())
        .import().expect("Failed to import texture.");
    let model = ModelImporter::new()
        .from_obj(Path::new("Car.obj"))
        .with_flat_normals(false)
        .with_name("car".into())
        .import().expect("Failed to import model.");
    
    model.push_events(&mut app);
    texture.push_events(&mut app);

    winit_context.run(app, wgpu_context);
}