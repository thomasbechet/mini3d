// use libc::{c_void, c_ulong};
// use mini3d_core::{app::App, glam::{Vec2, UVec2, Vec4}, graphics::SCREEN_RESOLUTION, math::rect::IRect};
// use mini3d_wgpu::WGPURenderer;

// use crate::app::mini3d_app;

// #[repr(C)]
// pub struct mini3d_renderer(*mut c_void);

// pub enum RendererContext {
//     None,
//     Wgpu {
//         context: Box<WGPURenderer>
//     }
// }

// enum RawWindowHandle {
//     Win32(raw_window_handle::Win32Handle),
//     Xlib(raw_window_handle::XlibHandle)
// }

// unsafe impl raw_window_handle::HasRawWindowHandle for RawWindowHandle {
//     fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
//         match self {
//             RawWindowHandle::Win32(handle) => raw_window_handle::RawWindowHandle::Win32(*handle),
//             RawWindowHandle::Xlib(handle) => raw_window_handle::RawWindowHandle::Xlib(*handle),
//         }
//     }
// }

// #[no_mangle]
// pub extern "C" fn mini3d_renderer_new_wgpu_win32(hinstance: *mut c_void, hwnd: *mut c_void) -> *mut mini3d_renderer {
//     let mut handle = raw_window_handle::Win32Handle::empty();
//     handle.hinstance = hinstance;
//     handle.hwnd = hwnd;
//     Box::into_raw(Box::new(RendererContext::Wgpu { context: Box::new(WGPURenderer::new(&RawWindowHandle::Win32(handle))) })) as *mut mini3d_renderer
// }

// #[no_mangle]
// pub extern "C" fn mini3d_renderer_new_wgpu_xlib(window: c_ulong, display: *mut c_void) -> *mut mini3d_renderer {
//     let mut handle = raw_window_handle::XlibHandle::empty();
//     handle.window = window;
//     handle.display = display;
//     handle.visual_id = 0;
//     Box::into_raw(Box::new(RendererContext::Wgpu { context: Box::new(WGPURenderer::new(&RawWindowHandle::Xlib(handle))) })) as *mut mini3d_renderer
// }

// #[no_mangle]
// pub extern "C" fn mini3d_renderer_delete(renderer: *mut mini3d_renderer) {
//     unsafe { drop(Box::from_raw(renderer as *mut RendererContext)); }
// }

// #[no_mangle]
// #[allow(clippy::missing_safety_doc)]
// pub unsafe extern "C" fn mini3d_renderer_render(renderer: *mut mini3d_renderer, app: *const mini3d_app) -> bool {
//     let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
//     let app = (app as *const App).as_ref().unwrap();
//     match renderer {
//         RendererContext::None => { true },
//         RendererContext::Wgpu { context } => {
//             todo!();
//             // match context.render(app, Vec4::ZERO, |_, _, _, _| {}) {
//             //     Ok(_) => {
//             //         true
//             //     },
//             //     Err(e) => {
//             //         eprintln!("{:?}", e);
//             //         false
//             //     }
//             // }
//         },
//     }
// }

// #[no_mangle]
// #[allow(clippy::missing_safety_doc)]
// pub unsafe extern "C" fn mini3d_renderer_resize(renderer: *mut mini3d_renderer, width: u32, height: u32) {
//     let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
//     match renderer {
//         RendererContext::None => {},
//         RendererContext::Wgpu { context } => {
//             context.resize(width, height);
//         },
//     }
// }

// #[no_mangle]
// #[allow(clippy::missing_safety_doc)]
// pub unsafe extern "C" fn mini3d_renderer_recreate(renderer: *mut mini3d_renderer) {
//     let renderer = (renderer as *mut RendererContext).as_mut().unwrap();
//     match renderer {
//         RendererContext::None => {},
//         RendererContext::Wgpu { context } => {
//             context.recreate();
//         },
//     }
// }
