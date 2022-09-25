use std::{ffi::CStr, slice};

use libc::c_void;
use mini3d::{glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION, input::button::ButtonState, event::{input::{InputEvent, ButtonEvent, AxisEvent, CursorEvent}, FrameEvents}};
use mini3d_wgpu::compute_fixed_viewport;

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
pub struct mini3d_event(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_event_new() -> *mut mini3d_event {
    Box::into_raw(Box::new(FrameEvents::new())) as *mut mini3d_event
}

#[no_mangle]
pub extern "C" fn mini3d_event_delete(event: *mut mini3d_event) {
    unsafe { drop(Box::from_raw(event as *mut FrameEvents)); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_event_push_input_button(event: *mut mini3d_event, name: *const libc::c_char, state: mini3d_button_state) {
    let event = (event as *mut FrameEvents).as_mut().unwrap();
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    event.push_input(InputEvent::Button(ButtonEvent {name, state: state.into() }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_event_push_input_axis(event: *mut mini3d_event, name: *const libc::c_char, value: f32) {
    let event = (event as *mut FrameEvents).as_mut().unwrap();
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    event.push_input(InputEvent::Axis(AxisEvent {name, value}));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_event_push_input_cursor_move(event: *mut mini3d_event, delta: *const f32) {
    let event = (event as *mut FrameEvents).as_mut().unwrap();
    let delta = slice::from_raw_parts(delta, 2);
    event.push_input(InputEvent::Cursor(CursorEvent::Move { delta: Vec2::from_slice(delta) }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_event_push_input_cursor_position(event: *mut mini3d_event, x: f32, y: f32, width: u32, height: u32) {
    let event = (event as *mut FrameEvents).as_mut().unwrap();

    let p: Vec2 = (x, y).into();
    let wsize: UVec2 = (width, height).into();
    let viewport = compute_fixed_viewport(wsize);
    let relp = p - Vec2::new(viewport.x, viewport.y);
    let position: Vec2 = (relp / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();

    event.push_input(InputEvent::Cursor(CursorEvent::Update { position }));
}