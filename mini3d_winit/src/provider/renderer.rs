use std::{cell::RefCell, rc::Rc};

use mini3d::renderer::provider::RendererProvider;
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

    fn next_event(&mut self) -> Option<mini3d::renderer::event::RendererEvent> {
        self.0.borrow_mut().next_event()
    }

    fn reset(&mut self) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().reset()
    }

    fn mesh_add(
        &mut self,
        mesh: &mini3d::feature::renderer::mesh::Mesh,
    ) -> Result<
        mini3d::renderer::provider::MeshHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().mesh_add(mesh)
    }

    fn mesh_remove(
        &mut self,
        handle: mini3d::renderer::provider::MeshHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().mesh_remove(handle)
    }

    fn texture_add(
        &mut self,
        texture: &mini3d::feature::renderer::texture::Texture,
    ) -> Result<
        mini3d::renderer::provider::TextureHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().texture_add(texture)
    }

    fn texture_remove(
        &mut self,
        handle: mini3d::renderer::provider::TextureHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().texture_remove(handle)
    }

    fn material_add(
        &mut self,
        desc: mini3d::renderer::provider::ProviderMaterialDescriptor,
    ) -> Result<
        mini3d::renderer::provider::MaterialHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().material_add(desc)
    }

    fn material_remove(
        &mut self,
        handle: mini3d::renderer::provider::MaterialHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().material_remove(handle)
    }

    fn screen_canvas_begin(
        &mut self,
        clear_color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().screen_canvas_begin(clear_color)
    }

    fn scene_canvas_begin(
        &mut self,
        canvas: mini3d::renderer::provider::SceneCanvasHandle,
        clear_color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_canvas_begin(canvas, clear_color)
    }

    fn canvas_end(&mut self) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_end()
    }

    fn canvas_blit_texture(
        &mut self,
        texture: mini3d::renderer::provider::TextureHandle,
        extent: mini3d::math::rect::IRect,
        texture_extent: mini3d::math::rect::IRect,
        filtering: mini3d::renderer::color::Color,
        wrap_mode: mini3d::renderer::graphics::TextureWrapMode,
        alpha_threshold: u8,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
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
        viewport: mini3d::renderer::provider::ViewportHandle,
        position: mini3d::glam::IVec2,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_blit_viewport(viewport, position)
    }

    fn canvas_fill_rect(
        &mut self,
        extent: mini3d::math::rect::IRect,
        color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_fill_rect(extent, color)
    }

    fn canvas_draw_rect(
        &mut self,
        extent: mini3d::math::rect::IRect,
        color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_rect(extent, color)
    }

    fn canvas_draw_line(
        &mut self,
        x0: mini3d::glam::IVec2,
        x1: mini3d::glam::IVec2,
        color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_line(x0, x1, color)
    }

    fn canvas_draw_vline(
        &mut self,
        x: i32,
        y0: i32,
        y1: i32,
        color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_vline(x, y0, y1, color)
    }

    fn canvas_draw_hline(
        &mut self,
        y: i32,
        x0: i32,
        x1: i32,
        color: mini3d::renderer::color::Color,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_draw_hline(y, x0, x1, color)
    }

    fn canvas_scissor(
        &mut self,
        extent: Option<mini3d::math::rect::IRect>,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().canvas_scissor(extent)
    }

    fn viewport_add(
        &mut self,
        resolution: mini3d::glam::UVec2,
    ) -> Result<
        mini3d::renderer::provider::ViewportHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().viewport_add(resolution)
    }

    fn viewport_remove(
        &mut self,
        handle: mini3d::renderer::provider::ViewportHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().viewport_remove(handle)
    }

    fn viewport_set_camera(
        &mut self,
        handle: mini3d::renderer::provider::ViewportHandle,
        camera: Option<mini3d::renderer::provider::SceneCameraHandle>,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().viewport_set_camera(handle, camera)
    }

    fn viewport_set_resolution(
        &mut self,
        handle: mini3d::renderer::provider::ViewportHandle,
        resolution: mini3d::glam::UVec2,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .viewport_set_resolution(handle, resolution)
    }

    fn scene_add(
        &mut self,
    ) -> Result<
        mini3d::renderer::provider::SceneHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_add()
    }

    fn scene_remove(
        &mut self,
        handle: mini3d::renderer::provider::SceneHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_remove(handle)
    }

    fn scene_camera_add(
        &mut self,
    ) -> Result<
        mini3d::renderer::provider::SceneCameraHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_camera_add()
    }

    fn scene_camera_remove(
        &mut self,
        handle: mini3d::renderer::provider::SceneCameraHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_camera_remove(handle)
    }

    fn scene_camera_update(
        &mut self,
        handle: mini3d::renderer::provider::SceneCameraHandle,
        eye: mini3d::glam::Vec3,
        forward: mini3d::glam::Vec3,
        up: mini3d::glam::Vec3,
        fov: f32,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .scene_camera_update(handle, eye, forward, up, fov)
    }

    fn scene_model_add(
        &mut self,
        mesh: mini3d::renderer::provider::MeshHandle,
    ) -> Result<
        mini3d::renderer::provider::SceneModelHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_model_add(mesh)
    }

    fn scene_model_remove(
        &mut self,
        handle: mini3d::renderer::provider::SceneModelHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_model_remove(handle)
    }

    fn scene_model_set_material(
        &mut self,
        handle: mini3d::renderer::provider::SceneModelHandle,
        index: usize,
        material: mini3d::renderer::provider::MaterialHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .scene_model_set_material(handle, index, material)
    }

    fn scene_model_transfer_matrix(
        &mut self,
        handle: mini3d::renderer::provider::SceneModelHandle,
        mat: mini3d::glam::Mat4,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_model_transfer_matrix(handle, mat)
    }

    fn scene_canvas_add(
        &mut self,
        resolution: mini3d::glam::UVec2,
    ) -> Result<
        mini3d::renderer::provider::SceneCanvasHandle,
        mini3d::renderer::provider::RendererProviderError,
    > {
        self.0.borrow_mut().scene_canvas_add(resolution)
    }

    fn scene_canvas_remove(
        &mut self,
        handle: mini3d::renderer::provider::SceneCanvasHandle,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0.borrow_mut().scene_canvas_remove(handle)
    }

    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: mini3d::renderer::provider::SceneCanvasHandle,
        mat: mini3d::glam::Mat4,
    ) -> Result<(), mini3d::renderer::provider::RendererProviderError> {
        self.0
            .borrow_mut()
            .scene_canvas_transfer_matrix(handle, mat)
    }
}
