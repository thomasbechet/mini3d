use libc::c_void;
use mini3d_wgpu::WGPUContext;

#[repr(C)] 
pub struct mini3d_renderer(*mut c_void);

pub enum RendererContext {
    None,
    Wgpu {
        context: Box<WGPUContext>
    }
}

struct RawWindowHandle(raw_window_handle::Win32Handle);

unsafe impl raw_window_handle::HasRawWindowHandle for RawWindowHandle {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        raw_window_handle::RawWindowHandle::Win32(self.0)
    }
}

#[no_mangle]
pub extern "C" fn mini3d_renderer_new_wgpu_win32(hinstance: *mut c_void, hwnd: *mut c_void) -> *mut mini3d_renderer {
    let mut handle = raw_window_handle::Win32Handle::empty();
    handle.hinstance = hinstance;
    handle.hwnd = hwnd; 
    Box::into_raw(Box::new(RendererContext::Wgpu { context: Box::new(WGPUContext::new(&RawWindowHandle(handle))) })) as *mut mini3d_renderer
}

#[no_mangle]
pub extern "C" fn mini3d_renderer_delete(app: *mut mini3d_renderer) {
    unsafe { Box::from_raw(app as *mut RendererContext); }
}