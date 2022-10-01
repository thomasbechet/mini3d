use libc::c_void;
use mini3d::{app::{self, App}, input::action::ActionState, event::AppEvents, backend::BackendDescriptor, request::AppRequests};
use mini3d_os::program::OSProgram;

use crate::{renderer::{mini3d_renderer, RendererContext}, event::mini3d_app_events, request::mini3d_app_requests};

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
pub struct mini3d_app(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_app_new() -> *mut mini3d_app {
    let app = app::App::new::<OSProgram>(()).unwrap(); 
    Box::into_raw(Box::new(app)) as *mut mini3d_app
}

#[no_mangle]
pub extern "C" fn mini3d_app_delete(app: *mut mini3d_app) {
    unsafe { drop(Box::from_raw(app as *mut app::App)); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_app_progress(
    app: *mut mini3d_app,
    events: *mut mini3d_app_events,
    requests: *mut mini3d_app_requests,
    renderer: *mut mini3d_renderer,
    delta_time: f32,
) -> bool {
    let app = (app as *mut App).as_mut().unwrap();
    let events = (events as *mut AppEvents).as_mut().unwrap();
    let requests = (requests as *mut AppRequests).as_mut().unwrap();
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
    app.progress(backend_descriptor, events, requests, delta_time).is_ok()
}