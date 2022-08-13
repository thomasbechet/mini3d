use std::collections::HashMap;

use mini3d_core::{app::{App}, service::{renderer::{RendererService, RendererError, SCREEN_RESOLUTION}}, input::{event::{ButtonEvent, ButtonState, AxisEvent, TextEvent, CursorEvent}, binding::{Button, Axis}, input_manager::InputName}, event::{self, InputEvent}, glam::{Vec2, UVec2}};
use mini3d_wgpu::compute_viewport;
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

impl WinitContext {

    pub fn new() -> Self {
        let event_loop = event_loop::EventLoop::new();
        let window = window::WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(600, 400))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();
        window.set_cursor_visible(false);
        let input = WinitInput::new();
        WinitContext { window, event_loop, input }
    }

    pub fn run(
        mut self, 
        mut app: App, 
        mut renderer: impl RendererService + 'static
    ) {
        let event_loop = self.event_loop;
        event_loop.run(move |event, _, control_flow| {

            // Handle inputs
            if self.input.input_helper.update(&event) {
                if self.input.input_helper.key_pressed(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit;
                }
                app.push_event(event::Event::Input(event::InputEvent::Axis(AxisEvent {
                    name: Axis::CURSOR_X,
                    value: self.input.input_helper.mouse_diff().0,
                })));
                app.push_event(event::Event::Input(event::InputEvent::Axis(AxisEvent {
                    name: Axis::CURSOR_Y,
                    value: self.input.input_helper.mouse_diff().1,
                })));
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
                                        app.push_event(event::Event::Input(InputEvent::Button(ButtonEvent {
                                            name: name,
                                            state: action_state
                                        })));
                                    }
                                }
                            }
                            WindowEvent::CloseRequested => {
                                app.push_event(event::Event::CloseRequested);
                                *control_flow = ControlFlow::Exit;
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                renderer.resize(new_inner_size.width, new_inner_size.height);
                            }
                            WindowEvent::Resized(_) => {
                                let inner_size = self.window.inner_size(); 
                                renderer.resize(inner_size.width, inner_size.height);
                            }
                            WindowEvent::ReceivedCharacter(c) => {
                                app.push_event(event::Event::Input(InputEvent::Text(TextEvent::Character(c))));
                            }
                            WindowEvent::CursorMoved { device_id: _, position, .. } => {
                                let p = Vec2::new(position.x as f32, position.y as f32);
                                let wsize: UVec2 = (self.window.inner_size().width, self.window.inner_size().height).into();
                                let viewport = compute_viewport(wsize);
                                let relp = p - Vec2::new(viewport.x, viewport.y);

                                app.push_event(event::Event::Input(event::InputEvent::Cursor(CursorEvent::Update { 
                                    position: ((relp / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2()).into()
                                })));
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if window_id == self.window.id() {
                        match renderer.render() {
                            Ok(_) => {}
                            Err(RendererError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                Event::MainEventsCleared => {
                    app.progress();
                    app.render(&mut renderer);
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}