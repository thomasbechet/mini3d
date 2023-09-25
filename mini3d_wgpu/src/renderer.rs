use std::collections::HashMap;

use mini3d::feature::renderer::{mesh, texture};
use mini3d::glam::{IVec2, Mat4, UVec2, Vec3, Vec4};
use mini3d::math::rect::IRect;
use mini3d::renderer::color::{srgb_to_linear, Color};
use mini3d::renderer::event::RendererEvent;
use mini3d::renderer::graphics::TextureWrapMode;
use mini3d::renderer::provider::{
    MaterialHandle, MeshHandle, ProviderMaterialDescriptor, RendererProvider,
    RendererProviderError, SceneCameraHandle, SceneCanvasHandle, SceneModelHandle, TextureHandle,
    ViewportHandle,
};
use mini3d::renderer::{RendererStatistics, SCREEN_RESOLUTION};
use mini3d::utils::uid::{SequentialGenerator, UID};

use crate::blit_bind_group::{create_blit_bind_group, create_blit_bind_group_layout};
use crate::blit_pipeline::{
    create_blit_pipeline, create_blit_pipeline_layout, create_blit_shader_module,
};
use crate::camera::Camera;
use crate::context::WGPUContext;
use crate::error::WGPURendererError;
use crate::flat_pipeline::create_flat_pipeline;
use crate::graphics_canvas::GraphicsCanvas;
use crate::graphics_renderer::GraphicsRenderer;
use crate::material_bind_group::{
    create_flat_material_bind_group, create_flat_material_bind_group_layout,
};
use crate::mesh_pass::{create_mesh_pass_bind_group_layout, MeshPass};
use crate::model_buffer::{ModelBuffer, ModelIndex};
use crate::texture::Texture;
use crate::vertex_allocator::{VertexAllocator, VertexBufferDescriptor};
use crate::viewport::Viewport;
use crate::viewport_renderer::ViewportRenderer;

pub const MAX_MODEL_COUNT: usize = 256;
pub const MAX_OBJECT_COUNT: usize = 512;
pub const MAX_VERTEX_COUNT: usize = 125000;

struct Mesh {
    submeshes: Vec<UID>,
}

pub(crate) struct Material {
    pub(crate) bind_group: wgpu::BindGroup,
}

/// Concrete submesh object (can be clipped)
/// Multiple object can have a single model
pub(crate) struct Object {
    pub(crate) submesh: UID,
    pub(crate) material: MaterialHandle,
    pub(crate) model_index: ModelIndex,
    pub(crate) draw_forward_pass: bool,
    pub(crate) draw_shadow_pass: bool,
}

/// API model representation
/// Model has a single transform matrix
pub(crate) struct Model {
    mesh: MeshHandle,
    model_index: ModelIndex,
    objects: Vec<Option<UID>>,
}

pub struct WGPURenderer {
    // Context
    context: WGPUContext,
    generator: SequentialGenerator,

    // Scene Render Pass
    viewport_renderer: ViewportRenderer,
    flat_pipeline: wgpu::RenderPipeline,
    flat_material_bind_group_layout: wgpu::BindGroupLayout,

    // Post Process Render Pass
    blit_canvas_bind_group_layout: wgpu::BindGroupLayout,
    blit_canvas_pipeline: wgpu::RenderPipeline,

    // Assets
    vertex_allocator: VertexAllocator,
    meshes: HashMap<MeshHandle, Mesh>,
    submeshes: HashMap<UID, VertexBufferDescriptor>,
    textures: HashMap<TextureHandle, Texture>,
    materials: HashMap<MaterialHandle, Material>,

    // Scene resources
    cameras: HashMap<SceneCameraHandle, Camera>,
    models: HashMap<SceneModelHandle, Model>,
    model_buffer: ModelBuffer,
    objects: HashMap<UID, Object>,

    // Mesh passes
    mesh_pass_bind_group_layout: wgpu::BindGroupLayout,
    forward_mesh_pass: MeshPass,

    // Viewports
    viewports: HashMap<ViewportHandle, Viewport>,

    // Canvas resources
    nearest_sampler: wgpu::Sampler,
    graphics_renderer: GraphicsRenderer,
    canvases: HashMap<UID, GraphicsCanvas>,
    screen_canvas: UID,
    current_canvas: Option<UID>,
    screen_canvas_blit_bind_group: wgpu::BindGroup,

    // Statistics
    statistics: RendererStatistics,
}

impl WGPURenderer {
    pub fn new<
        W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle,
    >(
        window: &W,
    ) -> Self {
        //////// Context ////////
        let context = WGPUContext::new(&window);
        let mut generator = SequentialGenerator::default();

        //////// Common Resources ////////

        let nearest_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let linear_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("linear_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        //////// Scene Render Pass ////////

        let mesh_pass_bind_group_layout = create_mesh_pass_bind_group_layout(&context);
        let model_buffer = ModelBuffer::new(&context, MAX_MODEL_COUNT);
        let flat_material_bind_group_layout = create_flat_material_bind_group_layout(&context);
        let viewport_renderer = ViewportRenderer::new(&context, &model_buffer, &nearest_sampler);
        let flat_pipeline = create_flat_pipeline(
            &context,
            &viewport_renderer.viewport_bind_group_layout,
            &mesh_pass_bind_group_layout,
            &flat_material_bind_group_layout,
        );
        let vertex_allocator = VertexAllocator::new(&context, MAX_VERTEX_COUNT);

        //////// Blit Canvas Render Pass ////////

        let blit_shader_module = create_blit_shader_module(&context);
        let blit_canvas_bind_group_layout = create_blit_bind_group_layout(&context);
        let blit_canvas_pipeline_layout =
            create_blit_pipeline_layout(&context, &blit_canvas_bind_group_layout);
        let blit_canvas_pipeline = create_blit_pipeline(
            &context,
            &blit_canvas_pipeline_layout,
            &blit_shader_module,
            context.config.format,
            wgpu::BlendState::ALPHA_BLENDING,
            "blit_canvas_pipeline",
        );

        /////// Mesh Pass ///////
        let forward_mesh_pass = MeshPass::new(
            &context,
            &mesh_pass_bind_group_layout,
            MAX_OBJECT_COUNT,
            MAX_OBJECT_COUNT,
        );

        //////// Canvas ////////
        let graphics_renderer = GraphicsRenderer::new(&context);
        let canvas = GraphicsCanvas::new(&context, &graphics_renderer, SCREEN_RESOLUTION);
        let screen_canvas_blit_bind_group = create_blit_bind_group(
            &context,
            &blit_canvas_bind_group_layout,
            &canvas.color_view,
            &linear_sampler,
            Some("screen_canvas_blit_bind_group"),
        );
        let screen_canvas = generator.next();
        let canvases = HashMap::from([(screen_canvas, canvas)]);

        Self {
            context,
            generator: Default::default(),

            viewport_renderer,
            flat_pipeline,
            flat_material_bind_group_layout,

            blit_canvas_bind_group_layout,
            blit_canvas_pipeline,

            vertex_allocator,
            meshes: Default::default(),
            submeshes: Default::default(),
            textures: Default::default(),
            materials: Default::default(),

            cameras: Default::default(),
            models: Default::default(),
            model_buffer,
            objects: Default::default(),

            mesh_pass_bind_group_layout,
            forward_mesh_pass,

            nearest_sampler,
            graphics_renderer,
            canvases,
            current_canvas: None,
            screen_canvas,
            screen_canvas_blit_bind_group,

            viewports: Default::default(),

            statistics: RendererStatistics::default(),
        }
    }

    pub fn context(&self) -> &WGPUContext {
        &self.context
    }

    pub fn recreate(&mut self) {
        self.context.recreate();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.context.resize(width, height);
        }
    }

    pub fn render<
        F: FnOnce(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView),
    >(
        &mut self,
        engine_viewport: Vec4,
        egui_pass: F,
    ) -> Result<(), WGPURendererError> {
        // Acquire next surface texture
        let output = self
            .context
            .surface
            .get_current_texture()
            .map_err(|_| WGPURendererError::SurfaceAcquisition)?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Create frame encoder
        let mut encoder =
            self.context
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("encoder"),
                });

        // Update models
        self.model_buffer.write_buffer(&self.context);

        // Update mesh passes
        {
            if self.forward_mesh_pass.out_of_date() {
                println!("rebuild forward mesh pass");
                self.forward_mesh_pass.build(&self.objects, &self.submeshes);
                self.forward_mesh_pass.write_buffers(&self.context);
            }
        }

        // Render viewports
        self.viewport_renderer.render(
            &self.context,
            &self.viewports,
            &self.cameras,
            &self.materials,
            &self.submeshes,
            &self.vertex_allocator,
            &self.flat_pipeline,
            &self.forward_mesh_pass,
            &mut self.statistics,
            &mut encoder,
        );

        // Render canvases
        for canvas in self.canvases.values_mut() {
            self.graphics_renderer.render_canvas(
                &self.context,
                &self.textures,
                &self.viewports,
                canvas,
                &mut encoder,
            );
        }

        // Blit screen canvas
        {
            let mut clear_color = [25.0 / 255.0, 27.0 / 255.0, 43.0 / 255.0];
            if self.context.config.format.describe().srgb {
                clear_color = srgb_to_linear(clear_color);
            }
            let mut blit_canvas_render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("blit_canvas_render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: clear_color[0] as f64,
                                g: clear_color[1] as f64,
                                b: clear_color[2] as f64,
                                a: 1.0,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });

            // Setup viewport
            blit_canvas_render_pass.set_viewport(
                engine_viewport.x,
                engine_viewport.y,
                engine_viewport.z,
                engine_viewport.w,
                0.0,
                1.0,
            );

            // Blit canvas
            blit_canvas_render_pass.set_pipeline(&self.blit_canvas_pipeline);
            blit_canvas_render_pass.set_bind_group(0, &self.screen_canvas_blit_bind_group, &[]);
            blit_canvas_render_pass.draw(0..3, 0..1);
        }

        // egui pass
        egui_pass(
            &self.context.device,
            &self.context.queue,
            &mut encoder,
            &output_view,
        );

        // Submit queue and present
        self.context.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    fn add_object(
        &mut self,
        submesh: UID,
        material: MaterialHandle,
        model_index: usize,
    ) -> Result<UID, WGPURendererError> {
        let uid = self.generator.next();
        self.objects.insert(
            uid,
            Object {
                submesh,
                material,
                model_index,
                draw_forward_pass: true,
                draw_shadow_pass: false,
            },
        );
        self.forward_mesh_pass.add(uid)?;
        Ok(uid)
    }
    fn remove_object(&mut self, uid: UID) {
        let object = self.objects.remove(&uid).unwrap();
        if object.draw_forward_pass {
            self.forward_mesh_pass.remove(uid);
        }
        if object.draw_shadow_pass {
            // TODO: remove from pass
        }
    }
}

impl RendererProvider for WGPURenderer {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    /// Global API

    fn next_event(&mut self) -> Option<RendererEvent> {
        None
    }
    fn reset(&mut self) -> Result<(), RendererProviderError> {
        // Remove all models (and objects)
        let handles = self.models.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            self.scene_model_remove(handle)?;
        }
        self.cameras.clear();
        self.graphics_renderer.reset();
        self.viewports.clear();
        // Remove all canvases except the screen canvas
        let handles = self.canvases.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            if handle != self.screen_canvas {
                self.canvases.remove(&handle);
            }
        }
        // Remove resources
        self.meshes.clear();
        self.vertex_allocator.clear();
        self.submeshes.clear();
        self.textures.clear();
        self.materials.clear();
        Ok(())
    }

    /// Assets API

    fn mesh_add(&mut self, mesh: &mesh::Mesh) -> Result<MeshHandle, RendererProviderError> {
        let mut submeshes: Vec<UID> = Default::default();
        for submesh in mesh.submeshes.iter() {
            let descriptor = self
                .vertex_allocator
                .add(&self.context, &submesh.vertices)
                .map_err(|_| RendererProviderError::MaxResourcesReached)?;
            let submesh_uid = self.generator.next();
            self.submeshes.insert(submesh_uid, descriptor);
            submeshes.push(submesh_uid);
        }
        let handle: MeshHandle = self.generator.next().into();
        self.meshes.insert(handle, Mesh { submeshes });
        Ok(handle)
    }
    fn mesh_remove(&mut self, handle: MeshHandle) -> Result<(), RendererProviderError> {
        todo!()
    }

    fn texture_add(
        &mut self,
        texture: &texture::Texture,
    ) -> Result<TextureHandle, RendererProviderError> {
        let handle: TextureHandle = self.generator.next().into();
        self.textures.insert(
            handle,
            Texture::from_asset(
                &self.context,
                texture,
                wgpu::TextureUsages::TEXTURE_BINDING,
                None,
            ),
        );
        Ok(handle)
    }
    fn texture_remove(&mut self, handle: TextureHandle) -> Result<(), RendererProviderError> {
        todo!()
    }

    fn material_add(
        &mut self,
        desc: ProviderMaterialDescriptor,
    ) -> Result<MaterialHandle, RendererProviderError> {
        let diffuse = self.textures.get(&desc.diffuse).expect("Texture not found");
        let handle: MaterialHandle = self.generator.next().into();
        self.materials.insert(
            handle,
            Material {
                bind_group: create_flat_material_bind_group(
                    &self.context,
                    &self.flat_material_bind_group_layout,
                    &diffuse.view,
                    desc.name,
                ),
            },
        );
        Ok(handle)
    }
    fn material_remove(&mut self, handle: MaterialHandle) -> Result<(), RendererProviderError> {
        todo!()
    }

    /// Canvas API

    fn screen_canvas_begin(&mut self, clear_color: Color) -> Result<(), RendererProviderError> {
        self.current_canvas = Some(self.screen_canvas);
        let canvas = self.canvases.get_mut(&self.screen_canvas).unwrap();
        canvas.render_pass.begin(clear_color);
        Ok(())
    }
    fn scene_canvas_begin(
        &mut self,
        canvas: SceneCanvasHandle,
        clear_color: Color,
    ) -> Result<(), RendererProviderError> {
        self.current_canvas = Some(self.screen_canvas);
        let canvas = self.canvases.get_mut(&canvas.into()).unwrap();
        canvas.render_pass.begin(clear_color);
        Ok(())
    }
    fn canvas_end(&mut self) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .unwrap();
        canvas.render_pass.end();
        self.current_canvas = None;
        Ok(())
    }
    fn canvas_blit_texture(
        &mut self,
        texture: TextureHandle,
        extent: IRect,
        tex_extent: IRect,
        filtering: Color,
        wrap_mode: TextureWrapMode,
        alpha_threshold: u8,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas.render_pass.blit_rect(
            texture,
            extent,
            tex_extent,
            filtering,
            wrap_mode,
            alpha_threshold,
        );
        Ok(())
    }
    fn canvas_blit_viewport(
        &mut self,
        handle: ViewportHandle,
        position: IVec2,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        let viewport = self
            .viewports
            .get(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas
            .render_pass
            .blit_viewport(handle, viewport.extent, position);
        Ok(())
    }
    fn canvas_fill_rect(
        &mut self,
        extent: IRect,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas.render_pass.fill_rect(extent, color);
        Ok(())
    }
    fn canvas_draw_rect(
        &mut self,
        extent: IRect,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas.render_pass.draw_rect(extent, color);
        Ok(())
    }
    fn canvas_draw_line(
        &mut self,
        x0: IVec2,
        x1: IVec2,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas.render_pass.draw_line(x0, x1, color);
        Ok(())
    }
    fn canvas_draw_vline(
        &mut self,
        x: i32,
        y0: i32,
        y1: i32,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas.render_pass.draw_vline(x, y0, y1, color);
        Ok(())
    }
    fn canvas_draw_hline(
        &mut self,
        y: i32,
        x0: i32,
        x1: i32,
        color: Color,
    ) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        canvas.render_pass.draw_hline(y, x0, x1, color);
        Ok(())
    }
    fn canvas_scissor(&mut self, extent: Option<IRect>) -> Result<(), RendererProviderError> {
        let canvas = self
            .canvases
            .get_mut(&self.current_canvas.unwrap())
            .ok_or(RendererProviderError::ResourceNotFound)?;
        if let Some(extent) = extent {
            canvas.render_pass.scissor(extent);
        } else {
            canvas
                .render_pass
                .scissor(IRect::new(0, 0, canvas.extent.width, canvas.extent.height));
        }
        Ok(())
    }

    /// Viewport API

    fn viewport_add(&mut self, resolution: UVec2) -> Result<ViewportHandle, RendererProviderError> {
        let handle: ViewportHandle = self.generator.next().into();
        self.viewports
            .insert(handle, Viewport::new(&self.context, resolution));
        Ok(handle)
    }
    fn viewport_remove(&mut self, handle: ViewportHandle) -> Result<(), RendererProviderError> {
        self.viewports
            .remove(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        Ok(())
    }
    fn viewport_set_camera(
        &mut self,
        handle: ViewportHandle,
        camera: Option<SceneCameraHandle>,
    ) -> Result<(), RendererProviderError> {
        let viewport = self
            .viewports
            .get_mut(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        viewport.camera = camera;
        Ok(())
    }
    fn viewport_set_resolution(
        &mut self,
        handle: ViewportHandle,
        resolution: UVec2,
    ) -> Result<(), RendererProviderError> {
        let viewport = self
            .viewports
            .get_mut(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        viewport.resize(&self.context, resolution);
        Ok(())
    }

    /// Scene API

    fn scene_add(
        &mut self,
    ) -> Result<mini3d::renderer::provider::SceneHandle, RendererProviderError> {
        todo!()
    }
    fn scene_remove(
        &mut self,
        handle: mini3d::renderer::provider::SceneHandle,
    ) -> Result<(), RendererProviderError> {
        todo!()
    }
    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle, RendererProviderError> {
        let handle: SceneCameraHandle = self.generator.next().into();
        self.cameras.insert(handle, Camera::default());
        Ok(handle)
    }
    fn scene_camera_remove(
        &mut self,
        handle: SceneCameraHandle,
    ) -> Result<(), RendererProviderError> {
        self.cameras
            .remove(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        Ok(())
    }
    fn scene_camera_update(
        &mut self,
        handle: SceneCameraHandle,
        eye: Vec3,
        forward: Vec3,
        up: Vec3,
        fov: f32,
    ) -> Result<(), RendererProviderError> {
        let camera = self
            .cameras
            .get_mut(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        camera.update(eye, forward, up, fov);
        Ok(())
    }

    fn scene_model_add(
        &mut self,
        mesh_handle: MeshHandle,
    ) -> Result<SceneModelHandle, RendererProviderError> {
        // Reserve the model index
        let model_index = self.model_buffer.add();
        // Generate the handle
        let handle: SceneModelHandle = self.generator.next().into();
        // Insert model (empty by default)
        let mesh = self
            .meshes
            .get(&mesh_handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        self.models.insert(
            handle,
            Model {
                mesh: mesh_handle,
                model_index,
                objects: vec![None; mesh.submeshes.len()],
            },
        );
        // Return handle
        Ok(handle)
    }
    fn scene_model_remove(
        &mut self,
        handle: SceneModelHandle,
    ) -> Result<(), RendererProviderError> {
        let model = self
            .models
            .remove(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        for object in model.objects.iter().flatten() {
            self.remove_object(*object);
        }
        self.model_buffer.remove(model.model_index);
        Ok(())
    }
    fn scene_model_set_material(
        &mut self,
        handle: SceneModelHandle,
        index: usize,
        material: MaterialHandle,
    ) -> Result<(), RendererProviderError> {
        // Check input
        let model = self
            .models
            .get(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        let mesh = self
            .meshes
            .get(&model.mesh)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        if index >= model.objects.len() {
            return Err(RendererProviderError::InvalidMatrialIndex);
        }
        // Get model info
        let submesh = *mesh.submeshes.get(index).unwrap();
        let model_index = model.model_index;
        let previous_object = *model.objects.get(index).unwrap();
        // Remove previous object
        if let Some(previous_uid) = previous_object {
            self.remove_object(previous_uid);
        }
        // Add object
        let object_uid = self
            .add_object(submesh, material, model_index)
            .map_err(|_| RendererProviderError::MaxResourcesReached)?;
        *self
            .models
            .get_mut(&handle)
            .unwrap()
            .objects
            .get_mut(index)
            .unwrap() = Some(object_uid);
        Ok(())
    }
    fn scene_model_transfer_matrix(
        &mut self,
        handle: SceneModelHandle,
        mat: Mat4,
    ) -> Result<(), RendererProviderError> {
        let model = self
            .models
            .get(&handle)
            .ok_or(RendererProviderError::ResourceNotFound)?;
        self.model_buffer.set_transform(model.model_index, &mat);
        Ok(())
    }
    fn scene_canvas_add(
        &mut self,
        resolution: UVec2,
    ) -> Result<SceneCanvasHandle, RendererProviderError> {
        todo!()
    }
    fn scene_canvas_remove(
        &mut self,
        handle: SceneCanvasHandle,
    ) -> Result<(), RendererProviderError> {
        todo!()
    }
    fn scene_canvas_transfer_matrix(
        &mut self,
        handle: SceneCanvasHandle,
        mat: Mat4,
    ) -> Result<(), RendererProviderError> {
        todo!()
    }
}
