use std::{ffi::CStr, slice};

use mini3d::{app, event::{Event, InputEvent}, input::event::{self, ButtonEvent, AxisEvent, CursorEvent}, glam::Vec2};

#[repr(C)]
pub enum ButtonState {
    Pressed,
    Released
}

impl Into<event::ButtonState> for ButtonState {
    fn into(self) -> event::ButtonState {
        match self {
            ButtonState::Pressed => event::ButtonState::Pressed,
            ButtonState::Released => event::ButtonState::Released,
        }
    }
}

#[repr(C)] 
pub struct App { private: [u8; 0] }

#[no_mangle]
pub extern "C" fn mini3d_app_new() -> *mut App {
    Box::into_raw(Box::new(app::App::new())) as *mut App
}

#[no_mangle]
pub extern "C" fn mini3d_app_delete(app: *mut App) {
    unsafe { Box::from_raw(app as *mut app::App); }
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_push_close_requested(app: *mut App) {
    let app = app as *mut app::App;
    (&mut *app).push_event(Event::CloseRequested);
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_push_input_button(app: *mut App, name: *const libc::c_char, state: ButtonState) {
    let app = app as *mut app::App;
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    (&mut *app).push_event(Event::Input(InputEvent::Button(ButtonEvent {name, state: state.into()})));
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_push_input_axis(app: *mut App, name: *const libc::c_char, value: f32) {
    let app = app as *mut app::App;
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    (&mut *app).push_event(Event::Input(InputEvent::Axis(AxisEvent {name, value})));
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_push_input_cursor_move(app: *mut App, delta: *const f32) {
    let app = app as *mut app::App;
    let delta = slice::from_raw_parts(delta, 2);
    (&mut *app).push_event(Event::Input(InputEvent::Cursor(CursorEvent::Move { delta: Vec2::from_slice(delta) })));
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_push_input_cursor_position(app: *mut App, position: *const f32) {
    let app = app as *mut app::App;
    let position = slice::from_raw_parts(position, 2);
    (&mut *app).push_event(Event::Input(InputEvent::Cursor(CursorEvent::Update { position: Vec2::from_slice(position) })));
}