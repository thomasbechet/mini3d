use std::{cell::RefCell, rc::Rc};

use mini3d_core::renderer::provider::RendererProvider;
use mini3d_wgpu::WGPURenderer;

pub(crate) struct WinitRendererProvider(Rc<RefCell<WGPURenderer>>);

impl WinitRendererProvider {
    pub(crate) fn new(renderer: Rc<RefCell<WGPURenderer>>) -> Self {
        Self(renderer)
    }
}

impl RendererProvider for WinitRendererProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn next_event(&mut self) -> Option<mini3d_core::renderer::event::RendererEvent> {
        self.0.borrow_mut().next_event()
    }

    fn reset(&mut self) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().reset()
    }

    fn mesh_add(
        &mut self,
        mesh: &mini3d_core::feature::renderer::mesh::Mesh,
    ) -> Result<
        mini3d_core::renderer::provider::MeshProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().mesh_add(mesh)
    }

    fn mesh_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::MeshProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().mesh_remove(handle)
    }

    fn texture_add(
        &mut self,
        texture: &mini3d_core::feature::renderer::texture::Texture,
    ) -> Result<
        mini3d_core::renderer::provider::TextureProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().texture_add(texture)
    }

    fn texture_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::TextureProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().texture_remove(handle)
    }

    fn material_add(
        &mut self,
        desc: mini3d_core::renderer::provider::ProviderMaterialDescriptor,
    ) -> Result<
        mini3d_core::renderer::provider::MaterialProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().material_add(desc)
    }

    fn material_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::MaterialProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().material_remove(handle)
    }

    fn screen_canvas_begin(
        &mut self,
        clear_color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().screen_canvas_begin(clear_color)
    }

    fn scene_canvas_begin(
        &mut self,
        canvas: mini3d_core::renderer::provider::SceneCanvasProviderHandle,
        clear_color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_canvas_begin(canvas, clear_color)
    }

    fn canvas_end(&mut self) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_end()
    }

    fn canvas_blit_texture(
        &mut self,
        texture: mini3d_core::renderer::provider::TextureProviderHandle,
        extent: mini3d_core::math::rect::IRect,
        texture_extent: mini3d_core::math::rect::IRect,
        filtering: mini3d_core::renderer::color::Color,
        wrap_mode: mini3d_core::renderer::graphics::TextureWrapMode,
        alpha_threshold: u8,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_blit_texture(
            texture,
            extent,
            texture_extent,
            filtering,
            wrap_mode,
            alpha_threshold,
        )
    }

    fn canvas_blit_viewport(
        &mut self,
        viewport: mini3d_core::renderer::provider::ViewportProviderHandle,
        position: mini3d_core::glam::IVec2,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_blit_viewport(viewport, position)
    }

    fn canvas_fill_rect(
        &mut self,
        extent: mini3d_core::math::rect::IRect,
        color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_fill_rect(extent, color)
    }

    fn canvas_draw_rect(
        &mut self,
        extent: mini3d_core::math::rect::IRect,
        color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_rect(extent, color)
    }

    fn canvas_draw_line(
        &mut self,
        x0: mini3d_core::glam::IVec2,
        x1: mini3d_core::glam::IVec2,
        color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_line(x0, x1, color)
    }

    fn canvas_draw_vline(
        &mut self,
        x: i32,
        y0: i32,
        y1: i32,
        color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_vline(x, y0, y1, color)
    }

    fn canvas_draw_hline(
        &mut self,
        y: i32,
        x0: i32,
        x1: i32,
        color: mini3d_core::renderer::color::Color,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_hline(y, x0, x1, color)
    }

    fn canvas_scissor(
        &mut self,
        extent: Option<mini3d_core::math::rect::IRect>,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_scissor(extent)
    }

    fn viewport_add(
        &mut self,
        resolution: mini3d_core::glam::UVec2,
    ) -> Result<
        mini3d_core::renderer::provider::ViewportProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().viewport_add(resolution)
    }

    fn viewport_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::ViewportProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().viewport_remove(handle)
    }

    fn viewport_set_camera(
        &mut self,
        handle: mini3d_core::renderer::provider::ViewportProviderHandle,
        camera: Option<mini3d_core::renderer::provider::SceneCameraProviderHandle>,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().viewport_set_camera(handle, camera)
    }

    fn viewport_set_resolution(
        &mut self,
        handle: mini3d_core::renderer::provider::ViewportProviderHandle,
        resolution: mini3d_core::glam::UVec2,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .viewport_set_resolution(handle, resolution)
    }

    fn scene_add(
        &mut self,
    ) -> Result<
        mini3d_core::renderer::provider::SceneProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_add()
    }

    fn scene_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_remove(handle)
    }

    fn scene_camera_add(
        &mut self,
    ) -> Result<
        mini3d_core::renderer::provider::SceneCameraProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_camera_add()
    }

    fn scene_camera_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneCameraProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_camera_remove(handle)
    }

    fn scene_camera_update(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneCameraProviderHandle,
        eye: mini3d_core::glam::Vec3,
        forward: mini3d_core::glam::Vec3,
        up: mini3d_core::glam::Vec3,
        fov: f32,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .scene_camera_update(handle, eye, forward, up, fov)
    }

    fn scene_model_add(
        &mut self,
        mesh: mini3d_core::renderer::provider::MeshProviderHandle,
    ) -> Result<
        mini3d_core::renderer::provider::SceneModelProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_model_add(mesh)
    }

    fn scene_model_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneModelProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_model_remove(handle)
    }

    fn scene_model_set_material(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneModelProviderHandle,
        index: usize,
        material: mini3d_core::renderer::provider::MaterialProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .scene_model_set_material(handle, index, material)
    }

    fn scene_model_transfer_matrix(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneModelProviderHandle,
        mat: mini3d_core::glam::Mat4,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_model_transfer_matrix(handle, mat)
    }

    fn scene_canvas_add(
        &mut self,
        resolution: mini3d_core::glam::UVec2,
    ) -> Result<
        mini3d_core::renderer::provider::SceneCanvasProviderHandle,
        mini3d_core::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_canvas_add(resolution)
    }

    fn scene_canvas_remove(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneCanvasProviderHandle,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_canvas_remove(handle)
    }

    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: mini3d_core::renderer::provider::SceneCanvasProviderHandle,
        mat: mini3d_core::glam::Mat4,
    ) -> Result<(), mini3d_core::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .scene_canvas_transfer_matrix(handle, mat)
    }
}
