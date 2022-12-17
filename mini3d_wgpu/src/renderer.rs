use std::collections::HashMap;

use mini3d::anyhow::{Result, Context, anyhow};
use mini3d::engine::Engine;
use mini3d::feature::asset::{mesh, texture, font};
use mini3d::glam::{Vec4, Mat4, Vec3, UVec2, IVec2};
use mini3d::math::rect::IRect;
use mini3d::renderer::{RendererStatistics, SCREEN_WIDTH, SCREEN_HEIGHT};
use mini3d::renderer::backend::{RendererBackend, BackendMaterialDescriptor, MeshHandle, MaterialHandle, TextureHandle, SceneModelHandle, FontHandle, SceneCameraHandle, CanvasHandle, CanvasSpriteHandle, CanvasViewportHandle, SurfaceCanvasHandle};
use mini3d::renderer::color::{srgb_to_linear, Color};
use mini3d::uid::{UID, SequentialGenerator};

use crate::blit_bind_group::create_blit_bind_group_layout;
use crate::blit_pipeline::{create_blit_pipeline_layout, create_blit_pipeline, create_blit_shader_module};
use crate::camera::Camera;
use crate::canvas::{Canvas, SurfaceCanvas, CanvasViewport, CanvasSprite};
use crate::canvas_renderer::CanvasRenderer;
use crate::global_bind_group::{create_global_bind_group, create_global_bind_group_layout};
use crate::global_buffer::GlobalBuffer;
use crate::mesh_pass::{MeshPass, create_mesh_pass_bind_group_layout, GPUDrawIndirect};
use crate::model_buffer::{ModelBuffer, ModelIndex};
use crate::context::WGPUContext;
use crate::material_bind_group::{create_flat_material_bind_group_layout, create_flat_material_bind_group};
use crate::flat_pipeline::create_flat_pipeline;
use crate::texture::Texture;
use crate::vertex_allocator::{VertexAllocator, VertexBufferDescriptor};

pub const MAX_MODEL_COUNT: usize = 256;
pub const MAX_OBJECT_COUNT: usize = 512;
pub const MAX_VERTEX_COUNT: usize = 125000;

struct Mesh {
    submeshes: Vec<UID>,
}

pub(crate) struct Material {
    pub(crate) bind_group: wgpu::BindGroup,
}

struct Font {

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

fn convert_clear_color(color: Color) -> wgpu::Color {
    let clear_color: [f32; 4] = color.into();
    wgpu::Color {
        r: clear_color[0] as f64,
        g: clear_color[1] as f64,
        b: clear_color[2] as f64,
        a: clear_color[3] as f64,
    }
}

pub struct WGPURenderer {

    // Context
    context: WGPUContext,
    generator: SequentialGenerator,
    
    // Scene Render Pass
    global_uniform_buffer: GlobalBuffer,
    global_bind_group: wgpu::BindGroup,
    flat_pipeline: wgpu::RenderPipeline,
    flat_material_bind_group_layout: wgpu::BindGroupLayout,
    
    // Post Process Render Pass
    blit_canvas_bind_group_layout: wgpu::BindGroupLayout,
    blit_canvas_pipeline: wgpu::RenderPipeline,
    blit_canvas_bind_group: Option<wgpu::BindGroup>,
    
    // Assets
    vertex_allocator: VertexAllocator,
    meshes: HashMap<MeshHandle, Mesh>,
    submeshes: HashMap<UID, VertexBufferDescriptor>,
    textures: HashMap<TextureHandle, Texture>,
    materials: HashMap<MaterialHandle, Material>,
    fonts: HashMap<FontHandle, Font>,
    
    // Scene resources
    cameras: HashMap<SceneCameraHandle, Camera>,
    models: HashMap<SceneModelHandle, Model>,
    model_buffer: ModelBuffer,
    objects: HashMap<UID, Object>,

    // Mesh passes
    mesh_pass_bind_group_layout: wgpu::BindGroupLayout,
    forward_mesh_pass: MeshPass,

    // Canvas resources
    sampler: wgpu::Sampler,
    canvas_renderer: CanvasRenderer,
    canvases: HashMap<CanvasHandle, Canvas>,
    item_to_canvas: HashMap<UID, CanvasHandle>,

    // Surface resources
    surface_canvas: HashMap<SurfaceCanvasHandle, SurfaceCanvas>,

    // Statistics
    statistics: RendererStatistics,
}

impl WGPURenderer {

    pub fn new<W: raw_window_handle::HasRawWindowHandle + raw_window_handle::HasRawDisplayHandle>(window: &W) -> Self {

        //////// Context ////////
        let context = WGPUContext::new(&window);

        //////// Common Resources ////////
        
        let nearest_sampler = context.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        
        //////// Scene Render Pass ////////

        let mesh_pass_bind_group_layout = create_mesh_pass_bind_group_layout(&context);
        let model_buffer = ModelBuffer::new(&context, MAX_MODEL_COUNT);
        let global_bind_group_layout = create_global_bind_group_layout(&context);
        let flat_material_bind_group_layout = create_flat_material_bind_group_layout(&context);
        let global_buffer = GlobalBuffer::new(&context);
        let global_bind_group = create_global_bind_group(
            &context, 
            &global_bind_group_layout, 
            &global_buffer,
            &model_buffer,
            &nearest_sampler,
        );
        let flat_pipeline = create_flat_pipeline(
            &context, 
            &global_bind_group_layout,
            &mesh_pass_bind_group_layout,
            &flat_material_bind_group_layout,
        );
        let vertex_allocator = VertexAllocator::new(&context, MAX_VERTEX_COUNT);

        //////// Blit Canvas Render Pass ////////
        
        let blit_shader_module = create_blit_shader_module(&context);
        let blit_canvas_bind_group_layout = create_blit_bind_group_layout(&context);
        let blit_canvas_pipeline_layout = create_blit_pipeline_layout(&context, &blit_canvas_bind_group_layout);
        let blit_canvas_pipeline = create_blit_pipeline(
            &context, 
            &blit_canvas_pipeline_layout, 
            &blit_shader_module, 
            context.config.format, 
            wgpu::BlendState::ALPHA_BLENDING, 
            "blit_canvas_pipeline"
        );

        /////// Mesh Pass ///////
        let forward_mesh_pass = MeshPass::new(
            &context, &mesh_pass_bind_group_layout,
            MAX_OBJECT_COUNT,
            MAX_OBJECT_COUNT,
        );

        //////// Canvas ////////
        let canvas_pipeline = CanvasRenderer::new(&context);

        Self {
            context,
            generator: Default::default(),

            global_uniform_buffer: global_buffer,
            global_bind_group,
            flat_pipeline,
            flat_material_bind_group_layout,
            
            blit_canvas_bind_group_layout,
            blit_canvas_pipeline,
            blit_canvas_bind_group: None,
            
            vertex_allocator,
            meshes: Default::default(),
            submeshes: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
            fonts: Default::default(),
            
            cameras: Default::default(),
            models: Default::default(),
            model_buffer,
            objects: Default::default(),
        
            mesh_pass_bind_group_layout,
            forward_mesh_pass,

            sampler: nearest_sampler,
            canvas_renderer: canvas_pipeline,
            canvases: Default::default(),
            item_to_canvas: Default::default(),

            surface_canvas: Default::default(),

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

    pub fn render<F: FnOnce(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView)>(
        &mut self,
        engine: &Engine,
        engine_viewport: Vec4,
        egui_pass: F,
    ) -> Result<()> {

        // Acquire next surface texture
        let output = self.context.surface.get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        
        // Create frame encoder
        let mut encoder = self
            .context.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("encoder"),
            });

        // Update models
        self.model_buffer.write_buffer(&self.context);

        // Update mesh passes
        {
            if self.forward_mesh_pass.out_of_date() {
                println!("rebuild forward mesh pass");
                self.forward_mesh_pass.build(&self.objects, &self.submeshes)?;
                self.forward_mesh_pass.write_buffers(&self.context);
            }
        }

        // Render viewports
        for canvas in self.canvases.values_mut() {
            for viewport in canvas.viewports.values_mut() {

                // Retrieve the camera
                if viewport.camera.is_none() { continue; }
                let camera = self.cameras.get(&viewport.camera.unwrap()).with_context(|| "Camera not found")?;

                // Compute camera matrices
                let projection = camera.projection(viewport.aspect_ratio());
                let view = camera.view();
                self.global_uniform_buffer.set_world_to_clip(&(projection * view));
                self.global_uniform_buffer.write_buffer(&self.context);

                // Forward Render Pass
                {
                    let mut forward_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("forward_render_pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &viewport.color_view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                                store: true,
                            },
                        })],
                        depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                            view: &viewport.depth_view,
                            depth_ops: Some(wgpu::Operations {
                                load: wgpu::LoadOp::Clear(1.0),
                                store: true,
                            }),
                            stencil_ops: None,
                        }),
                    });

                    forward_render_pass.set_pipeline(&self.flat_pipeline);
                    forward_render_pass.set_bind_group(0, &self.global_bind_group, &[]);
                    forward_render_pass.set_bind_group(1, &self.forward_mesh_pass.bind_group, &[]);

                    forward_render_pass.set_vertex_buffer(0, self.vertex_allocator.position_buffer.slice(..));
                    forward_render_pass.set_vertex_buffer(1, self.vertex_allocator.normal_buffer.slice(..));
                    forward_render_pass.set_vertex_buffer(2, self.vertex_allocator.uv_buffer.slice(..));

                    // Multi draw indirect
                    {
                        let mut triangle_count = 0;
                        for batch in &self.forward_mesh_pass.multi_instanced_batches {

                            // Bind materials
                            let material = self.materials.get(&batch.material)
                                .expect("Failed to get material during forward pass");
                            forward_render_pass.set_bind_group(2, &material.bind_group, &[]);
                        
                            // Indirect draw
                            forward_render_pass.multi_draw_indirect(
                                &self.forward_mesh_pass.indirect_command_buffer, 
                                (std::mem::size_of::<GPUDrawIndirect>() * batch.first) as u64, 
                                batch.count as u32,
                            );
                            triangle_count += batch.triangle_count;
                        }
                        self.statistics.draw_count = self.forward_mesh_pass.multi_instanced_batches.len();
                        self.statistics.triangle_count = triangle_count;
                    }
                    
                    // Classic draw
                    // {
                    //     self.statistics.triangle_count = 0;
                    //     let mut previous_material: MaterialHandle = Default::default();
                    //     for batch in &self.forward_mesh_pass.instanced_batches {
                            
                    //         // Check change in material
                    //         if batch.material != previous_material {
                    //             previous_material = batch.material;
                    //             let material = self.materials.get(&batch.material)
                    //                 .expect("Failed to get material during forward pass");
                    //             forward_render_pass.set_bind_group(2, &material.bind_group, &[]);
                    //         }

                    //         // Draw instanced
                    //         let descriptor = self.submeshes.get(&batch.submesh)
                    //             .expect("Failed to get submesh descriptor");
                    //         let vertex_start = descriptor.base_index;
                    //         let vertex_stop = vertex_start + descriptor.vertex_count;
                    //         let instance_start = batch.first_instance as u32;
                    //         let instance_stop = batch.first_instance as u32 + batch.instance_count as u32;
                    //         forward_render_pass.draw(
                    //             vertex_start..vertex_stop, 
                    //             instance_start..instance_stop,
                    //         );

                    //         self.statistics.triangle_count += batch.triangle_count;
                    //     }
                    //     self.statistics.draw_count = self.forward_mesh_pass.instanced_batches.len();
                    // }
                }
            }
        }

        // Render canvas
        self.canvas_renderer.write_buffer(&self.context, &self.sampler, &self.textures, &self.canvases);
        self.canvas_renderer.render(&self.canvases, &mut encoder);

        // Show canvas on screen
        {
            let mut clear_color = [25.0 / 255.0, 27.0 / 255.0, 43.0 / 255.0];
            if self.context.config.format.describe().srgb {
                clear_color = srgb_to_linear(clear_color);
            }
            let mut blit_canvas_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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

            // Setup the scissor rect
            blit_canvas_render_pass.set_scissor_rect(
                engine_viewport.x as u32,
                engine_viewport.y as u32,
                engine_viewport.z as u32,
                engine_viewport.w as u32,
            );

            // Render each canvas
            let mut surfaces: Vec<&SurfaceCanvas> = self.surface_canvas.values().collect();
            surfaces.sort_by(|a, b| a.z_index.cmp(&b.z_index));
            for surface_canvas in surfaces {
                let canvas = self.canvases.get(&surface_canvas.canvas).expect("Failed to get canvas");

                // Compute viewport
                let x = engine_viewport.x + surface_canvas.position.x as f32;
                let y = engine_viewport.y + surface_canvas.position.y as f32;
                let w = (canvas.extent.width as f32 / SCREEN_WIDTH as f32) * engine_viewport.z;
                let h = (canvas.extent.height as f32 / SCREEN_HEIGHT as f32) * engine_viewport.w;
                blit_canvas_render_pass.set_viewport(x, y, w, h, 0.0, 1.0);
            
                blit_canvas_render_pass.set_pipeline(&self.blit_canvas_pipeline);
                blit_canvas_render_pass.set_bind_group(0, &surface_canvas.bind_group, &[]);
                blit_canvas_render_pass.draw(0..3, 0..1);
            }
        }

        // egui pass
        egui_pass(&self.context.device, &self.context.queue, &mut encoder, &output_view);

        // Submit queue and present
        self.context.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }

    fn add_object(&mut self, submesh: UID, material: MaterialHandle, model_index: usize) -> Result<UID> {
        let uid = self.generator.next();
        self.objects.insert(uid, Object { 
            submesh,
            material,
            model_index, 
            draw_forward_pass: true,
            draw_shadow_pass: false,
        });
        self.forward_mesh_pass.add(uid)?;
        Ok(uid)
    }
    fn remove_object(&mut self, uid: UID) -> Result<()> {
        let object = self.objects.remove(&uid).unwrap();
        if object.draw_forward_pass {
            self.forward_mesh_pass.remove(uid)?;
        }
        if object.draw_shadow_pass {
            // TODO: remove from pass
        }
        Ok(())
    }
}

impl RendererBackend for WGPURenderer {

    /// Global API

    fn reset(&mut self) -> Result<()> {
        // Remove all models (and objects)
        let handles = self.models.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            self.scene_model_remove(handle)?;
        }
        // Remove resources
        self.meshes.clear();
        self.vertex_allocator.clear();
        self.submeshes.clear();
        self.textures.clear();
        self.materials.clear();
        self.fonts.clear();
        Ok(())
    }
    
    /// Assets API

    fn mesh_add(&mut self, mesh: &mesh::Mesh) -> Result<MeshHandle> {
        let mut submeshes: Vec<UID> = Default::default();
        for submesh in mesh.submeshes.iter() {
            let descriptor = self.vertex_allocator.add(&self.context, &submesh.vertices)
                .with_context(|| "Failed to create submesh")?;
            let submesh_uid = self.generator.next();
            self.submeshes.insert(submesh_uid, descriptor);
            submeshes.push(submesh_uid);
        }
        let handle: MeshHandle = self.generator.next().into();
        self.meshes.insert(handle, Mesh { submeshes });
        Ok(handle)
    }
    
    fn texture_add(&mut self, texture: &texture::Texture) -> Result<TextureHandle> {
        let handle: TextureHandle = self.generator.next().into();
        self.textures.insert(handle, Texture::from_asset(&self.context, texture, 
            wgpu::TextureUsages::TEXTURE_BINDING, None));
        Ok(handle)
    }
    
    fn material_add(&mut self, desc: BackendMaterialDescriptor) -> Result<MaterialHandle> {
        let diffuse = self.textures.get(&desc.diffuse).expect("Texture not found");
        let handle: MaterialHandle = self.generator.next().into();
        self.materials.insert(handle, Material { 
            bind_group: create_flat_material_bind_group(
                &self.context, 
                &self.flat_material_bind_group_layout, 
                &diffuse.view,
                desc.name,
            )
        });
        Ok(handle)
    }
    
    fn font_add(&mut self, font: &font::Font) -> Result<FontHandle> {
        let handle: FontHandle = self.generator.next().into();
        self.fonts.insert(handle, Font {});
        Ok(handle)
    }

    /// Canvas API

    fn canvas_add(&mut self, width: u32, height: u32) -> Result<CanvasHandle> {
        let handle: CanvasHandle = self.generator.next().into();
        self.canvases.insert(handle, Canvas::new(&self.context, UVec2::new(width, height)));
        Ok(handle)
    }
    fn canvas_remove(&mut self, handle: CanvasHandle) -> Result<()> {
        self.canvases.remove(&handle);
        Ok(())
    }
    fn canvas_set_clear_color(&mut self, handle: CanvasHandle, color: Color) -> Result<()> { 
        let canvas = self.canvases.get_mut(&handle).with_context(|| "Canvas not found")?;
        canvas.clear_color = convert_clear_color(color);
        Ok(())
    }
    
    fn canvas_sprite_add(&mut self, canvas: CanvasHandle, texture: TextureHandle, position: IVec2, extent: IRect) -> Result<CanvasSpriteHandle> {
        let handle: CanvasSpriteHandle = self.generator.next().into();
        self.item_to_canvas.insert(handle.into(), canvas);
        let canvas = self.canvases.get_mut(&canvas).with_context(|| "Canvas not found")?;
        canvas.sprites.insert(handle, CanvasSprite {
            texture,
            z_index: 0,
            position,
            extent,
            color: Color::WHITE,
        });
        Ok(handle)
    }
    fn canvas_sprite_remove(&mut self, handle: CanvasSpriteHandle) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        canvas.sprites.remove(&handle).with_context(|| "Blit not found")?;
        Ok(())
    }
    fn canvas_sprite_set_position(&mut self, handle: CanvasSpriteHandle, position: IVec2) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let sprite = canvas.sprites.get_mut(&handle).with_context(|| "Blit not found")?;
        sprite.position = position;
        Ok(())
    }
    fn canvas_sprite_set_extent(&mut self, handle: CanvasSpriteHandle, extent: IRect) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let sprite = canvas.sprites.get_mut(&handle).with_context(|| "Blit not found")?;
        sprite.extent = extent;
        Ok(())
    }
    fn canvas_sprite_set_z_index(&mut self, handle: CanvasSpriteHandle, z_index: i32) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let blit = canvas.sprites.get_mut(&handle).with_context(|| "Blit not found")?;
        blit.z_index = z_index;
        Ok(())
    }
    fn canvas_sprite_set_texture(&mut self, handle: CanvasSpriteHandle, texture: TextureHandle) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let blit = canvas.sprites.get_mut(&handle).with_context(|| "Blit not found")?;
        blit.texture = texture;
        Ok(())
    }
    fn canvas_sprite_set_color(&mut self, handle: CanvasSpriteHandle, color: Color) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let blit = canvas.sprites.get_mut(&handle).with_context(|| "Blit not found")?;
        blit.color = color;
        Ok(())
    }

    fn canvas_viewport_add(&mut self, canvas: CanvasHandle, position: IVec2, resolution: UVec2) -> Result<CanvasViewportHandle> {
        let handle: CanvasViewportHandle = self.generator.next().into();
        self.item_to_canvas.insert(handle.into(), canvas);
        let canvas = self.canvases.get_mut(&canvas).with_context(|| "Canvas not found")?;
        canvas.viewports.insert(handle, CanvasViewport::new(&self.context, position, resolution));
        Ok(handle)
    }
    fn canvas_viewport_remove(&mut self, handle: CanvasViewportHandle) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        canvas.viewports.remove(&handle).with_context(|| "Viewport not found")?;
        Ok(())
    }
    fn canvas_viewport_set_z_index(&mut self, handle: CanvasViewportHandle, z_index: i32) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let viewport = canvas.viewports.get_mut(&handle).with_context(|| "Viewport not found")?;
        viewport.z_index = z_index;
        Ok(())
    }
    fn canvas_viewport_set_position(&mut self, handle: CanvasViewportHandle, position: IVec2) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let viewport = canvas.viewports.get_mut(&handle).with_context(|| "Viewport not found")?;
        viewport.position = position;
        Ok(())
    }
    fn canvas_viewport_set_camera(&mut self, handle: CanvasViewportHandle, camera: Option<SceneCameraHandle>) -> Result<()> {
        let canvas = self.item_to_canvas.get(&handle.into()).with_context(|| "Canvas not found")?;
        let canvas = self.canvases.get_mut(canvas).with_context(|| "Canvas not found")?;
        let viewport = canvas.viewports.get_mut(&handle).with_context(|| "Viewport not found")?;
        viewport.camera = camera;
        Ok(())
    }

    // TODO: complete canvas API

    /// Scene API

    fn scene_camera_add(&mut self) -> Result<SceneCameraHandle> {
        let handle: SceneCameraHandle = self.generator.next().into();
        self.cameras.insert(handle, Camera::default());
        Ok(handle)
    }
    fn scene_camera_remove(&mut self, handle: SceneCameraHandle) -> Result<()> {
        self.cameras.remove(&handle).with_context(|| "Camera not found")?;
        Ok(())
    }
    fn scene_camera_update(&mut self, handle: SceneCameraHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> {
        let camera = self.cameras.get_mut(&handle).with_context(|| "Camera not found")?;
        camera.update(eye, forward, up, fov);
        Ok(())
    }

    fn scene_model_add(&mut self, mesh_handle: MeshHandle) -> Result<SceneModelHandle> {
        // Reserve the model index
        let model_index = self.model_buffer.add();
        // Generate the handle
        let handle: SceneModelHandle = self.generator.next().into();
        // Insert model (empty by default)
        let mesh = self.meshes.get(&mesh_handle).with_context(|| "Mesh not found")?;
        self.models.insert(handle, Model { mesh: mesh_handle, model_index, objects: vec![None; mesh.submeshes.len()] });
        // Return handle
        Ok(handle)
    }
    fn scene_model_remove(&mut self, handle: SceneModelHandle) -> Result<()> {
        let model = self.models.remove(&handle).with_context(|| "Model not found")?;
        for object in model.objects.iter().flatten() {
            self.remove_object(*object)?;
        }
        self.model_buffer.remove(model.model_index);
        Ok(())
    }
    fn scene_model_set_material(&mut self, handle: SceneModelHandle, index: usize, material: MaterialHandle) -> Result<()> {
        // Check input
        let model = self.models.get(&handle).with_context(|| "Model not found")?;
        let mesh = self.meshes.get(&model.mesh).with_context(|| "Mesh not found")?;
        if index >= model.objects.len() { return Err(anyhow!("Invalid index")); }
        // Get model info
        let submesh = *mesh.submeshes.get(index).unwrap();
        let model_index = model.model_index;
        let previous_object = *model.objects.get(index).unwrap();
        // Remove previous object
        if let Some(previous_uid) = previous_object {
            self.remove_object(previous_uid)?;
        }
        // Add object
        let object_uid = self.add_object(submesh, material, model_index)?;
        *self.models.get_mut(&handle).unwrap().objects.get_mut(index).unwrap() = Some(object_uid);
        Ok(())
    }
    fn scene_model_transfer_matrix(&mut self, handle: SceneModelHandle, mat: Mat4) -> Result<()> {
        let model = self.models.get(&handle).with_context(|| "Model not found")?;
        self.model_buffer.set_transform(model.model_index, &mat);
        Ok(())
    }

    /// Surface API

    fn surface_canvas_add(&mut self, canvas_handle: CanvasHandle, position: IVec2, z_index: i32) -> Result<SurfaceCanvasHandle> {
        let handle = self.generator.next().into();
        let canvas = self.canvases.get(&canvas_handle).with_context(|| "Canvas not found")?;
        let surface_canvas = SurfaceCanvas::new(
            &self.context, 
            position, 
            &self.blit_canvas_bind_group_layout, 
            &self.sampler, 
            canvas_handle, 
            canvas,
            z_index,
        );
        self.surface_canvas.insert(handle, surface_canvas);
        Ok(handle)
    }
    fn surface_canvas_remove(&mut self, handle: SurfaceCanvasHandle) -> Result<()> {
        self.surface_canvas.remove(&handle).with_context(|| "Surface canvas not found")?;
        Ok(())
    }

    /// Statistics API

    fn statistics(&self) -> Result<RendererStatistics> { 
        Ok(self.statistics)
    }
}