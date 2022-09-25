use libc::c_void;
use mini3d::{application::{self, Application}, input::button::ButtonState, event::FrameEvents, backend::BackendDescriptor};
use mini3d_os::program::OSProgram;

use crate::{renderer::{mini3d_renderer, RendererContext}, event::mini3d_event};

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
pub extern "C" fn mini3d_app_new() -> *mut mini3d_application {
    Box::into_raw(Box::new(application::Application::new::<OSProgram>(()))) as *mut mini3d_application
}

#[no_mangle]
pub extern "C" fn mini3d_application_delete(app: *mut mini3d_application) {
    unsafe { drop(Box::from_raw(app as *mut application::Application)); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_application_progress(
    app: *mut mini3d_application,
    events: *mut mini3d_event,
    renderer: *mut mini3d_renderer,
) -> bool {
    let app = (app as *mut Application).as_mut().unwrap();
    let events = (events as *mut FrameEvents).as_mut().unwrap();
    let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
    
    // Create the backend descriptor
    let mut backend_descriptor = BackendDescriptor::new();
    // Renderer backend
    match renderer {
        RendererContext::None => {},
        RendererContext::Wgpu { context } => {
            backend_descriptor = backend_descriptor.with_renderer(context.as_mut());
        },
    }
    // Progress the application
    app.progress(backend_descriptor, events).is_ok()
}