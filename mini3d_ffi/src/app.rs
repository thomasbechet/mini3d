use std::{ffi::CStr, slice};

use libc::c_void;
use mini3d::{app::{self, App}, event::{Event, InputEvent}, input::event::{ButtonEvent, AxisEvent, CursorEvent, ButtonState}, glam::Vec2};

use crate::renderer::{mini3d_renderer, RendererContext};

#[repr(C)]
pub enum mini3d_button_state {
    Pressed,
    Released
}

impl From<mini3d_button_state> for ButtonState {
    fn from(state: mini3d_button_state) -> Self {
        match state {
            mini3d_button_state::Pressed => ButtonState::Pressed,
            mini3d_button_state::Released => ButtonState::Released,
        }
    }
}

#[repr(C)] 
pub struct mini3d_app(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_app_new() -> *mut mini3d_app {
    Box::into_raw(Box::new(app::App::default())) as *mut mini3d_app
}

#[no_mangle]
pub extern "C" fn mini3d_app_delete(app: *mut mini3d_app) {
    unsafe { Box::from_raw(app as *mut app::App); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_render(app: *mut mini3d_app, renderer: *mut mini3d_renderer) {
    let app = app as *mut App;
    let renderer = renderer as *mut RendererContext;
    match &mut *renderer {
        RendererContext::None => {},
        RendererContext::Wgpu { context } => {
            (*app).render(context.as_mut());
        },
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_push_close_requested(app: *mut mini3d_app) {
    let app = app as *mut App;
    (&mut *app).push_event(Event::CloseRequested);
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_push_input_button(app: *mut mini3d_app, name: *const libc::c_char, state: mini3d_button_state) {
    let app = app as *mut App;
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    (&mut *app).push_event(Event::Input(InputEvent::Button(ButtonEvent {name, state: state.into() })));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_push_input_axis(app: *mut mini3d_app, name: *const libc::c_char, value: f32) {
    let app = app as *mut App;
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    (&mut *app).push_event(Event::Input(InputEvent::Axis(AxisEvent {name, value})));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_push_input_cursor_move(app: *mut mini3d_app, delta: *const f32) {
    let app = app as *mut App;
    let delta = slice::from_raw_parts(delta, 2);
    (&mut *app).push_event(Event::Input(InputEvent::Cursor(CursorEvent::Move { delta: Vec2::from_slice(delta) })));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_push_input_cursor_position(app: *mut mini3d_app, position: *const f32) {
    let app = app as *mut App;
    let position = slice::from_raw_parts(position, 2);
    (&mut *app).push_event(Event::Input(InputEvent::Cursor(CursorEvent::Update { position: Vec2::from_slice(position) })));
}