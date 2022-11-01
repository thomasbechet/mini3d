use libc::c_void;
use mini3d::{input::{action::{ActionState, ActionInputId}, axis::AxisInputId}, event::{input::{InputEvent, InputActionEvent, InputAxisEvent}, AppEvents}, slotmap::KeyData};

#[repr(C)]
pub enum mini3d_action_state {
    Pressed,
    Released
}

impl From<mini3d_action_state> for ActionState {
    fn from(state: mini3d_action_state) -> Self {
        match state {
            mini3d_action_state::Pressed => ActionState::Pressed,
            mini3d_action_state::Released => ActionState::Released,
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
pub unsafe extern "C" fn mini3d_app_events_push_input_action(
    event: *mut mini3d_app_events, 
    id: u64,
    state: mini3d_action_state,
) {
    let event = (event as *mut AppEvents).as_mut().unwrap();
    event.push_input(InputEvent::Action(InputActionEvent { 
        id: ActionInputId::from(KeyData::from_ffi(id)), 
        state: state.into() 
    }));
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_events_push_input_axis(
    event: *mut mini3d_app_events, 
    id: u64,
    value: f32,
) {
    let event = (event as *mut AppEvents).as_mut().unwrap();
    event.push_input(InputEvent::Axis(InputAxisEvent {
        id: AxisInputId::from(KeyData::from_ffi(id)), 
        value,
    }));
}