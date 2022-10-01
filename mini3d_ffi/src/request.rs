use libc::c_void;
use mini3d::request::AppRequests;

#[repr(C)] 
pub struct mini3d_app_requests(*mut c_void);

#[no_mangle]
pub extern "C" fn mini3d_app_requests_new() -> *mut mini3d_app_requests {
    Box::into_raw(Box::new(AppRequests::new())) as *mut mini3d_app_requests
}

#[no_mangle]
pub extern "C" fn mini3d_app_requests_delete(requests: *mut mini3d_app_requests) {
    unsafe { drop(Box::from_raw(requests as *mut AppRequests)); }
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_requests_shutdown(requests: *const mini3d_app_requests) -> bool {
    let requests = (requests as *const AppRequests).as_ref().unwrap();
    requests.shutdown()
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_requests_reload_bindings(requests: *const mini3d_app_requests) -> bool {
    let requests = (requests as *const AppRequests).as_ref().unwrap();
    requests.reload_bindings()
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_app_requests_reset(requests: *mut mini3d_app_requests) {
    let requests = (requests as *mut AppRequests).as_mut().unwrap();
    requests.reset();
}