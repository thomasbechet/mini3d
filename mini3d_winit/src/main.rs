use std::collections::HashMap;

use mini3d::{application::{Application}, input::{event::{ButtonEvent, ButtonState, AxisEvent, TextEvent, CursorEvent, self, InputEvent}, binding::{Button, Axis}, range::InputName}, glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION, event_recorder::EventRecorder};
use mini3d_wgpu::{compute_viewport, WGPUContext};
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
            .with_inner_size(PhysicalSize::new(600, 400))
            .with_resizable(true)
            .build(&event_loop)
            .unwrap();
        window.set_cursor_visible(false);
        let input = WinitInput::new();
        Self { window, event_loop, input }
    }
}

impl WinitContext {
    pub fn run(
        mut self, 
        mut app: Application, 
        mut recorder: EventRecorder,
        mut renderer: WGPUContext,
    ) {
        let event_loop = self.event_loop;
        event_loop.run(move |event, _, control_flow| {

            // Handle inputs
            if self.input.input_helper.update(&event) {
                if self.input.input_helper.key_pressed(VirtualKeyCode::Escape) {
                    *control_flow = ControlFlow::Exit;
                }
                recorder.push_input_event(event::InputEvent::Axis(AxisEvent {
                    name: Axis::CURSOR_X,
                    value: self.input.input_helper.mouse_diff().0,
                }));
                recorder.push_input_event(event::InputEvent::Axis(AxisEvent {
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
                                        recorder.push_input_event(InputEvent::Button(ButtonEvent {
                                            name,
                                            state: action_state
                                        }));
                                    }
                                }
                            }
                            WindowEvent::CloseRequested => {
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
                                recorder.push_input_event(InputEvent::Text(TextEvent::Character(c)));
                            }
                            WindowEvent::CursorMoved { device_id: _, position, .. } => {
                                let p = Vec2::new(position.x as f32, position.y as f32);
                                let wsize: UVec2 = (self.window.inner_size().width, self.window.inner_size().height).into();
                                let viewport = compute_viewport(wsize);
                                let relp = p - Vec2::new(viewport.x, viewport.y);

                                recorder.push_input_event(event::InputEvent::Cursor(CursorEvent::Update { 
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
                Event::MainEventsCleared => {
                    app.progress(&recorder);
                    recorder.reset();
                    self.window.request_redraw();
                }
                _ => {}
            }
        });
    }
}

fn main() {
    let winit_context = WinitContext::default();
    let wgpu_context = WGPUContext::new(&winit_context.window);
    let app = Application::default();
    let recorder = EventRecorder::default();
    winit_context.run(app, recorder, wgpu_context);
}