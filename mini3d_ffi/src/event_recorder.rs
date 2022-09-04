use std::{ffi::CStr, slice};

use libc::c_void;
use mini3d::{event_recorder::EventRecorder, input::event::{InputEvent, ButtonEvent, AxisEvent, CursorEvent}, glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION};
use mini3d_wgpu::compute_viewport;

use crate::application::mini3d_button_state;

#[repr(C)] 
pub struct mini3d_event_recorder(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_event_recorder_new() -> *mut mini3d_event_recorder {
    Box::into_raw(Box::new(EventRecorder::default())) as *mut mini3d_event_recorder
}

#[no_mangle]
pub extern "C" fn mini3d_event_recorder_delete(app: *mut mini3d_event_recorder) {
    unsafe { Box::from_raw(app as *mut EventRecorder); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_event_recorder_reset(recorder: *mut mini3d_event_recorder) {
    let recorder = recorder as *mut EventRecorder;
    (&mut *recorder).reset();
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_record_input_button(recorder: *mut mini3d_event_recorder, name: *const libc::c_char, state: mini3d_button_state) {
    let recorder = recorder as *mut EventRecorder;
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    (&mut *recorder).push_input_event(InputEvent::Button(ButtonEvent {name, state: state.into() }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_record_input_axis(recorder: *mut mini3d_event_recorder, name: *const libc::c_char, value: f32) {
    let recorder = recorder as *mut EventRecorder;
    let name = CStr::from_ptr(name).to_str().expect("Invalid");
    (&mut *recorder).push_input_event(InputEvent::Axis(AxisEvent {name, value}));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_record_input_cursor_move(recorder: *mut mini3d_event_recorder, delta: *const f32) {
    let recorder = recorder as *mut EventRecorder;
    let delta = slice::from_raw_parts(delta, 2);
    (&mut *recorder).push_input_event(InputEvent::Cursor(CursorEvent::Move { delta: Vec2::from_slice(delta) }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_record_input_cursor_position(recorder: *mut mini3d_event_recorder, x: f32, y: f32, width: u32, height: u32) {
    let recorder = recorder as *mut EventRecorder;

    let p: Vec2 = (x, y).into();
    let wsize: UVec2 = (width, height).into();
    let viewport = compute_viewport(wsize);
    let relp = p - Vec2::new(viewport.x, viewport.y);
    let position: Vec2 = (relp / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();

    (&mut *recorder).push_input_event(InputEvent::Cursor(CursorEvent::Update { position }));
}