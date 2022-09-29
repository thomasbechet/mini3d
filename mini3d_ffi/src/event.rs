use std::slice;

use libc::c_void;
use mini3d::{glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION, input::{button::{ButtonState, ButtonInputId}, axis::AxisInputId}, event::{input::{InputEvent, ButtonEvent, AxisEvent, MouseEvent}, AppEvents}, slotmap::KeyData};
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
pub struct mini3d_app_events(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_app_events_new() -> *mut mini3d_app_events {
    Box::into_raw(Box::new(AppEvents::new())) as *mut mini3d_app_events
}

#[no_mangle]
pub extern "C" fn mini3d_app_events_delete(event: *mut mini3d_app_events) {
    unsafe { drop(Box::from_raw(event as *mut AppEvents)); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_events_push_input_button(
    event: *mut mini3d_app_events, 
    id: libc::c_ulong,
    state: mini3d_button_state,
) {
    let event = (event as *mut AppEvents).as_mut().unwrap();
    event.push_input(InputEvent::Button(ButtonEvent { 
        id: ButtonInputId::from(KeyData::from_ffi(id)), 
        state: state.into() 
    }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_events_push_input_axis(
    event: *mut mini3d_app_events, 
    id: libc::c_ulong,
    value: f32,
) {
    let event = (event as *mut AppEvents).as_mut().unwrap();
    event.push_input(InputEvent::Axis(AxisEvent {
        id: AxisInputId::from(KeyData::from_ffi(id)), 
        value,
    }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_events_push_input_mouse_move(
    event: *mut mini3d_app_events, 
    delta: *const f32
) {
    let event = (event as *mut AppEvents).as_mut().unwrap();
    let delta = slice::from_raw_parts(delta, 2);
    event.push_input(InputEvent::Mouse(MouseEvent::Move { delta: Vec2::from_slice(delta) }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_events_push_input_mouse_position(
    event: *mut mini3d_app_events, 
    x: f32, 
    y: f32, 
    width: u32, 
    height: u32
) {
    let event = (event as *mut AppEvents).as_mut().unwrap();

    let p: Vec2 = (x, y).into();
    let wsize: UVec2 = (width, height).into();
    let viewport = compute_fixed_viewport(wsize);
    let relp = p - Vec2::new(viewport.x, viewport.y);
    let position: Vec2 = (relp / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();

    event.push_input(InputEvent::Mouse(MouseEvent::Update { position }));
}