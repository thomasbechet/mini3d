use libc::c_void;
use mini3d::{input::{InputDatabase, button::ButtonInputId, axis::AxisInputId, InputGroupId}, app::App, slotmap::{KeyData, Key}};

use crate::app::mini3d_app;

#[repr(C)]
pub struct mini3d_input_database {
    buttons: *mut u64,
    button_count: usize,
    axis: *mut u64,
    axis_count: usize,
    groups: *mut u64,
    group_count: usize,
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_input_database_read(app: *const mini3d_app) -> mini3d_input_database {
    let app = (app as *const App).as_ref().unwrap();
    // Recover inputs
    let buttons = InputDatabase::iter_buttons(app).collect::<Vec<_>>();
    let axis = InputDatabase::iter_axis(app).collect::<Vec<_>>();
    let groups = InputDatabase::iter_groups(app).collect::<Vec<_>>();
    // Allocate memory
    let database = mini3d_input_database {
        buttons: libc::malloc(std::mem::size_of::<u64>() * buttons.len()) as *mut u64,
        button_count: buttons.len(),
        axis: libc::malloc(std::mem::size_of::<u64>() * axis.len()) as *mut u64,
        axis_count: axis.len(),
        groups: libc::malloc(std::mem::size_of::<u64>() * groups.len()) as *mut u64,
        group_count: groups.len(),
    };
    // Copy ids
    for (i, id) in buttons.iter().enumerate() {
        *database.buttons.offset(i as isize) = KeyData::as_ffi(id.data());
    }
    for (i, id) in axis.iter().enumerate() {
        *database.axis.offset(i as isize) = KeyData::as_ffi(id.data());
    }
    for (i, id) in groups.iter().enumerate() {
        *database.groups.offset(i as isize) = KeyData::as_ffi(id.data());
    }
    // Return database
    database
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_input_database_free(inputs: *mut mini3d_input_database) {
    libc::free((*inputs).buttons as *mut c_void);
    libc::free((*inputs).axis as *mut c_void);
    libc::free((*inputs).groups as *mut c_void);
}

#[repr(C)]
pub struct mini3d_input_button {
    name: *const libc::c_char,
    group: u64,
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_input_button(app: *const mini3d_app, id: u64) -> mini3d_input_button {
    let app = (app as *const App).as_ref().unwrap();
    let b = InputDatabase::button(app, ButtonInputId::from(KeyData::from_ffi(id))).unwrap();
    mini3d_input_button {
        name: b.name.as_ptr() as *const libc::c_char,
        group: KeyData::as_ffi(b.group.data()),
    }
}

#[repr(C)]
pub struct mini3d_input_axis {
    name: *const libc::c_char,
    group: u64,
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_input_axis(app: *const mini3d_app, id: u64) -> mini3d_input_axis {
    let app = (app as *const App).as_ref().unwrap();
    let a = InputDatabase::axis(app, AxisInputId::from(KeyData::from_ffi(id))).unwrap();
    mini3d_input_axis {
        name: a.name.as_ptr() as *const libc::c_char,
        group: KeyData::as_ffi(a.group.data()),
    }
}

#[repr(C)]
pub struct mini3d_input_group {
    name: *const libc::c_char,
}

#[no_mangle]
pub unsafe extern "C" fn mini3d_input_group(app: *const mini3d_app, id: u64) -> mini3d_input_group {
    let app = (app as *const App).as_ref().unwrap();
    let g = InputDatabase::group(app, InputGroupId::from(KeyData::from_ffi(id))).unwrap();
    mini3d_input_group {
        name: g.name.as_ptr() as *const libc::c_char,
    }
}