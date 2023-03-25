use std::{time::{SystemTime, Instant}, path::Path, fs::File, io::{Read, Write}};

use gui::{WindowGUI, WindowControl};
use mapper::InputMapper;
use mini3d::{event::{Events, system::SystemEvent, input::{InputEvent, InputTextEvent}, asset::{ImportAssetEvent, AssetImportEntry}}, request::Requests, engine::Engine, glam::Vec2, renderer::SCREEN_RESOLUTION, feature::asset::rhai_script::RhaiScript};
use mini3d_utils::{image::ImageImporter, model::ModelImporter};
use mini3d_wgpu::WGPURenderer;
use utils::{compute_fixed_viewport, ViewportMode};
use window::Window;
use winit::{event_loop::{EventLoop, ControlFlow}, event::{Event, DeviceEvent, WindowEvent, ElementState, VirtualKeyCode, MouseButton, MouseScrollDelta}};

pub mod gui;
pub mod mapper;
pub mod utils;
pub mod window;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DisplayMode {
    FullscreenFocus,
    WindowedFocus,
    WindowedUnfocus,
}

fn set_display_mode(window: &mut Window, gui: &mut WindowGUI, mode: DisplayMode) -> DisplayMode {
    match mode {
        DisplayMode::FullscreenFocus => {
            window.set_fullscreen(true);
            window.set_focus(true);
            gui.set_visible(false);
        },
        DisplayMode::WindowedFocus => {
            window.set_fullscreen(false);
            window.set_focus(true);
            gui.set_visible(true);
        },
        DisplayMode::WindowedUnfocus => {
            window.set_fullscreen(false);
            window.set_focus(false);
            gui.set_visible(true);
        },
    }
    mode
}

fn main() {
    // Window
    let event_loop = EventLoop::new();
    let mut window = Window::new(&event_loop);
    let mut mapper = InputMapper::new(); 

    // Renderer
    let mut renderer = WGPURenderer::new(&window.handle);
    let mut gui = WindowGUI::new(renderer.context(), &window.handle, &event_loop, &mapper);

    // Engine
    let mut engine = Engine::new(mini3d_os::system::init::init).expect("Failed to create engine");
    let mut events = Events::new();
    let mut requests = Requests::new();

    let mut last_click: Option<SystemTime> = None;
    let mut last_time = Instant::now();
    let mut mouse_motion = (0.0, 0.0);
    let mut wheel_motion = (0.0, 0.0);
    let mut last_mouse_motion = mouse_motion;
    let mut last_wheel_motion = wheel_motion;

    // Controllers
    let mut gilrs = gilrs::Gilrs::new().unwrap();
    
    // Set initial display
    let mut display_mode = set_display_mode(&mut window, &mut gui, DisplayMode::WindowedUnfocus);
    let viewport_mode = ViewportMode::StretchKeepAspect;
    // let viewport_mode = ViewportMode::FixedBestFit;

    // Save state
    let mut save_state = false;
    let mut load_state = false;

    ImageImporter::new()
        .from_source(Path::new("assets/car.png"))
        .with_name("car")
        .import().expect("Failed to import car texture.")
        .push(&mut events);
    ImageImporter::new()
        .from_source(Path::new("assets/Sprites/UI_Flat_Frame_01_Standard.png"))
        .with_name("frame")
        .import().expect("Failed to import frame texture.")
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
    let script = std::fs::read_to_string("assets/inventory.rhai").expect("Failed to load.");
    events.asset.push(ImportAssetEvent::RhaiScript(AssetImportEntry {
        name: "inventory".to_string(), 
        data: RhaiScript { source: script },
    }));

    // Enter loop
    event_loop.run(move |event, _, control_flow| {

        // Update gui
        if display_mode == DisplayMode::WindowedUnfocus {
            gui.handle_event(&event, &mut mapper, &mut window);
        }

        // Match window events
        match event {
            Event::DeviceEvent { device_id: _, event } => {
                match event {
                    DeviceEvent::MouseMotion { delta } => {
                        mouse_motion.0 += delta.0;
                        mouse_motion.1 += delta.1;
                    },
                    DeviceEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(x, y) } => {
                        wheel_motion.0 += x;
                        wheel_motion.1 += y;
                    },
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
                            if state == ElementState::Pressed && keycode == VirtualKeyCode::Escape && !gui.is_recording() {
                                display_mode = set_display_mode(&mut window, &mut gui, DisplayMode::WindowedUnfocus);
                            }

                            // Save/Load state
                            if state == ElementState::Pressed && keycode == VirtualKeyCode::F5 && !gui.is_recording() {
                                save_state = true;
                            } else if state == ElementState::Pressed && keycode == VirtualKeyCode::F6 && !gui.is_recording() {
                                load_state = true;
                            }

                            // Toggle fullscreen
                            if state == ElementState::Pressed && keycode == VirtualKeyCode::F11 && !gui.is_fullscreen() {
                                match display_mode {
                                    DisplayMode::FullscreenFocus => {
                                        display_mode = set_display_mode(&mut window, &mut gui, DisplayMode::WindowedFocus);
                                    },
                                    _ => {
                                        display_mode = set_display_mode(&mut window, &mut gui, DisplayMode::FullscreenFocus);
                                    }
                                }
                            }

                            // Dispatch keyboard
                            if window.is_focus() {
                                mapper.dispatch_keyboard(keycode, state, &mut events);
                            }
                        }
                        WindowEvent::MouseInput { device_id: _, state, button, .. } => {
                            
                            // Focus mouse
                            if state == ElementState::Pressed && button == MouseButton::Left && !window.is_focus() && !gui.is_fullscreen() {
                                if last_click.is_none() {
                                    last_click = Some(SystemTime::now());
                                } else {
                                    display_mode = set_display_mode(&mut window, &mut gui, DisplayMode::WindowedFocus);
                                }
                            }

                            // Dispatch mouse
                            if window.is_focus() {
                                mapper.dispatch_mouse_button(button, state, &mut events);
                            }
                        }
                        WindowEvent::CloseRequested => {
                            events.system.push(SystemEvent::Shutdown);
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
                                events.input.push(InputEvent::Text(InputTextEvent {
                                    stream: "main".into(),
                                    value: c.to_string(),
                                }));
                            }
                        }
                        WindowEvent::CursorMoved { device_id: _, position, .. } => {
                            if window.is_focus() {
                                let position = Vec2::new(position.x as f32, position.y as f32);
                                let viewport = compute_fixed_viewport(gui.central_viewport(), viewport_mode);
                                let relative_position = position - Vec2::new(viewport.x, viewport.y);
                                let final_position = (relative_position / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();
                                mapper.dispatch_mouse_cursor((final_position.x, final_position.y), &mut events);
                            }
                        }
                        // WindowEvent::MouseWheel { device_id: _, delta, .. } => {
                        //     if window.is_focus() {
                        //         if let MouseScrollDelta::LineDelta(x, y) = delta {
                        //             mapper.dispatch_mouse_wheel((x, y), &mut events);
                        //         }
                        //     }
                        // }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(_) => {
                
            }
            Event::MainEventsCleared => {

                // Dispatch mouse motion and reset
                if window.is_focus() {
                    if mouse_motion.0 != last_mouse_motion.0 || mouse_motion.1 != last_mouse_motion.1 {
                        mapper.dispatch_mouse_motion(mouse_motion, &mut events);
                        last_mouse_motion = mouse_motion;
                    }
                    if wheel_motion.0 != last_wheel_motion.0 || wheel_motion.1 != last_wheel_motion.1 {
                        mapper.dispatch_mouse_wheel(wheel_motion, &mut events);
                        last_wheel_motion = wheel_motion;
                    }
                } 
                mouse_motion = (0.0, 0.0);
                wheel_motion = (0.0, 0.0);

                // Dispatch controller events
                while let Some(gilrs::Event { id, event, .. }) = &gilrs.next_event() {
                    if display_mode == DisplayMode::WindowedUnfocus {
                        gui.handle_controller_event(event, *id, &mut mapper, &mut window);
                    } else {
                        match event {
                            gilrs::EventType::ButtonPressed(button, _) => {
                                mapper.dispatch_controller_button(*id, *button, true, &mut events);
                            },
                            gilrs::EventType::ButtonReleased(button, _) => {
                                mapper.dispatch_controller_button(*id, *button, false, &mut events);
                            },
                            gilrs::EventType::AxisChanged(axis, value, _) => {
                                mapper.dispatch_controller_axis(*id, *axis, *value, &mut events);
                            },
                            _ => {}
                        }
                    }
                }
                
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
                let last_display_mode = display_mode;
                gui.ui(
                    &mut window,
                    &engine,
                    &mut mapper,
                    &mut WindowControl {
                        control_flow,
                        display_mode: &mut display_mode,
                        request_save: &mut save_state,
                        request_load: &mut load_state,
                    },
                );

                // Update display mode
                if display_mode != last_display_mode {
                    set_display_mode(&mut window, &mut gui, display_mode);
                }

                // Progress engine
                engine.progress(&events, &mut requests, delta_time).expect("Failed to progress engine");
                engine.update_renderer(&mut renderer, false).expect("Failed to render");
                
                // Save/Load state
                if save_state {

                    // {
                    //     let file = File::create("assets/state.json").expect("Failed to create file");
                    //     let mut serializer = serde_json::Serializer::new(file);
                    //     engine.save_state(&mut serializer).expect("Failed to serialize");
                    // }

                    {
                        let mut file = File::create("assets/state.bin").unwrap();
                        let mut bytes: Vec<u8> = Default::default();
                        let mut bincode_serializer = bincode::Serializer::new(&mut bytes, bincode::options());
                        engine.save_state(&mut bincode_serializer).expect("Failed to serialize");
                        bytes = miniz_oxide::deflate::compress_to_vec_zlib(bytes.as_slice(), 10);
                        file.write_all(&bytes).unwrap();
                    }
                    
                    // {
                    //     let mut file = File::create("assets/state_postcard.bin").unwrap();
                    //     struct EngineSerialize<'a> {
                    //         engine: &'a Engine,
                    //     }
                    //     use serde::Serialize;
                    //     impl<'a> Serialize for EngineSerialize<'a> {
                    //         fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                    //             where S: serde::Serializer {
                    //             self.engine.save_state(serializer)
                    //         }
                    //     }
                    //     let mut bytes = postcard::to_allocvec(&EngineSerialize { engine: &engine }).unwrap();
                    //     bytes = miniz_oxide::deflate::compress_to_vec_zlib(bytes.as_slice(), 10);
                    //     file.write_all(&bytes).unwrap();
                    // }

                    save_state = false;
                } else if load_state {
                    
                    // {
                    //     let file = File::open("assets/state.json").expect("Failed to open file");
                    //     let mut deserializer = serde_json::Deserializer::from_reader(file);
                    //     engine.load_state(&mut deserializer).expect("Failed to deserialize");
                    // }
                    
                    {
                        let mut file = File::open("assets/state.bin").expect("Failed to open file");
                        let mut bytes: Vec<u8> = Default::default();
                        file.read_to_end(&mut bytes).expect("Failed to read to end");
                        let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes).expect("Failed to decompress");
                        let mut deserializer = bincode::Deserializer::from_slice(&bytes, bincode::options());
                        engine.load_state(&mut deserializer).expect("Failed to load state");
                    }

                    // {
                    //     let mut file = File::open("assets/state_postcard.bin").expect("Failed to open file");
                    //     let mut bytes: Vec<u8> = Default::default();
                    //     file.read_to_end(&mut bytes).expect("Failed to read to end");
                    //     let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes).expect("Failed to decompress");
                    //     let mut deserializer = postcard::Deserializer::from_bytes(&bytes);
                    //     engine.load_state(&mut deserializer).expect("Failed to load state");
                    // }

                    engine.update_renderer(&mut renderer, true).expect("Failed to reset renderer");

                    load_state = false;
                }
                
                // Invoke WGPU Renderer
                let viewport = compute_fixed_viewport(gui.central_viewport(), viewport_mode);
                renderer.render(viewport, |device, queue, encoder, output| {
                    gui.render(&window.handle, device, queue, encoder, output);
                }).expect("Failed to render");

                // Check shutdown
                if requests.shutdown() {
                    println!("Request Shutdown");
                    *control_flow = ControlFlow::Exit;
                }

                // Check input reloading
                if requests.reload_input_mapping() {
                    mapper.refresh(&engine);
                }

                // Reset requests and events
                requests.reset();
                events.clear();

                // Check exit
                if *control_flow != ControlFlow::Exit {
                    window.handle.request_redraw();
                }
            }
            _ => {}
        }
    });
}