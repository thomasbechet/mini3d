// use libc::c_void;
// use mini3d_core::event::{input::{InputEvent, InputActionEvent, InputAxisEvent}, Events};

// #[repr(C)]
// pub enum mini3d_action_state {
//     Pressed,
//     Released
// }

// #[repr(C)]
// pub struct mini3d_app_events(*mut c_void);

// #[no_mangle]
// pub extern "C" fn mini3d_app_events_new() -> *mut mini3d_app_events {
//     Box::into_raw(Box::new(Events::new())) as *mut mini3d_app_events
// }

// #[no_mangle]
// pub extern "C" fn mini3d_app_events_delete(event: *mut mini3d_app_events) {
//     unsafe { drop(Box::from_raw(event as *mut Events)); }
// }

// #[no_mangle]
// #[allow(clippy::missing_safety_doc)]
// pub unsafe extern "C" fn mini3d_app_events_push_input_action(
//     event: *mut mini3d_app_events,
//     uid: u64,
//     pressed: bool,
// ) {
//     let event = (event as *mut Events).as_mut().unwrap();
//     event.input.push(InputEvent::Action(InputActionEvent {
//         action: uid.into(),
//         pressed,
//     }));
// }

// #[no_mangle]
// #[allow(clippy::missing_safety_doc)]
// pub unsafe extern "C" fn mini3d_app_events_push_input_axis(
//     event: *mut mini3d_app_events,
//     uid: u64,
//     value: f32,
// ) {
//     let event = (event as *mut Events).as_mut().unwrap();
//     event.input.push(InputEvent::Axis(InputAxisEvent {
//         axis: uid.into(),
//         value,
//     }));
// }
