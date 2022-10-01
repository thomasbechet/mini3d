use std::{collections::HashMap, path::Path, time::Instant};

use mini3d::{app::{App}, glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION, input::{action::{ActionState, ActionInputId, ActionInput}, InputDatabase, axis::{AxisInputId, AxisInput}}, event::{input::{InputEvent, ActionEvent, TextEvent, AxisEvent}, system::SystemEvent, AppEvents}, backend::BackendDescriptor, request::AppRequests, slotmap::Key};
use mini3d_os::program::OSProgram;
use mini3d_utils::{image::ImageImporter, model::ModelImporter};
use mini3d_wgpu::{compute_fixed_viewport, WGPURenderer};
use wgpu::SurfaceError;
use winit::{window, event_loop::{self, ControlFlow}, dpi::PhysicalSize, event::{Event, WindowEvent, VirtualKeyCode, ElementState, DeviceEvent}};
use winit_input_helper::WinitInputHelper;

struct WinitInput {
    pub input_helper: WinitInputHelper,
    pub keycode_to_bindings: HashMap<VirtualKeyCode, (String, ActionInputId)>,
    pub keycode_to_axis: HashMap<VirtualKeyCode, (String, f32, AxisInputId)>,
    pub cursor_x: AxisInputId,
    pub cursor_y: AxisInputId,
    pub motion_x: AxisInputId,
    pub motion_y: AxisInputId,
    pub mouse_motion: Vec2,
}

impl WinitInput {
    pub fn new() -> Self {
        Self { 
            input_helper: WinitInputHelper::new(),
            keycode_to_bindings: HashMap::from([
                (VirtualKeyCode::Z, (ActionInput::UP.to_string(), ActionInputId::null())),
                (VirtualKeyCode::Q, (ActionInput::LEFT.to_string(), ActionInputId::null())),
                (VirtualKeyCode::S, (ActionInput::DOWN.to_string(), ActionInputId::null())),
                (VirtualKeyCode::D, (ActionInput::RIGHT.to_string(), ActionInputId::null())),
                (VirtualKeyCode::C, ("switch_mode".to_string(), ActionInputId::null())),
                (VirtualKeyCode::A, ("roll_left".to_string(), ActionInputId::null())),
                (VirtualKeyCode::E, ("roll_right".to_string(), ActionInputId::null())),
                (VirtualKeyCode::F, ("toggle_layout".to_string(), ActionInputId::null())),
            ]),
            keycode_to_axis: HashMap::from([
                (VirtualKeyCode::O, (AxisInput::MOTION_Y.to_string(), -1.0, AxisInputId::null())),
                (VirtualKeyCode::L, (AxisInput::MOTION_Y.to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::K, (AxisInput::MOTION_X.to_string(), -1.0,AxisInputId::null())),
                (VirtualKeyCode::M, (AxisInput::MOTION_X.to_string(), 1.0, AxisInputId::null())),

                (VirtualKeyCode::Z, ("move_forward".to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::S, ("move_backward".to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::Q, ("move_left".to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::D, ("move_right".to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::X, ("move_up".to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::W, ("move_down".to_string(), 1.0, AxisInputId::null())),
            ]),
            cursor_x: AxisInputId::null(),
            cursor_y: AxisInputId::null(),
            motion_x: AxisInputId::null(),
            motion_y: AxisInputId::null(),
            mouse_motion: Vec2::ZERO,
        }
    }

    pub fn reload(&mut self, app: &App) {

        println!("reload bindings");

        // Update actions
        for id in InputDatabase::iter_actions(app) {
            let action = InputDatabase::action(app, id).unwrap();
            if let Some(entry) = self.keycode_to_bindings.values_mut().find(|e| e.0 == action.name) {
                entry.1 = id;
            }
        }
        
        // Update axis
        for id in InputDatabase::iter_axis(app) {
            let axis = InputDatabase::axis(app, id).unwrap();
            if axis.name == AxisInput::CURSOR_X {
                self.cursor_x = axis.id;
            } else if axis.name == AxisInput::CURSOR_Y {
                self.cursor_y = axis.id;
            } else if axis.name == AxisInput::MOTION_X {
                self.motion_x = axis.id;
            } else if axis.name == AxisInput::MOTION_Y {
                self.motion_y = axis.id;
            }
            self.keycode_to_axis.values_mut()
                .filter(|e| e.0 == axis.name)
                .for_each(|e| e.2 = id);
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
        window.set_cursor_grab(true);
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
        mut app: App, 
        mut events: AppEvents, 
        mut requests: AppRequests,
        mut renderer: WGPURenderer,
    ) {
        let event_loop = self.event_loop;
        let mut last = Instant::now();
        event_loop.run(move |event, _, control_flow| {

            // Handle inputs
            if self.input.input_helper.update(&event) {
                if self.input.input_helper.key_pressed(VirtualKeyCode::Escape) {
                    events.push_system(SystemEvent::Shutdown);
                }
            }

            // Match window events
            match event {
                Event::DeviceEvent { device_id: _, event } => {
                    match event {
                        DeviceEvent::MouseMotion { delta } => {
                            self.input.mouse_motion.x = delta.0 as f32;
                            self.input.mouse_motion.y = delta.1 as f32;
                        }
                        _ => {}
                    }
                }
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
                                if let Some((_, id)) = self.input.keycode_to_bindings.get(&keycode) {
                                    events.push_input(InputEvent::Action(ActionEvent { id: *id, state: action_state }));
                                }

                                if let Some((_name, dir, id)) = self.input.keycode_to_axis.get(&keycode) {
                                    let value = match action_state {
                                        ActionState::Pressed => *dir,
                                        ActionState::Released => 0.0,
                                    };
                                    // println!("{} {:?} {:?} {:?}", name, keycode, value, *id);
                                    events.push_input(InputEvent::Axis(AxisEvent { id: *id, value: value}));
                                }
                            }
                            WindowEvent::CloseRequested => {
                                events.push_system(SystemEvent::Shutdown);
                            }
                            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                                renderer.resize(new_inner_size.width, new_inner_size.height);
                            }
                            WindowEvent::Resized(_) => {
                                let inner_size = self.window.inner_size(); 
                                renderer.resize(inner_size.width, inner_size.height);
                            }
                            WindowEvent::ReceivedCharacter(c) => {
                                events.push_input(InputEvent::Text(TextEvent::Character(c)));
                            }
                            WindowEvent::CursorMoved { device_id: _, position, .. } => {
                                let p = Vec2::new(position.x as f32, position.y as f32);
                                let wsize: UVec2 = (self.window.inner_size().width, self.window.inner_size().height).into();
                                let viewport = compute_fixed_viewport(wsize);
                                let relp = p - Vec2::new(viewport.x, viewport.y);
                                let final_position = (relp / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();
                                events.push_input(InputEvent::Axis(AxisEvent { id: self.input.cursor_x, value: final_position.x }));
                                events.push_input(InputEvent::Axis(AxisEvent { id: self.input.cursor_y, value: final_position.y }));                                    
                            }
                            _ => {}
                        }
                    }
                }
                Event::RedrawRequested(window_id) => {
                    if window_id == self.window.id() {
                        match renderer.render(&mut app) {
                            Ok(_) => {}
                            Err(SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                }
                Event::MainEventsCleared => {

                    // Push mouse motion events
                    events.push_input(InputEvent::Axis(AxisEvent { id: self.input.motion_x, value: self.input.mouse_motion.x * 0.01 as f32 }));
                    events.push_input(InputEvent::Axis(AxisEvent { id: self.input.motion_y, value: self.input.mouse_motion.y * 0.01 as f32 }));

                    // Build backend descriptor
                    let desc = BackendDescriptor::new()
                        .with_renderer(&mut renderer);
                    
                    // Compute delta time
                    let now = Instant::now();
                    let delta_time = (now - last).as_secs_f32();
                    last = now;

                    // Progress application
                    app.progress(desc, &mut events, &mut requests, delta_time)
                        .expect("Failed to progress application");

                    self.input.mouse_motion = Vec2::ZERO;

                    // Handle requests
                    if requests.shutdown() {
                        println!("Request Shutdown");
                        *control_flow = ControlFlow::Exit;
                    } else {
                        self.window.request_redraw();
                    }
                    if requests.reload_bindings() {
                        self.input.reload(&app);
                    }
                    requests.reset();
                }
                _ => {}
            }
        });
    }
}

fn main() {
    let winit_context = WinitContext::default();
    let wgpu_context = WGPURenderer::new(&winit_context.window);
    let app = App::new::<OSProgram>(())
        .expect("Failed to create application with OS program");
    let mut events = AppEvents::new();
    let requests = AppRequests::new();
    
    ImageImporter::new()
        .from_source(Path::new("assets/car.png"))
        .with_name("car")
        .import().expect("Failed to import car texture.")
        .push(&mut events);
    ModelImporter::new()
        .from_obj(Path::new("assets/car.obj"))
        .with_flat_normals(false)
        .with_name("car")
        .import().expect("Failed to import car model.")
        .push(&mut events);  
    ImageImporter::new()
        .from_source(Path::new("assets/alfred.png"))
        .with_name("alfred")
        .import().expect("Failed to import alfred texture.")
        .push(&mut events);
    ModelImporter::new()
        .from_obj(Path::new("assets/alfred.obj"))
        .with_flat_normals(false)
        .with_name("alfred")
        .import().expect("Failed to import alfred model.")
        .push(&mut events);

    winit_context.run(app, events, requests, wgpu_context);
}