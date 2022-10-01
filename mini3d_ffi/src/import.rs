use std::{path::Path, ffi::CStr};

use mini3d::event::AppEvents;
use mini3d_utils::{image::ImageImporter, model::ModelImporter};

use crate::event::mini3d_app_events;

#[repr(C)]
pub struct mini3d_utils_import_image_info {
    source: *const libc::c_char,
    name: *const libc::c_char,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_utils_import_image(
    info: *const mini3d_utils_import_image_info,
    events: *mut mini3d_app_events,
) {
    let events = (events as *mut AppEvents).as_mut().unwrap();
    let c_source = CStr::from_ptr((*info).source).to_str().unwrap();
    let c_name = CStr::from_ptr((*info).name).to_str().unwrap();
    ImageImporter::new()
        .from_source(Path::new(c_source))
        .with_name(c_name)
        .import().expect("Failed to import image")
        .push(events);
}

#[repr(C)]
pub struct mini3d_utils_import_model_info {
    obj_source: *const libc::c_char,
    name: *const libc::c_char,
    flat_normals: bool,
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_utils_import_model(
    info: *const mini3d_utils_import_model_info,
    events: *mut mini3d_app_events,
) {
    let events = (events as *mut AppEvents).as_mut().unwrap();
    let c_obj_source = CStr::from_ptr((*info).obj_source).to_str().unwrap();
    let c_name = CStr::from_ptr((*info).name).to_str().unwrap();
    ModelImporter::new()
        .from_obj(Path::new(c_obj_source))
        .with_flat_normals((*info).flat_normals)
        .with_name(c_name)
        .import().expect("Failed to import model")
        .push(events);  
}