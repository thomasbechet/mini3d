use libc::c_void;

#[repr(C)] 
pub struct mini3d_app_requests(*mut c_void);

// #[no_mangle]
// pub extern "C" fn mini3d_app_requests_new() -> *mut mini3d_app_requests {
//     Box::into_raw(Box::new(Requests::new())) as *mut mini3d_app_requests
// }

// #[no_mangle]
// pub extern "C" fn mini3d_app_requests_delete(requests: *mut mini3d_app_requests) {
//     unsafe { drop(Box::from_raw(requests as *mut Requests)); }
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_app_requests_shutdown(requests: *const mini3d_app_requests) -> bool {
//     let requests = (requests as *const Requests).as_ref().unwrap();
//     requests.shutdown()
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_app_requests_reload_input_mapping(requests: *const mini3d_app_requests) -> bool {
//     let requests = (requests as *const Requests).as_ref().unwrap();
//     requests.reload_input_mapping()
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_app_requests_reset(requests: *mut mini3d_app_requests) {
//     let requests = (requests as *mut Requests).as_mut().unwrap();
//     requests.reset();
// }