use libc::c_void;
use mini3d::{application::{self, Application}, input::event::ButtonState, event_recorder::EventRecorder};

use crate::event_recorder::mini3d_event_recorder;

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
pub struct mini3d_application(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_application_new() -> *mut mini3d_application {
    Box::into_raw(Box::new(application::Application::default())) as *mut mini3d_application
}

#[no_mangle]
pub extern "C" fn mini3d_application_delete(app: *mut mini3d_application) {
    unsafe { Box::from_raw(app as *mut application::Application); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_application_progress(app: *mut mini3d_application, recorder: *const mini3d_event_recorder) {
    let app = (app as *mut Application).as_mut().unwrap();
    let recorder = (recorder as *const EventRecorder).as_ref().unwrap();
    app.progress(recorder);
}