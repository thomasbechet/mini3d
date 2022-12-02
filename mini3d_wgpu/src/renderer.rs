use std::collections::HashMap;

use mini3d::anyhow::{Result, Context, anyhow};
use mini3d::engine::Engine;
use mini3d::feature::asset::{mesh, texture, font};
use mini3d::glam::{Vec4, Mat4, Vec3};
use mini3d::renderer::RendererStatistics;
use mini3d::renderer::backend::{RendererBackend, BackendMaterialDescriptor, MeshHandle, MaterialHandle, TextureHandle, ModelHandle, FontHandle, CameraHandle};
use mini3d::renderer::color::srgb_to_linear;
use mini3d::renderer::command_buffer::CommandBuffer;
use mini3d::uid::{UID, SequentialGenerator};

use crate::blit_bind_group::{create_blit_bind_group_layout, create_blit_bind_group};
use crate::blit_pipeline::{create_blit_pipeline_layout, create_blit_pipeline, create_blit_shader_module};
use crate::camera::Camera;
use crate::global_bind_group::{create_global_bind_group, create_global_bind_group_layout};
use crate::global_buffer::GlobalBuffer;
use crate::mesh_pass::{MeshPass, create_mesh_pass_bind_group_layout, GPUDrawIndirect};
use crate::model_buffer::{ModelBuffer, ModelIndex};
use crate::context::WGPUContext;
use crate::material_bind_group::{create_flat_material_bind_group_layout, create_flat_material_bind_group};
use crate::render_target::RenderTarget;
use crate::flat_pipeline::create_flat_pipeline;
use crate::surface_buffer::{SurfaceBuffer, Color};
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
    global_uniform_buffer: GlobalBuffer,
    global_bind_group: wgpu::BindGroup,
    flat_pipeline: wgpu::RenderPipeline,
    flat_material_bind_group_layout: wgpu::BindGroupLayout,
    
    // Surface Render Pass
    surface_buffer: SurfaceBuffer,
    surface_bind_group: wgpu::BindGroup,
    surface_pipeline: wgpu::RenderPipeline,
    
    // Post Process Render Pass
    render_target: RenderTarget,
    post_process_bind_group: wgpu::BindGroup,
    post_process_pipeline: wgpu::RenderPipeline,
    
    // Immediate commands
    command_buffers: Vec<CommandBuffer>,
    
    // Assets
    vertex_allocator: VertexAllocator,
    meshes: HashMap<MeshHandle, Mesh>,
    submeshes: HashMap<UID, VertexBufferDescriptor>,
    textures: HashMap<TextureHandle, Texture>,
    materials: HashMap<MaterialHandle, Material>,
    
    // Scene resources
    models: HashMap<ModelHandle, Model>,
    model_buffer: ModelBuffer,
    objects: HashMap<UID, Object>,
    camera: Camera,

    // Mesh passes
    mesh_pass_bind_group_layout: wgpu::BindGroupLayout,
    forward_mesh_pass: MeshPass,

    // Statistics
    statistics: RendererStatistics,
}

impl WGPURenderer {

    pub fn new<W: raw_window_handle::HasRawWindowHandle>(window: &W) -> Self {

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
        let vertex_buffer = VertexAllocator::new(&context, MAX_VERTEX_COUNT);

        //////// Surface Render Pass ////////
         
        let surface_buffer = SurfaceBuffer::new(&context);
        let surface_bind_group_layout = create_blit_bind_group_layout(&context);
        let surface_pipeline_layout = create_blit_pipeline_layout(&context, &surface_bind_group_layout);
        let blit_shader_module = create_blit_shader_module(&context);
        let surface_bind_group = create_blit_bind_group(
            &context, 
            &surface_bind_group_layout, 
            &surface_buffer.texture_view, 
            &nearest_sampler, 
            "surface_blit_bind_group"
        );
        let surface_pipeline = create_blit_pipeline(
            &context, 
            &surface_pipeline_layout, 
            &blit_shader_module, 
            context.config.format, 
            wgpu::BlendState::ALPHA_BLENDING,
            "surface_blit_pipeline"
        );

        //////// Post Process Render Pass ////////
        
        let render_target = RenderTarget::new(&context);
        let pp_bind_group_layout = create_blit_bind_group_layout(&context);
        let pp_pipeline_layout = create_blit_pipeline_layout(&context, &pp_bind_group_layout);
        let post_process_bind_group = create_blit_bind_group(
            &context, 
            &pp_bind_group_layout, 
            &render_target.render_view, 
            &nearest_sampler, 
            "post_process_bind_group"
        );
        let post_process_pipeline = create_blit_pipeline(
            &context, 
            &pp_pipeline_layout, 
            &blit_shader_module, 
            context.config.format, 
            wgpu::BlendState::REPLACE, 
            "post_process_pipeline"
        );

        /////// Mesh Pass ///////
        let forward_mesh_pass = MeshPass::new(
            &context, &mesh_pass_bind_group_layout, 
            MAX_OBJECT_COUNT, 
            MAX_OBJECT_COUNT,
        );

        Self {
            context,
            generator: Default::default(),

            global_uniform_buffer: global_buffer,
            global_bind_group,
            flat_pipeline,
            flat_material_bind_group_layout,
            
            surface_buffer,
            surface_bind_group,
            surface_pipeline,
            
            render_target,
            post_process_bind_group,
            post_process_pipeline,
            
            command_buffers: Default::default(),
            
            vertex_allocator: vertex_buffer,
            meshes: Default::default(),
            submeshes: Default::default(),
            textures: Default::default(),
            materials: Default::default(),
            
            models: Default::default(),
            model_buffer,
            objects: Default::default(),
            camera: Camera::default(),
        
            mesh_pass_bind_group_layout,
            forward_mesh_pass,

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
        
        // Process immediate commands
        self.surface_buffer.clear(Color::from_color_alpha(Color::BLACK, 0));
        for command in &self.command_buffers {
            self.surface_buffer.draw_command_buffer(engine, command);
        }

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

        // Camera Matrix
        let projection = self.camera.projection();
        let view = self.camera.view();
        self.global_uniform_buffer.set_world_to_clip(&(projection * view));
        self.global_uniform_buffer.write_buffer(&self.context);

        // Update Surface Buffer
        self.surface_buffer.write_texture(&self.context);

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

        // Forward Render Pass
        {
            let mut forward_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("forward_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render_target.depth_view,
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

        // Post Process Render Pass
        {
            let mut clear_color = [25.0 / 255.0, 27.0 / 255.0, 43.0 / 255.0];
            if self.context.config.format.describe().srgb {
                clear_color = srgb_to_linear(clear_color);
            }
            let mut post_process_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post_process_render_pass"),
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

            // Compute viewport        
            post_process_render_pass.set_viewport(
                engine_viewport.x, 
                engine_viewport.y, 
                engine_viewport.z, 
                engine_viewport.w, 
                0.0, 1.0
            );
        
            post_process_render_pass.set_pipeline(&self.post_process_pipeline);
            post_process_render_pass.set_bind_group(0, &self.post_process_bind_group, &[]);
            post_process_render_pass.draw(0..3, 0..1);
        }

        {
            let mut surface_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("surface_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            surface_render_pass.set_viewport(
                engine_viewport.x, 
                engine_viewport.y, 
                engine_viewport.z,
                engine_viewport.w,
                0.0, 1.0
            );
            self.statistics.viewport = (RenderTarget::extent().width, RenderTarget::extent().height);

            surface_render_pass.set_pipeline(&self.surface_pipeline);
            surface_render_pass.set_bind_group(0, &self.surface_bind_group, &[]);
            surface_render_pass.draw(0..3, 0..1);        
        }

        // egui pass
        {
            egui_pass(&self.context.device, &self.context.queue, &mut encoder, &output_view);
        }

        // Submit queue and present
        self.context.queue.submit(Some(encoder.finish()));
        output.present();

        // Clear resources
        self.command_buffers.clear();

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

    fn reset(&mut self) -> Result<()> {
        // Remove all models (and objects)
        let handles = self.models.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            self.model_remove(handle)?;
        }
        // Remove all meshes
        self.meshes.clear();
        self.vertex_allocator.clear();
        self.submeshes.clear();
        // Remove all textures
        self.textures.clear();
        // Remove all materials
        self.materials.clear();
        Ok(())
    }

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

    fn font_add(&mut self, _font: &font::Font) -> Result<FontHandle> { 
        Ok(0.into()) 
    }

    fn camera_add(&mut self) -> Result<CameraHandle> {
        Ok(0.into())
    }
    fn camera_remove(&mut self, _handle: CameraHandle) -> Result<()> {
        Ok(())
    }
    fn camera_update(&mut self, _handle: CameraHandle, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> {
        self.camera.update(eye, forward, up, fov);
        Ok(())
    }

    fn model_add(&mut self, mesh_handle: MeshHandle) -> Result<ModelHandle> {
        // Reserve the model index
        let model_index = self.model_buffer.add();
        // Generate the handle
        let handle: ModelHandle = self.generator.next().into();
        // Insert model (empty by default)
        let mesh = self.meshes.get(&mesh_handle).with_context(|| "Mesh not found")?;
        self.models.insert(handle, Model { mesh: mesh_handle, model_index, objects: vec![None; mesh.submeshes.len()] });
        // Return handle
        Ok(handle)
    }
    fn model_remove(&mut self, handle: ModelHandle) -> Result<()> {
        let model = self.models.remove(&handle).with_context(|| "Model not found")?;
        for object in model.objects.iter().flatten() {
            self.remove_object(*object)?;
        }
        self.model_buffer.remove(model.model_index);
        Ok(())
    }
    fn model_set_material(&mut self, handle: ModelHandle, index: usize, material: MaterialHandle) -> Result<()> {
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
    fn model_transfer_matrix(&mut self, handle: ModelHandle, mat: Mat4) -> Result<()> {
        let model = self.models.get(&handle).with_context(|| "Model not found")?;
        self.model_buffer.set_transform(model.model_index, &mat);
        Ok(())
    }

    fn submit_command_buffer(&mut self, command: CommandBuffer) -> Result<()> {
        self.command_buffers.push(command);
        Ok(())
    }

    fn statistics(&self) -> Result<RendererStatistics> { 
        Ok(self.statistics)
    }
}