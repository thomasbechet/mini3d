use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Write},
    path::Path,
    rc::Rc,
    time::{Instant, SystemTime},
};

use mini3d_core::{
    platform::{
        event::{AssetImportEntry, ImportAssetEvent, PlatformEvent},
        provider::PlatformProvider,
    },
    script::resource::Script,
    serialize::SliceDecoder,
    simulation::{Simulation, SimulationConfig},
};
use mini3d_importer::{image::ImageImporter, model::ModelImporter};
use mini3d_input::mapper::InputMapper;
use mini3d_stdlog::logger::stdout::StdoutLogger;
use mini3d_wgpu::renderer::WGPURenderer;
use provider::{
    input::WinitInputProvider, storage::WinitStorageProvider, system::WinitSystemProvider,
};
// use serde::Serialize;
use virtual_disk::VirtualDisk;
use window::Window;
use winit::{
    event::{
        DeviceEvent, ElementState, Event, KeyEvent, MouseButton, MouseScrollDelta, WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
};

pub mod provider;
pub mod virtual_disk;
pub mod window;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum DisplayMode {
    FullscreenFocus,
    WindowedFocus,
    WindowedUnfocus,
}

fn set_display_mode(window: &mut Window, mode: DisplayMode) -> DisplayMode {
    match mode {
        DisplayMode::FullscreenFocus => {
            window.set_fullscreen(true);
            window.set_focus(true);
        }
        DisplayMode::WindowedFocus => {
            window.set_fullscreen(false);
            window.set_focus(true);
        }
        DisplayMode::WindowedUnfocus => {
            window.set_fullscreen(false);
            window.set_focus(false);
        }
    }
    mode
}

struct WinitSystemStatus {
    imports: Vec<ImportAssetEvent>,
    stop_event: bool,
    running: bool,
}

impl Default for WinitSystemStatus {
    fn default() -> Self {
        Self {
            imports: Vec::default(),
            stop_event: false,
            running: true,
        }
    }
}

impl WinitSystemStatus {
    fn stop(&mut self) {
        self.stop_event = true;
    }
}

impl PlatformProvider for WinitSystemStatus {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_import(&mut self) -> Option<ImportAssetEvent> {
        self.imports.pop()
    }

    fn next_event(&mut self) -> Option<PlatformEvent> {
        if self.stop_event {
            self.stop_event = false;
            Some(PlatformEvent::RequestStop)
        } else {
            None
        }
    }

    fn request_stop(&mut self) {
        self.running = false;
    }
}

fn main_run() {
    std::env::set_var("RUST_BACKTRACE", "full");

    // Window
    let event_loop = EventLoop::new().expect("Failed to create event loop");
    let mut window = Window::new(&event_loop);

    // Input
    let input = Rc::new(RefCell::new(WinitInputProvider::default()));

    // Renderer
    let renderer = Rc::new(RefCell::new(WGPURenderer::new(&window)));

    // Disk
    let disk = Rc::new(RefCell::new(VirtualDisk::new()));

    // System
    let system_status = Rc::new(RefCell::new(WinitSystemStatus::default()));

    let mut instance = Simulation::new(SimulationConfig::default());
    instance.set_input(input.clone());
    instance.set_storage(WinitStorageProvider::new(disk));
    instance.set_platform(WinitSystemProvider::new(system_status.clone()));
    instance.set_renderer(WinitRendererProvider::new(renderer.clone()));
    instance.set_logger(StdoutLogger);

    let mut last_click: Option<SystemTime> = None;
    let mut last_time = Instant::now();
    let mut mouse_motion = (0.0, 0.0);
    let mut wheel_motion = (0.0, 0.0);
    let mut last_mouse_motion = mouse_motion;
    let mut last_wheel_motion = wheel_motion;

    // Controllers
    let mut gilrs = gilrs::Gilrs::new().unwrap();

    // Set initial display
    let mut display_mode = set_display_mode(&mut window, DisplayMode::WindowedUnfocus);

    // Save state
    let mut save_state = false;
    let mut load_state = false;

    ImageImporter::new()
        .from_source(Path::new("assets/car.png"))
        .with_name("car_tex")
        .import()
        .expect("Failed to import car texture.")
        .push(&mut system_status.borrow_mut().imports);
    ImageImporter::new()
        .from_source(Path::new("assets/GUI.png"))
        .with_name("GUI")
        .import()
        .expect("Failed to import GUI texture.")
        .push(&mut system_status.borrow_mut().imports);

    ModelImporter::new()
        .from_obj(Path::new("assets/car.obj"))
        .with_flat_normals(false)
        .with_name("car_mesh")
        .import()
        .expect("Failed to import car model.")
        .push(&mut system_status.borrow_mut().imports);
    ImageImporter::new()
        .from_source(Path::new("assets/alfred.png"))
        .with_name("alfred_tex")
        .import()
        .expect("Failed to import alfred texture.")
        .push(&mut system_status.borrow_mut().imports);
    ModelImporter::new()
        .from_obj(Path::new("assets/alfred.obj"))
        .with_flat_normals(false)
        .with_name("alfred_mesh")
        .import()
        .expect("Failed to import alfred model.")
        .push(&mut system_status.borrow_mut().imports);
    let script = std::fs::read_to_string("assets/script_main.ms").expect("Failed to load.");
    system_status
        .borrow_mut()
        .imports
        .push(ImportAssetEvent::Script(AssetImportEntry {
            name: "main_script".into(),
            data: Script { source: script },
        }));
    let script = std::fs::read_to_string("assets/script_utils.ms").expect("Failed to load.");
    system_status
        .borrow_mut()
        .imports
        .push(ImportAssetEvent::Script(AssetImportEntry {
            name: "utils_script".into(),
            data: Script { source: script },
        }));

    // Enter loop
    event_loop.run(move |event, _, control_flow| {
        // Update gui
        if display_mode == DisplayMode::WindowedUnfocus {
            gui.handle_event(&event, &mut mapper.borrow_mut(), &mut window);
        }

        // Match window events
        match event {
            Event::DeviceEvent {
                device_id: _,
                event,
            } => match event {
                DeviceEvent::MouseMotion { delta } => {
                    mouse_motion.0 += delta.0;
                    mouse_motion.1 += delta.1;
                }
                DeviceEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(x, y),
                } => {
                    wheel_motion.0 += x;
                    wheel_motion.1 += y;
                }
                _ => {}
            },
            Event::WindowEvent { window_id, event } => {
                if window_id == window.handle.id() {
                    match event {
                        WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    physical_key: PhysicalKey::Code(keycode),
                                    state,
                                    ..
                                },
                            ..
                        } => {
                            // Unfocus mouse
                            if state == ElementState::Pressed && keycode == KeyCode::Escape {
                                display_mode =
                                    set_display_mode(&mut window, DisplayMode::WindowedUnfocus);
                            }

                            // Save/Load state
                            if state == ElementState::Pressed && keycode == KeyCode::F5 {
                                save_state = true;
                            } else if state == ElementState::Pressed && keycode == KeyCode::F6 {
                                load_state = true;
                            }

                            // Toggle fullscreen
                            if state == ElementState::Pressed && keycode == KeyCode::F11 {
                                match display_mode {
                                    DisplayMode::FullscreenFocus => {
                                        display_mode = set_display_mode(
                                            &mut window,
                                            DisplayMode::WindowedFocus,
                                        );
                                    }
                                    _ => {
                                        display_mode = set_display_mode(
                                            &mut window,
                                            DisplayMode::FullscreenFocus,
                                        );
                                    }
                                }
                            }

                            // Dispatch keyboard
                            if window.is_focus() {
                                input
                                    .borrow_mut()
                                    .dispatch_keyboard(keycode, state == ElementState::Pressed);
                            }
                        }
                        WindowEvent::MouseInput {
                            device_id: _,
                            state,
                            button,
                            ..
                        } => {
                            // Focus mouse
                            if state == ElementState::Pressed
                                && button == MouseButton::Left
                                && !window.is_focus()
                            {
                                if last_click.is_none() {
                                    last_click = Some(SystemTime::now());
                                } else {
                                    display_mode =
                                        set_display_mode(&mut window, DisplayMode::WindowedFocus);
                                }
                            }

                            // Dispatch mouse
                            if window.is_focus() {
                                input
                                    .borrow_mut()
                                    .dispatch_mouse_button(button, state == ElementState::Pressed);
                            }
                        }
                        WindowEvent::CloseRequested => {
                            system_status.borrow_mut().stop();
                        }
                        WindowEvent::ScaleFactorChanged { .. } => {
                            let new_inner_size = window.handle.inner_size();
                            renderer
                                .borrow_mut()
                                .resize(new_inner_size.width, new_inner_size.height);
                        }
                        WindowEvent::Resized(_) => {
                            let inner_size = window.handle.inner_size();
                            renderer
                                .borrow_mut()
                                .resize(inner_size.width, inner_size.height);
                        }
                        WindowEvent::ReceivedCharacter(c) => {
                            if window.is_focus() {
                                input.borrow_mut().dispatch_text(c.to_string());
                            }
                        }
                        WindowEvent::CursorMoved {
                            device_id: _,
                            position,
                            ..
                        } => {
                            if window.is_focus() {
                                let position = renderer
                                    .get_mut()
                                    .cursor_position(position.x as f32, position.y as f32);
                                input.borrow_mut().dispatch_mouse_cursor(position);
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
            Event::MainEventsCleared => {
                // Dispatch mouse motion and reset
                if window.is_focus() {
                    if mouse_motion.0 != last_mouse_motion.0
                        || mouse_motion.1 != last_mouse_motion.1
                    {
                        mapper.borrow_mut().dispatch_mouse_motion(mouse_motion);
                        last_mouse_motion = mouse_motion;
                    }
                    if wheel_motion.0 != last_wheel_motion.0
                        || wheel_motion.1 != last_wheel_motion.1
                    {
                        mapper.borrow_mut().dispatch_mouse_wheel(wheel_motion);
                        last_wheel_motion = wheel_motion;
                    }
                }
                mouse_motion = (0.0, 0.0);
                wheel_motion = (0.0, 0.0);

                // Dispatch controller events
                while let Some(gilrs::Event { id, event, .. }) = &gilrs.next_event() {
                    if display_mode == DisplayMode::WindowedUnfocus {
                        gui.handle_controller_event(
                            event,
                            *id,
                            &mut mapper.borrow_mut(),
                            &mut window,
                        );
                    } else {
                        match event {
                            gilrs::EventType::ButtonPressed(button, _) => {
                                mapper
                                    .borrow_mut()
                                    .dispatch_controller_button(*id, *button, true);
                            }
                            gilrs::EventType::ButtonReleased(button, _) => {
                                mapper
                                    .borrow_mut()
                                    .dispatch_controller_button(*id, *button, false);
                            }
                            gilrs::EventType::AxisChanged(axis, value, _) => {
                                mapper
                                    .borrow_mut()
                                    .dispatch_controller_axis(*id, *axis, *value);
                            }
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
                let dt = (now - last_time).as_secs_f64();
                last_time = now;

                // Update GUI
                let last_display_mode = display_mode;
                gui.ui(
                    &mut window,
                    &mut mapper.borrow_mut(),
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

                // Progress instance
                instance.tick().expect("Failed to progress instance");

                // Save/Load state
                if save_state {
                    // {
                    //     let file = File::create("assets/state.json").expect("Failed to create file");
                    //     let mut serializer = serde_json::Serializer::new(file);
                    //     sim.save_state(&mut serializer).expect("Failed to serialize");
                    // }

                    {
                        let mut file = File::create("assets/state.bin").unwrap();
                        let mut bytes = Vec::<u8>::default();
                        instance.save(&mut bytes).unwrap();
                        let bytes = miniz_oxide::deflate::compress_to_vec_zlib(&bytes, 10);
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
                        let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes)
                            .expect("Failed to decompress");
                        let mut decoder = SliceDecoder::new(&bytes);
                        instance.load(&mut decoder).expect("Failed to load state");
                    }

                    // {
                    //     let mut file = File::open("assets/state_postcard.bin").expect("Failed to open file");
                    //     let mut bytes: Vec<u8> = Default::default();
                    //     file.read_to_end(&mut bytes).expect("Failed to read to end");
                    //     let bytes = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes).expect("Failed to decompress");
                    //     let mut deserializer = postcard::Deserializer::from_bytes(&bytes);
                    //     engine.load_state(&mut deserializer).expect("Failed to load state");
                    // }

                    load_state = false;
                }

                // Invoke WGPU Renderer
                renderer.borrow_mut().render().expect("Failed to render");

                // Check shutdown
                if !system_status.borrow_mut().running {
                    println!("Instance shutdown");
                    *control_flow = ControlFlow::Exit;
                }

                // Check exit
                if *control_flow != ControlFlow::Exit {
                    window.handle.request_redraw();
                }
            }
            _ => {}
        }
    });
}

fn main() {
    // {
    //     {
    //         let mut file = File::create("assets/state.bin").unwrap();
    //         let mut bytes = Vec::new();
    //         let vec = vec![Test {a: 1, b: 2}, Test {a: 3, b: 4}];
    //         <Test as mini3d_core::serialize::Serialize>::Header::default().serialize(&mut bytes).unwrap();
    //         vec.serialize(&mut bytes).unwrap();
    //         file.write_all(&bytes).unwrap();
    //     }
    //     {
    //         let mut file = File::open("assets/state.bin").expect("Failed to open file");
    //         let mut bytes: Vec<u8> = Default::default();
    //         file.read_to_end(&mut bytes).expect("Failed to read to end");
    //         let mut decoder = SliceDecoder::new(&bytes);
    //         let header = <Test as mini3d_core::serialize::Serialize>::Header::deserialize(&mut decoder, &Default::default()).unwrap();
    //         let vec = Vec::<Test>::deserialize(&mut decoder, &header).unwrap();
    //         println!("{:?}", vec);
    //     }
    // }
    main_run();
    // 2 * 4 + 1 * 5
    // let program = Program::empty()
    //     .put(Opcode::PUSHLW)
    //     .put_int(2)
    //     .put(Opcode::PUSHLW)
    //     .put_int(4)
    //     .put(Opcode::MULI)
    //     .put(Opcode::PUSHLW)
    //     .put_int(1)
    //     .put(Opcode::PUSHLW)
    //     .put_int(5)
    //     .put(Opcode::MULI)
    //     .put(Opcode::ADDI)
    //     .put(Opcode::INT);
    // let mut vm = VirtualMachine::new(program);
    // vm.run();
}
