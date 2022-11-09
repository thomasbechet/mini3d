// use core::slice;
// use std::ffi::CString;

// use libc::c_void;
// use mini3d::{app::App, slotmap::KeyData};

// use crate::app::mini3d_app;

// #[repr(C)]
// pub struct mini3d_input_database {
//     actions: *mut u64,
//     action_count: u32,
//     axis: *mut u64,
//     axis_count: u32,
//     groups: *mut u64,
//     group_count: u32,
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_input_database_read(app: *const mini3d_app) -> mini3d_input_database {
//     let app = (app as *const App).as_ref().unwrap();
//     // Recover inputs
//     let actions = InputDatabase::iter_actions(app).collect::<Vec<_>>();
//     let axis = InputDatabase::iter_axis(app).collect::<Vec<_>>();
//     let groups = InputDatabase::iter_groups(app).collect::<Vec<_>>();
//     // Allocate memory
//     let database = mini3d_input_database {
//         actions: libc::malloc(std::mem::size_of::<u64>() * actions.len()) as *mut u64,
//         action_count: actions.len() as u32,
//         axis: libc::malloc(std::mem::size_of::<u64>() * axis.len()) as *mut u64,
//         axis_count: axis.len() as u32,
//         groups: libc::malloc(std::mem::size_of::<u64>() * groups.len()) as *mut u64,
//         group_count: groups.len() as u32,
//     };
//     // Copy ids
//     for (i, id) in actions.iter().enumerate() {
//         *database.actions.offset(i as isize) = KeyData::as_ffi(id.data());
//     }
//     for (i, id) in axis.iter().enumerate() {
//         *database.axis.offset(i as isize) = KeyData::as_ffi(id.data());
//     }
//     for (i, id) in groups.iter().enumerate() {
//         *database.groups.offset(i as isize) = KeyData::as_ffi(id.data());
//     }
//     // Return database
//     database
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_input_database_free(inputs: *mut mini3d_input_database) {
//     libc::free((*inputs).actions as *mut c_void);
//     libc::free((*inputs).axis as *mut c_void);
//     libc::free((*inputs).groups as *mut c_void);
// }

// #[repr(C)]
// pub struct mini3d_input_action {
//     name: [libc::c_char; 128],
//     group: u64,
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_input_database_get_action(
//     app: *const mini3d_app, 
//     id: u64,
//     action: *mut mini3d_input_action,
// ) -> libc::c_int {
//     let app = (app as *const App).as_ref().unwrap();
//     let b = InputDatabase::action(app, ActionInputId::from(KeyData::from_ffi(id))).unwrap();
//     // Convert name string
//     let bytes = match CString::new(b.descriptor.name.clone()) {
//         Ok(s) => s,
//         Err(_) => return -1,
//     };
//     let bytes = bytes.as_bytes_with_nul();
//     let bytes = slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len());
//     (*action).name[..bytes.len()].copy_from_slice(bytes);
//     (*action).group = KeyData::as_ffi(b.group.data());
//     0    
// }

// #[repr(C)]
// pub struct mini3d_input_axis {
//     name: [libc::c_char; 128],
//     group: u64,
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_input_database_get_axis(
//     app: *const mini3d_app, 
//     id: u64,
//     axis: *mut mini3d_input_axis,
// ) -> libc::c_int {
//     let app = (app as *const App).as_ref().unwrap();
//     let a = InputDatabase::axis(app, AxisInputId::from(KeyData::from_ffi(id))).unwrap();
//     // Convert name string
//     let bytes = match CString::new(a.descriptor.name.clone()) {
//         Ok(s) => s,
//         Err(_) => return -1,
//     };
//     let bytes = bytes.as_bytes_with_nul();
//     let bytes = slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len());
//     (*axis).name[..bytes.len()].copy_from_slice(bytes);
//     (*axis).group = KeyData::as_ffi(a.group.data());
//     0
// }

// #[repr(C)]
// pub struct mini3d_input_group {
//     name: [libc::c_char; 128],
// }

// #[no_mangle]
// pub unsafe extern "C" fn mini3d_input_database_get_group(
//     app: *const mini3d_app, 
//     id: u64,
//     group: *mut mini3d_input_group,
// ) -> libc::c_int {
//     let app = (app as *const App).as_ref().unwrap();
//     let g = InputDatabase::group(app, InputGroupId::from(KeyData::from_ffi(id))).unwrap();
//     // Convert name string
//     let bytes = match CString::new(g.name.clone()) {
//         Ok(s) => s,
//         Err(_) => return -1,
//     };
//     let bytes = bytes.as_bytes_with_nul();
//     let bytes = slice::from_raw_parts(bytes.as_ptr() as *const i8, bytes.len());
//     (*group).name[..bytes.len()].copy_from_slice(bytes);
//     0
// }