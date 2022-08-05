use std::collections::HashMap;

use mini3d_core::{app::{App}, service::{renderer::{RendererService, RendererError}}, input::{input_table::{ACTION_UP, ACTION_DOWN, ACTION_LEFT, ACTION_RIGHT, AXIS_VIEW_X, AXIS_VIEW_Y}, event::{ActionEvent, ActionState, AxisEvent, TextEvent}}, event::{self, InputEvent}};
use winit::{window, event_loop::{self, ControlFlow}, dpi::PhysicalSize, event::{Event, WindowEvent, VirtualKeyCode, ElementState}};
use winit_input_helper::WinitInputHelper;

struct WinitInput {
    pub input_helper: WinitInputHelper,
    pub action_mapping: HashMap<VirtualKeyCode, Vec<&'static str>>,
}

impl WinitInput {
    pub fn new() -> Self {
        WinitInput { 
            input_helper: WinitInputHelper::new(),
            action_mapping: HashMap::from([
                (VirtualKeyCode::Z, vec![ACTION_UP]),
                (VirtualKeyCode::S, vec![ACTION_DOWN]),
                (VirtualKeyCode::D, vec![ACTION_LEFT]),
                (VirtualKeyCode::Q, vec![ACTION_RIGHT]),
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
        window.set_cursor_visible(true);
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
                app.push_event(event::PlatformEvent::Input(event::InputEvent::Axis(AxisEvent {
                    name: AXIS_VIEW_X,
                    value: self.input.input_helper.mouse_diff().0,
                })));
                app.push_event(event::PlatformEvent::Input(event::InputEvent::Axis(AxisEvent {
                    name: AXIS_VIEW_Y,
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
                                    ElementState::Pressed => ActionState::Pressed,
                                    ElementState::Released => ActionState::Released,
                                };
                                if let Some(names) = self.input.action_mapping.get(&keycode) {
                                    for name in names {
                                        app.push_event(event::PlatformEvent::Input(InputEvent::Action(ActionEvent {
                                            name: name,
                                            state: action_state
                                        })));
                                    }
                                }
                            }
                            WindowEvent::CloseRequested => {
                                app.push_event(event::PlatformEvent::CloseRequested);
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
                                app.push_event(event::PlatformEvent::Input(InputEvent::Text(TextEvent::Character(c))));
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