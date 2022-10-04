use std::{time::{SystemTime, Instant}, path::Path};

use gui::WindowGUI;
use mapper::InputMapper;
use mini3d::{event::{AppEvents, system::SystemEvent, input::{InputEvent, TextEvent}}, request::AppRequests, app::App, glam::Vec2, graphics::SCREEN_RESOLUTION, backend::BackendDescriptor};
use mini3d_os::program::OSProgram;
use mini3d_utils::{image::ImageImporter, model::ModelImporter};
use mini3d_wgpu::WGPURenderer;
use utils::compute_fixed_viewport;
use wgpu::SurfaceError;
use window::Window;
use winit::{event_loop::{EventLoop, ControlFlow}, event::{Event, DeviceEvent, WindowEvent, ElementState, VirtualKeyCode, MouseButton}};

pub mod gui;
pub mod mapper;
pub mod utils;
pub mod window;

fn main() {
    // Window
    let event_loop = EventLoop::new();
    let mut window = Window::new(&event_loop);
    let mut mapper = InputMapper::new(); 

    // Renderer
    let mut renderer = WGPURenderer::new(&window.handle);
    let mut gui = WindowGUI::new(renderer.context(), &window.handle);

    // Application
    let mut app = App::new::<OSProgram>(())
        .expect("Failed to create application with OS program");
    let mut events = AppEvents::new();
    let mut requests = AppRequests::new();

    let mut last_click: Option<SystemTime> = None;
    let mut last_time = Instant::now();
    let mut mouse_motion = (0.0, 0.0);

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

    // Enter loop
    event_loop.run(move |event, _, control_flow| {

        // Update gui
        if !window.is_focus() {
            gui.handle_event(&event);
        }

        // Match window events
        match event {
            Event::DeviceEvent { device_id: _, event } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                        mouse_motion = delta;
                    }
                    _ => {}
                }
            }
            Event::WindowEvent { window_id, event } => {
                if window_id == window.handle.id() {
                    match event {
                        WindowEvent::KeyboardInput {
                            input: winit::event::KeyboardInput {
                                virtual_keycode: Some(keycode),
                                state,
                                ..
                            },
                            ..
                        } => {

                            // Unfocus mouse
                            if state == ElementState::Pressed && keycode == VirtualKeyCode::Escape && window.is_focus() {
                                window.set_focus(false);
                            }

                            // Toggle fullscreen
                            if state == ElementState::Pressed && keycode == VirtualKeyCode::P {
                                window.set_fullscreen(!window.is_fullscreen()); // Switch
                                gui.set_visible(!window.is_fullscreen());
                            }

                            // Dispatch keyboard
                            if window.is_focus() {
                                mapper.dispatch_keyboard(keycode, state, &mut events);
                            }
                        }
                        WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                            
                            // Focus mouse
                            if state == ElementState::Pressed && button == MouseButton::Left && !window.is_focus() {
                                if last_click.is_none() {
                                    last_click = Some(SystemTime::now());
                                } else {
                                    window.set_focus(true);
                                }
                            }

                            // Dispatch mouse
                            if window.is_focus() {
                                mapper.dispatch_mouse_button(button, state, &mut events);
                            }
                        }
                        WindowEvent::CloseRequested => {
                            events.push_system(SystemEvent::Shutdown);
                            *control_flow = ControlFlow::Exit;
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(new_inner_size.width, new_inner_size.height);
                        }
                        WindowEvent::Resized(_) => {
                            let inner_size = window.handle.inner_size();
                            renderer.resize(inner_size.width, inner_size.height);
                        }
                        WindowEvent::ReceivedCharacter(c) => {
                            if window.is_focus() {
                                events.push_input(InputEvent::Text(TextEvent::Character(c)));
                            }
                        }
                        WindowEvent::CursorMoved { device_id: _, position, .. } => {
                            if window.is_focus() {
                                let position = Vec2::new(position.x as f32, position.y as f32);
                                let viewport = compute_fixed_viewport(gui.central_viewport());
                                let relative_position = position - Vec2::new(viewport.x, viewport.y);
                                let final_position = (relative_position / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();
                                mapper.dispatch_mouse_cursor((final_position.x, final_position.y), &mut events);
                            }
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) => {
                if window_id == window.handle.id() {

                    // Compute app viewport
                    let viewport = compute_fixed_viewport(gui.central_viewport());

                    // Invoke WGPU Renderer
                    renderer.render(&mut app, viewport, |device, queue, encoder, output| {
                        gui.render(&window.handle, device, queue, encoder, output);
                    })
                    .map_err(|e| match e {
                        SurfaceError::OutOfMemory => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }).expect("Error");                        
                }
            }
            Event::MainEventsCleared => {

                // Dispatch mouse motion and reset
                if window.is_focus() {
                    mapper.dispatch_mouse_motion(mouse_motion, &mut events);
                }
                mouse_motion = (0.0, 0.0);

                // Build backend descriptor
                let desc = BackendDescriptor::new()
                    .with_renderer(&mut renderer);
                
                // Update last click
                if let Some(time) = last_click {
                    if time.elapsed().unwrap().as_millis() > 300 {
                        last_click = None;
                    }
                }

                // Compute delta time
                let now = Instant::now();
                let delta_time = (now - last_time).as_secs_f64();
                last_time = now;

                // Update GUI
                gui.ui(&mut window, control_flow, delta_time);

                // Progress application
                app.progress(desc, &mut events, &mut requests, delta_time)
                    .expect("Failed to progress application");

                // Check shutdown
                if requests.shutdown() {
                    println!("Request Shutdown");
                    *control_flow = ControlFlow::Exit;
                }

                // Check input reloading
                if requests.reload_input_mapping() {
                    mapper.reload(&app);
                }

                // Reset requests
                requests.reset();

                // Check exit
                if *control_flow != ControlFlow::Exit {
                    window.handle.request_redraw();
                }
            }
            _ => {}
        }
    });
}