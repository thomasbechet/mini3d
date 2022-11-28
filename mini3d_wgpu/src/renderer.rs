use std::collections::HashMap;

use mini3d::anyhow::{Result, Context};
use mini3d::app::App;
use mini3d::asset::AssetManager;
use mini3d::backend::renderer::{RendererBackend, RendererStatistics, RendererModelDescriptor};
use mini3d::feature;
use mini3d::glam::{Vec4, Mat4, Vec3};
use mini3d::graphics::color::srgb_to_linear;
use mini3d::graphics::command_buffer::CommandBuffer;
use mini3d::uid::UID;
use wgpu::SurfaceError;

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
use crate::vertex_buffer::{VertexBuffer, VertexBufferDescriptor};

struct Mesh {
    submeshes: Vec<UID>,
}

pub(crate) struct Material {
    pub(crate) texture: UID,
    pub(crate) bind_group: wgpu::BindGroup,
}

/// Concrete submesh object (can be clipped)
/// Multiple object can have a single model
pub(crate) struct Object {
    pub(crate) submesh: UID,
    pub(crate) material: UID,
    pub(crate) model_index: ModelIndex,
    pub(crate) draw_forward_pass: bool,
    pub(crate) draw_shadow_pass: bool,
}

/// API model representation
/// Model has a single transform matrix
pub(crate) struct ModelInstance {
    mesh: UID,
    materials: Vec<UID>,
    model_index: ModelIndex,
    objects: Vec<UID>,
}

#[derive(Default)]
struct UIDGenerator {
    next: u64,
}

impl UIDGenerator {
    fn next(&mut self) -> UID {
        self.next += 1;
        UID::from(self.next)
    }
}

pub struct WGPURenderer {

    // Context
    context: WGPUContext,
    uid_generator: UIDGenerator,
    
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
    vertex_buffer: VertexBuffer,
    meshes: HashMap<UID, Mesh>,
    submeshes: HashMap<UID, VertexBufferDescriptor>,
    textures: HashMap<UID, Texture>,
    materials: HashMap<UID, Material>,
    
    // Scene resources
    models: HashMap<UID, ModelInstance>,
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
        let model_buffer = ModelBuffer::new(&context, 256);
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
        let vertex_buffer = VertexBuffer::new(&context, 125000);

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
        let forward_mesh_pass = MeshPass::new(&context, &mesh_pass_bind_group_layout, 256, 256);

        Self {
            context,
            uid_generator: Default::default(),

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
            
            vertex_buffer,
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

    pub fn reset(&mut self) -> Result<()> {
        // Remove all models
        let handles = self.models.keys().copied().collect::<Vec<_>>();
        for handle in handles {
            self.remove_model(handle)?;
        }
        Ok(())
    }

    fn create_texture(&mut self, uid: UID, asset: &AssetManager) -> Result<()> {
        let texture = asset.entry::<feature::asset::texture::Texture>(uid)
            .with_context(|| "Texture not found")?;
        self.textures.insert(uid, Texture::from_asset(
            &self.context, 
            &texture.asset,
            wgpu::TextureUsages::TEXTURE_BINDING,
            Some(&texture.name),
        ));
        Ok(())
    }

    fn create_mesh(&mut self, uid: UID, asset: &AssetManager) -> Result<()> {
        let mesh = asset.get::<feature::asset::mesh::Mesh>(uid)
            .with_context(|| "Mesh asset not found")?;
        let mut submeshes: Vec<UID> = Default::default();
        for submesh in mesh.submeshes.iter() {
            let descriptor = self.vertex_buffer.add(&self.context, &submesh.vertices)
                .with_context(|| "Failed to create submesh")?;
            let submesh_uid = self.uid_generator.next();
            self.submeshes.insert(submesh_uid, descriptor);
            submeshes.push(submesh_uid);
        }
        self.meshes.insert(uid, Mesh { submeshes });
        Ok(())
    }

    fn create_material(&mut self, uid: UID, asset: &AssetManager) -> Result<()> {
        let material = asset.entry::<feature::asset::material::Material>(uid)
            .with_context(|| "Material asset not found")?;
        if !self.textures.contains_key(&material.asset.diffuse) {
            self.create_texture(material.asset.diffuse, asset)?;
        }
        let diffuse = self.textures.get(&material.asset.diffuse).expect("Texture not found");
        self.materials.insert(uid, Material { 
            texture: material.asset.diffuse,
            bind_group: create_flat_material_bind_group(
                &self.context, 
                &self.flat_material_bind_group_layout, 
                &diffuse.view,
                &material.name,
            )
        });
        Ok(())
    }

    pub fn render<F: FnOnce(&wgpu::Device, &wgpu::Queue, &mut wgpu::CommandEncoder, &wgpu::TextureView)>(
        &mut self,
        app: &App,
        app_viewport: Vec4,
        egui_pass: F,
    ) -> Result<(), SurfaceError> {
        
        // Process immediate commands
        self.surface_buffer.clear(Color::from_color_alpha(Color::BLACK, 0));
        for command in &self.command_buffers {
            self.surface_buffer.draw_command_buffer(app, command);
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
                self.forward_mesh_pass.build(&self.objects, &self.submeshes);
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

            forward_render_pass.set_vertex_buffer(0, self.vertex_buffer.position_buffer.slice(..));
            forward_render_pass.set_vertex_buffer(1, self.vertex_buffer.normal_buffer.slice(..));
            forward_render_pass.set_vertex_buffer(2, self.vertex_buffer.uv_buffer.slice(..));

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
            self.statistics.viewport = (RenderTarget::extent().width, RenderTarget::extent().height);

            // let mut previous_material: MaterialId = Default::default();
            // for batch in &self.forward_mesh_pass.instanced_batches {
                
            //     // Check change in material
            //     if batch.material != previous_material {
            //         previous_material = batch.material;
            //         let material = self.materials.get(batch.material)
            //             .expect("Failed to get material during forward pass");
            //         forward_render_pass.set_bind_group(2, &material.bind_group, &[]);
            //     }

            //     // Draw instanced
            //     let descriptor = self.submeshes.get(batch.submesh)
            //         .expect("Failed to get submesh descriptor");
            //     let vertex_start = descriptor.base_index;
            //     let vertex_stop = vertex_start + descriptor.vertex_count;
            //     let instance_start = batch.first_instance as u32;
            //     let instance_stop = batch.first_instance as u32 + batch.instance_count as u32;
            //     forward_render_pass.draw(
            //         vertex_start..vertex_stop, 
            //         instance_start..instance_stop,
            //     );
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
                app_viewport.x, 
                app_viewport.y, 
                app_viewport.z, 
                app_viewport.w, 
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
                app_viewport.x, 
                app_viewport.y, 
                app_viewport.z,
                app_viewport.w,
                0.0, 1.0
            );

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

        Ok(())
    }
}

impl RendererBackend for WGPURenderer {

    fn add_camera(&mut self) -> Result<UID> {
        Ok(Default::default())
    }
    fn remove_camera(&mut self, _handle: UID) -> Result<()> { 
        Ok(())
    }
    fn update_camera(&mut self, _handle: UID, eye: Vec3, forward: Vec3, up: Vec3, fov: f32) -> Result<()> { 
        self.camera.update(eye, forward, up, fov);
        Ok(())
    }

    fn add_model(&mut self, desc: &RendererModelDescriptor, asset: &AssetManager) -> Result<UID> { 
        match desc {
            RendererModelDescriptor::FromAsset(uid) => {

                // Find the model asset
                let model = asset.get::<feature::asset::model::Model>(*uid)
                    .with_context(|| "Model asset not found")?;

                // Create the model index
                let model_index = self.model_buffer.add();

                // Create missing meshes
                if !self.meshes.contains_key(&model.mesh) {
                    self.create_mesh(model.mesh, asset)?;
                }

                // Create missing materials
                model.materials.iter()
                    .map(|material| {
                        if !self.materials.contains_key(material) {
                            self.create_material(*material, asset)?;
                        }
                        Ok(())
                    }).collect::<Result<Vec<_>>>()?;

                // Create objects and collect ids
                let objects = self.meshes.get(&model.mesh).expect("Mesh was not created")
                    .submeshes.iter().enumerate()
                    .map(|(i, submesh)| {
                        let material = model.materials.get(i)
                            .with_context(|| format!("Missing material in model at index {}", i))?;
                        let object_uid = self.uid_generator.next();
                        self.objects.insert(object_uid, Object {
                            submesh: *submesh, 
                            material: *material,
                            model_index,
                            draw_forward_pass: true, 
                            draw_shadow_pass: false
                        });
                        // Insert in the corresponding pass
                        // TODO: check valid passes
                        self.forward_mesh_pass.add(object_uid);
                        Ok(object_uid)
                    })
                    .collect::<Result<Vec<_>>>()?;

                // Add model
                let model_uid = self.uid_generator.next();
                self.models.insert(model_uid, ModelInstance { 
                    mesh: model.mesh,
                    materials: model.materials.clone(),
                    model_index,
                    objects,
                });
                Ok(model_uid)
            },
        }
    }
    fn remove_model(&mut self, handle: UID) -> Result<()> { 

        // Remove model
        let model = self.models.remove(&handle).with_context(|| "Model not found")?;
        for object_uid in &model.objects {

            // Remove objects
            let object = self.objects.remove(object_uid).with_context(|| "Object not found")?;
            if object.draw_forward_pass {
                self.forward_mesh_pass.remove(*object_uid);
            }
            if object.draw_shadow_pass {
                // TODO: remove from pass
            }
        }

        // Remove index
        self.model_buffer.remove(model.model_index);

        Ok(())
    }
    fn update_model_transform(&mut self, handle: UID, mat: Mat4) -> Result<()> { 
        let model = self.models.get(&handle)
            .with_context(|| "Model id not found")?;
        self.model_buffer.set_transform(model.model_index, &mat);
        Ok(())
    }

    fn push_command_buffer(&mut self, command: CommandBuffer) {
        self.command_buffers.push(command);
    }
    fn reset_command_buffers(&mut self) {
        self.command_buffers.clear();
    }

    fn statistics(&self) -> RendererStatistics {
        self.statistics
    }
}