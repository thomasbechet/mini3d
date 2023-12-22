use native_windows_gui::ControlHandle;
use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle, Win32WindowHandle,
    WindowsDisplayHandle,
};
use winapi::{
    ctypes::c_int,
    shared::{basetsd::LONG_PTR, windef::HWND},
    um::winuser::GWL_HINSTANCE,
};

#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub(crate) fn get_window_long(handle: HWND, index: c_int) -> LONG_PTR {
    unsafe { winapi::um::winuser::GetWindowLongPtrW(handle, index) }
}

#[inline(always)]
#[cfg(target_pointer_width = "32")]
pub(crate) fn get_window_long(handle: HWND, index: c_int) -> LONG {
    unsafe { winapi::um::winuser::GetWindowLongW(handle, index) }
}

#[inline(always)]
#[cfg(target_pointer_width = "64")]
pub(crate) fn set_window_long(handle: HWND, index: c_int, value: LONG_PTR) -> LONG_PTR {
    unsafe { winapi::um::winuser::SetWindowLongPtrW(handle, index, value) }
}

#[inline(always)]
#[cfg(target_pointer_width = "32")]
pub(crate) fn set_window_long(handle: HWND, index: c_int, value: LONG) -> LONG {
    unsafe { winapi::um::winuser::SetWindowLongW(handle, index, value) }
}

pub(crate) unsafe fn toggle_fullscreen(
    control: native_windows_gui::ControlHandle,
    fullscreen: bool,
) {
    let hwnd = match control {
        ControlHandle::Hwnd(hwnd) => hwnd,
        _ => panic!("Invalid canvas handle"),
    };
    println!("toggle fullscreen {}", fullscreen);
    if fullscreen {
        let w = winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CXSCREEN);
        let h = winapi::um::winuser::GetSystemMetrics(winapi::um::winuser::SM_CYSCREEN);
        set_window_long(
            hwnd,
            winapi::um::winuser::GWL_STYLE,
            (winapi::um::winuser::WS_VISIBLE | winapi::um::winuser::WS_POPUP)
                .try_into()
                .unwrap(),
        );
        winapi::um::winuser::SetWindowPos(
            hwnd,
            winapi::um::winuser::WM_NULL as _,
            0,
            0,
            w,
            h,
            winapi::um::winuser::SWP_FRAMECHANGED,
        );
    } else {
        let w = 800;
        let h = 600;
        set_window_long(
            hwnd,
            winapi::um::winuser::GWL_STYLE,
            (winapi::um::winuser::WS_VISIBLE | winapi::um::winuser::WS_OVERLAPPEDWINDOW)
                .try_into()
                .unwrap(),
        );
        winapi::um::winuser::SetWindowPos(
            hwnd,
            winapi::um::winuser::HWND_TOP,
            0,
            0,
            w,
            h,
            winapi::um::winuser::SWP_FRAMECHANGED,
        );
    }
}

#[derive(Default)]
pub(crate) struct Win32ControlHandle(native_windows_gui::ControlHandle);

impl Win32ControlHandle {
    pub(crate) fn new(handle: native_windows_gui::ControlHandle) -> Self {
        Self(handle)
    }
}

unsafe impl HasRawWindowHandle for Win32ControlHandle {
    fn raw_window_handle(&self) -> RawWindowHandle {
        match self.0 {
            ControlHandle::Hwnd(hwnd) => {
                let hinstance = get_window_long(hwnd, GWL_HINSTANCE);
                let mut handle = Win32WindowHandle::empty();
                handle.hwnd = hwnd as _;
                handle.hinstance = hinstance as _;
                RawWindowHandle::Win32(handle)
            }
            _ => panic!("Invalid canvas handle"),
        }
    }
}

unsafe impl HasRawDisplayHandle for Win32ControlHandle {
    fn raw_display_handle(&self) -> RawDisplayHandle {
        match self.0 {
            ControlHandle::Hwnd(_hwnd) => RawDisplayHandle::Windows(WindowsDisplayHandle::empty()),
            _ => panic!("Invalid canvas handle"),
        }
    }
}
