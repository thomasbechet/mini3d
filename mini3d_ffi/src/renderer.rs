use libc::{c_void, c_ulong};
use mini3d::{app::App, glam::{Vec2, UVec2}, graphics::SCREEN_RESOLUTION};
use mini3d_wgpu::{WGPURenderer, compute_fixed_viewport};

use crate::app::mini3d_app;

#[repr(C)] 
pub struct mini3d_renderer(*mut c_void);

pub enum RendererContext {
    None,
    Wgpu {
        context: Box<WGPURenderer>
    }
}

enum RawWindowHandle {
    Win32(raw_window_handle::Win32Handle),
    Xlib(raw_window_handle::XlibHandle)
}

unsafe impl raw_window_handle::HasRawWindowHandle for RawWindowHandle {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        match self {
            RawWindowHandle::Win32(handle) => raw_window_handle::RawWindowHandle::Win32(*handle),
            RawWindowHandle::Xlib(handle) => raw_window_handle::RawWindowHandle::Xlib(*handle),
        }
    }
}

#[no_mangle]
pub extern "C" fn mini3d_renderer_new_wgpu_win32(hinstance: *mut c_void, hwnd: *mut c_void) -> *mut mini3d_renderer {
    let mut handle = raw_window_handle::Win32Handle::empty();
    handle.hinstance = hinstance;
    handle.hwnd = hwnd;
    Box::into_raw(Box::new(RendererContext::Wgpu { context: Box::new(WGPURenderer::new(&RawWindowHandle::Win32(handle))) })) as *mut mini3d_renderer
}

#[no_mangle]
pub extern "C" fn mini3d_renderer_new_wgpu_xlib(window: c_ulong, display: *mut c_void) -> *mut mini3d_renderer {
    let mut handle = raw_window_handle::XlibHandle::empty();
    handle.window = window;
    handle.display = display;
    handle.visual_id = 0;
    Box::into_raw(Box::new(RendererContext::Wgpu { context: Box::new(WGPURenderer::new(&RawWindowHandle::Xlib(handle))) })) as *mut mini3d_renderer
}

#[no_mangle]
pub extern "C" fn mini3d_renderer_delete(renderer: *mut mini3d_renderer) {
    unsafe { drop(Box::from_raw(renderer as *mut RendererContext)); }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_renderer_render(renderer: *mut mini3d_renderer, app: *const mini3d_app) -> bool {
    let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
    let app = (app as *const App).as_ref().unwrap();
    match renderer {
        RendererContext::None => { true },
        RendererContext::Wgpu { context } => {
            match context.render(app) {
                Ok(_) => {
                    true
                },
                Err(e) => {
                    eprintln!("{:?}", e);
                    false
                } 
            }
        },
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_renderer_resize(renderer: *mut mini3d_renderer, width: u32, height: u32) {
    let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
    match renderer {
        RendererContext::None => {},
        RendererContext::Wgpu { context } => {
            context.resize(width, height);
        },
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_renderer_recreate(renderer: *mut mini3d_renderer) {
    let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
    match renderer {
        RendererContext::None => {},
        RendererContext::Wgpu { context } => {
            context.recreate();
        },
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn mini3d_utils_inner_to_viewport_position(
    x: f32, y: f32,
    width: u32, height: u32,
    vx: *mut f32, vy: *mut f32,
) {
    let position = Vec2::new(x, y);
    let wsize = UVec2::new(width, height);
    let viewport = compute_fixed_viewport(wsize);
    let rel_position = position - Vec2::new(viewport.x, viewport.y);
    let final_position = (rel_position / Vec2::new(viewport.z, viewport.w)) * SCREEN_RESOLUTION.as_vec2();
    *vx = final_position.x;
    *vy = final_position.y;
}