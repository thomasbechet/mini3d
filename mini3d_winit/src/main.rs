use std::{collections::HashMap, path::Path};

use mini3d::{app::{App}, glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION, input::{button::{ButtonState, ButtonInputId, ButtonInput}, InputDatabase, axis::{AxisInputId, AxisInput}}, event::{input::{InputEvent, ButtonEvent, TextEvent, AxisEvent}, system::SystemEvent, AppEvents}, backend::BackendDescriptor, request::AppRequests, slotmap::Key};
use mini3d_os::program::OSProgram;
use mini3d_utils::{image::ImageImporter, model::ModelImporter};
use mini3d_wgpu::{compute_fixed_viewport, WGPURenderer};
use wgpu::SurfaceError;
use winit::{window, event_loop::{self, ControlFlow}, dpi::PhysicalSize, event::{Event, WindowEvent, VirtualKeyCode, ElementState}};
use winit_input_helper::WinitInputHelper;

struct WinitInput {
    pub input_helper: WinitInputHelper,
    pub button_bindings: HashMap<VirtualKeyCode, (String, ButtonInputId)>,
    pub axis_bindings: HashMap<VirtualKeyCode, (String, f32, AxisInputId)>,
    pub cursor_x: AxisInputId,
    pub cursor_y: AxisInputId,
}

impl WinitInput {
    pub fn new() -> Self {
        Self { 
            input_helper: WinitInputHelper::new(),
            button_bindings: HashMap::from([
                (VirtualKeyCode::Z, (ButtonInput::MOVE_UP.to_string(), ButtonInputId::null())),
                (VirtualKeyCode::Q, (ButtonInput::MOVE_LEFT.to_string(), ButtonInputId::null())),
                (VirtualKeyCode::S, (ButtonInput::MOVE_RIGHT.to_string(), ButtonInputId::null())),
                (VirtualKeyCode::D, (ButtonInput::MOVE_DOWN.to_string(), ButtonInputId::null())),
                (VirtualKeyCode::Space, (ButtonInput::SWITCH_CONTROL_MODE.to_string(), ButtonInputId::null())),
                (VirtualKeyCode::C, ("switch2".to_string(), ButtonInputId::null())),
            ]),
            axis_bindings: HashMap::from([
                (VirtualKeyCode::O, (AxisInput::MOTION_Y.to_string(), -1.0, AxisInputId::null())),
                (VirtualKeyCode::L, (AxisInput::MOTION_Y.to_string(), 1.0, AxisInputId::null())),
                (VirtualKeyCode::K, (AxisInput::MOTION_X.to_string(), -1.0,AxisInputId::null())),
                (VirtualKeyCode::M, (AxisInput::MOTION_X.to_string(), 1.0, AxisInputId::null())),
            ]),
            cursor_x: AxisInputId::null(),
            cursor_y: AxisInputId::null(),
        }
    }

    pub fn reload(&mut self, app: &App) {

        println!("reload bindings");

        // Update buttons
        for button in InputDatabase::iter_buttons(app) {
            let entry = self.button_bindings.values_mut()
                .find(|e| e.0 == button.name);
            if let Some(entry) = entry {
                entry.1 = button.id;
            }
        }
        
        // Update axis
        for axis in InputDatabase::iter_axis(app) {
            if axis.name == AxisInput::CURSOR_X {
                self.cursor_x = axis.id;
            } else if axis.name == AxisInput::CURSOR_Y {
                self.cursor_y = axis.id;
            }
            self.axis_bindings.values_mut()
                .filter(|e| e.0 == axis.name)
                .for_each(|e| e.2 = axis.id);
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
        mut app: App, 
        mut events: AppEvents, 
        mut requests: AppRequests,
        mut renderer: WGPURenderer,
    ) {
        let event_loop = self.event_loop;
        event_loop.run(move |event, _, control_flow| {

            // Handle inputs
            if self.input.input_helper.update(&event) {
                if self.input.input_helper.key_pressed(VirtualKeyCode::Escape) {
                    events.push_system(SystemEvent::Shutdown);
                }
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
                                if let Some((_, id)) = self.input.button_bindings.get(&keycode) {
                                    events.push_input(InputEvent::Button(ButtonEvent { id: *id, state: action_state }));
                                }

                                if let Some((_, dir, id)) = self.input.axis_bindings.get(&keycode) {
                                    let value = match action_state {
                                        ButtonState::Pressed => *dir,
                                        ButtonState::Released => 0.0,
                                    };
                                    // println!("{:?} {:?} {:?}", keycode, value, *id);
                                    events.push_input(InputEvent::Axis(AxisEvent { id: *id, value: value * 2.0}));
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

                    // Build backend descriptor
                    let desc = BackendDescriptor::new()
                        .with_renderer(&mut renderer);
                    
                    // Progress application
                    app.progress(desc, &mut events, &mut requests)
                        .expect("Failed to progress application");

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