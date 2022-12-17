use std::collections::HashMap;

use mini3d::{glam::{UVec2, IVec2}, renderer::{backend::{TextureHandle, CanvasSpriteHandle, CanvasViewportHandle, CanvasPrimitiveHandle, CanvasHandle, SceneCameraHandle}, color::Color}, math::rect::IRect, uid::UID};

use crate::{context::WGPUContext, blit_bind_group::create_blit_bind_group};

fn create_color_view(context: &WGPUContext, extent: &wgpu::Extent3d, format: wgpu::TextureFormat) -> wgpu::TextureView {
    let color_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("canvas_color_texture"),
    });
    color_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

fn create_depth_view(context: &WGPUContext, extent: &wgpu::Extent3d, format: wgpu::TextureFormat) -> wgpu::TextureView {
    let depth_texture = context.device.create_texture(&wgpu::TextureDescriptor {
        size: *extent,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        label: Some("canvas_depth_texture"),
    });
    depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
}

pub(crate) struct CanvasSprite {
    pub(crate) texture: TextureHandle,
    pub(crate) z_index: i32,
    pub(crate) position: IVec2,
    pub(crate) extent: IRect,
    pub(crate) color: Color,
}

pub(crate) struct CanvasPrimitive {
    pub(crate) position: IVec2,
}

pub(crate) struct CanvasViewport {
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) camera: Option<SceneCameraHandle>,
    pub(crate) position: IVec2,
    pub(crate) z_index: i32,
}

impl CanvasViewport {

    pub(crate) fn new(context: &WGPUContext, position: IVec2, resolution: UVec2) -> Self {
        
        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };

        Self {
            extent,
            color_view: create_color_view(context, &extent, VIEWPORT_COLOR_FORMAT),
            depth_view: create_depth_view(context, &extent, VIEWPORT_DEPTH_FORMAT),
            camera: None,
            position,
            z_index: 0,
        }
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        self.extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };
        self.color_view = create_color_view(context, &self.extent, VIEWPORT_COLOR_FORMAT);
        self.depth_view = create_depth_view(context, &self.extent, VIEWPORT_DEPTH_FORMAT);
    }

    pub(crate) fn aspect_ratio(&self) -> f32 {
        self.extent.width as f32 / self.extent.height as f32
    }
}

pub(crate) const CANVAS_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const CANVAS_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub(crate) const VIEWPORT_COLOR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;
pub(crate) const VIEWPORT_DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth24Plus;

pub(crate) struct Canvas {
    pub(crate) extent: wgpu::Extent3d,
    pub(crate) clear_color: wgpu::Color,
    pub(crate) color_view: wgpu::TextureView,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) sprites: HashMap<CanvasSpriteHandle, CanvasSprite>,
    pub(crate) viewports: HashMap<CanvasViewportHandle, CanvasViewport>,
    pub(crate) primitives: HashMap<CanvasPrimitiveHandle, CanvasPrimitive>,
}

impl Canvas {

    pub(crate) fn new(context: &WGPUContext, resolution: UVec2) -> Self {

        let extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };
    
        Self {
            extent,
            clear_color: wgpu::Color::TRANSPARENT,
            color_view: create_color_view(context, &extent, CANVAS_COLOR_FORMAT),
            depth_view: create_depth_view(context, &extent, CANVAS_DEPTH_FORMAT),
            sprites: Default::default(),
            viewports: Default::default(),
            primitives: Default::default(),
        }
    }

    pub(crate) fn resize(&mut self, context: &WGPUContext, resolution: UVec2) {
        self.extent = wgpu::Extent3d {
            width: resolution.x,
            height: resolution.y,
            depth_or_array_layers: 1
        };
        self.color_view = create_color_view(context, &self.extent, CANVAS_COLOR_FORMAT);
        self.depth_view = create_depth_view(context, &self.extent, CANVAS_DEPTH_FORMAT);
    }
}

pub(crate) struct SurfaceCanvas {
    pub(crate) canvas: CanvasHandle,
    pub(crate) position: IVec2,
    pub(crate) bind_group: wgpu::BindGroup,
    pub(crate) z_index: i32,
}

impl SurfaceCanvas {
    
    pub(crate) fn new(
        context: &WGPUContext, 
        position: IVec2,
        blit_bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler,
        canvas_handle: CanvasHandle, 
        canvas: &Canvas,
        z_index: i32,
    ) -> Self {
        Self {
            canvas: canvas_handle,
            position,
            bind_group: create_blit_bind_group(context, blit_bind_group_layout, 
                &canvas.color_view, sampler, Some("canvas_blit_bind_group")),
            z_index,
        }
    }

    pub(crate) fn recreate(
        &mut self, 
        context: &WGPUContext,
        blit_bind_group_layout: &wgpu::BindGroupLayout,
        sampler: &wgpu::Sampler, 
        canvas: &Canvas,
    ) {
        self.bind_group = create_blit_bind_group(context, blit_bind_group_layout,
            &canvas.color_view, sampler, Some("canvas_blit_bind_group"))
    }
}