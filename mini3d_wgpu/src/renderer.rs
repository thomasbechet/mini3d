use std::f32::consts::FRAC_PI_4;

use mini3d::glam::{UVec2, Vec4, Mat4, Vec3, Quat};
use mini3d::graphics::SCREEN_ASPECT_RATIO;
use mini3d::{
    application::Application,
    graphics::{
        SCREEN_HEIGHT, SCREEN_WIDTH,
    },
};
use wgpu::SurfaceError;

use crate::blit_bind_group::{create_blit_bind_group_layout, create_blit_bind_group};
use crate::blit_pipeline::{create_blit_pipeline_layout, create_blit_pipeline, create_blit_shader_module};
use crate::global_bind_group::{create_global_bind_group, create_global_bind_group_layout};
use crate::global_uniform_buffer::GlobalUniformBuffer;
use crate::instance_uniform_buffer::InstanceUniformBuffer;
use crate::context::WGPUContext;
use crate::material_bind_group::{create_material_bind_group_layout, create_material_bind_group};
use crate::render_target::RenderTarget;
use crate::scene_pipeline::create_scene_pipeline;
use crate::surface_buffer::{SurfaceBuffer, Color};
use crate::texture::Texture;
use crate::vertex_buffer::VertexBuffer;

pub fn compute_fixed_viewport(size: UVec2) -> Vec4 {
    if size.x as f32 / size.y as f32 >= (SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32) {
        let w = SCREEN_WIDTH as f32 * size.y as f32 / SCREEN_HEIGHT as f32;
        let h = size.y as f32;
        let x = (size.x / 2) as f32 - (w / 2.0);
        let y = 0.0;
        (x, y, w, h).into()
    } else {
        let w = size.x as f32;
        let h = SCREEN_HEIGHT as f32 * size.x as f32 / SCREEN_WIDTH as f32;
        let x = 0.0;
        let y = (size.y / 2) as f32 - (h / 2.0);
        (x, y, w, h).into()
    }
}

pub struct WGPURenderer {
    // Context
    context: WGPUContext,
    
    // Scene Render Pass
    instance_uniform_buffer: InstanceUniformBuffer,
    global_uniform_buffer: GlobalUniformBuffer,
    global_bind_group: wgpu::BindGroup,
    scene_pipeline: wgpu::RenderPipeline,
    material_bind_group_layout: wgpu::BindGroupLayout,
    car_vertex_buffer: Option<VertexBuffer>,
    car_texture: Option<Texture>,
    car_bind_group: Option<wgpu::BindGroup>,
    
    // Surface Render Pass
    surface_buffer: SurfaceBuffer,
    surface_bind_group: wgpu::BindGroup,
    surface_pipeline: wgpu::RenderPipeline,
    
    // Post Process Render Pass
    render_target: RenderTarget,
    post_process_bind_group: wgpu::BindGroup,
    post_process_pipeline: wgpu::RenderPipeline,

    angle: f32
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

        let global_bind_group_layout = create_global_bind_group_layout(&context);
        let material_bind_group_layout = create_material_bind_group_layout(&context);
        let global_uniform_buffer = GlobalUniformBuffer::new(&context);
        let global_bind_group = create_global_bind_group(
            &context, 
            &global_bind_group_layout, 
            &global_uniform_buffer,
            &nearest_sampler,
        );
        let scene_pipeline = create_scene_pipeline(
            &context, 
            &global_bind_group_layout,
            &material_bind_group_layout,
        );
        let car_vertex_buffer: Option<VertexBuffer> = None;
        let car_texture: Option<Texture> = None;
        let car_bind_group: Option<wgpu::BindGroup> = None;
        let instance_uniform_buffer = InstanceUniformBuffer::new(&context);

        //////// Surface Render Pass ////////
         
        let surface_buffer = SurfaceBuffer::new(&context);
        let blit_bind_group_layout = create_blit_bind_group_layout(&context);
        let blit_pipeline_layout = create_blit_pipeline_layout(&context, &blit_bind_group_layout);
        let blit_shader_module = create_blit_shader_module(&context);
        let surface_bind_group = create_blit_bind_group(
            &context, 
            &blit_bind_group_layout, 
            &surface_buffer.texture_view, 
            &nearest_sampler, 
            "surface_blit_bind_group"
        );
        let surface_pipeline = create_blit_pipeline(
            &context, 
            &blit_pipeline_layout, 
            &blit_shader_module, 
            RenderTarget::format(), 
            wgpu::BlendState::ALPHA_BLENDING,
            "surface_blit_pipeline"
        );

        //////// Post Process Render Pass ////////
        
        let render_target = RenderTarget::new(&context);
        let post_process_bind_group = create_blit_bind_group(
            &context, 
            &blit_bind_group_layout, 
            &render_target.render_view, 
            &nearest_sampler, 
            "post_process_bind_group"
        );
        let post_process_pipeline = create_blit_pipeline(
            &context, 
            &blit_pipeline_layout, 
            &blit_shader_module, 
            context.config.format, 
            wgpu::BlendState::REPLACE, 
            "post_process_pipeline"
        );

        Self {
            context,
            instance_uniform_buffer,
            global_uniform_buffer,
            global_bind_group,
            scene_pipeline,
            material_bind_group_layout,
            car_vertex_buffer,
            car_texture,
            car_bind_group,
            surface_buffer,
            surface_bind_group,
            surface_pipeline,
            render_target,
            post_process_bind_group,
            post_process_pipeline,
            angle: 0.0,
        }
    }

    pub fn recreate(&mut self) {
        self.context.recreate();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.context.resize(width, height);
        }
    }

    pub fn render(&mut self, app: &Application) -> Result<(), SurfaceError> {
        
        // Process immediate commands
        self.surface_buffer.clear(Color::from_color_alpha(Color::BLACK, 0));
        self.surface_buffer.draw_immediate_commands(&app);

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

        // Update uniform buffers
        self.angle += 0.02;

        // Model Matrix

        let model_count = 256;
        let model_row = 16;
        let model_range = 16.0;
        for i in 0..model_count {
            let x = i % model_row;
            let y = i / model_row;
            let model = Mat4::from_scale_rotation_translation(
                Vec3::ONE * 0.2, 
                Quat::from_rotation_y(self.angle), 
                Vec3::new(
                    (x as f32 / model_row as f32) * model_range, 
                    0.0,
                    (y as f32 / model_row as f32) * model_range, 
                )
            );
            self.instance_uniform_buffer.set_model(i, &model);
        }

        self.instance_uniform_buffer.write_buffer(&self.context);

        // Camera Matrix
        let projection = Mat4::perspective_rh(FRAC_PI_4, SCREEN_ASPECT_RATIO, 0.5, 50.0);
        let view = Mat4::look_at_rh(
            Vec3::new(8.0, 3.0, 0.0),
            Vec3::new(model_range * 0.5, 0.0, model_range * 0.5),
            Vec3::Y,
        );
        
        self.global_uniform_buffer.set_world_to_clip(&(projection * view));
        self.global_uniform_buffer.write_buffer(&self.context);

        // Create car material if missing
        if self.car_bind_group.is_none() {

            self.car_texture = Some(Texture::from_asset(
                &self.context,
                &app.assets.textures.get_from_name("car"), 
                wgpu::TextureUsages::TEXTURE_BINDING,
                Some("car_texture")
            ));
            self.car_bind_group = Some(create_material_bind_group(
                &self.context, 
                &self.material_bind_group_layout, 
                &self.car_texture.as_ref().unwrap().view, 
                "car_bind_group"
            ));
            self.car_vertex_buffer = Some(VertexBuffer::from_vertices(
                &self.context, 
                &app.assets.meshes.get_from_name("car").submeshes[0].vertices, 
                Some("car_mesh")
            ));
        }

        // Update Surface Buffer
        self.surface_buffer.write_texture(&self.context);

        // Scene Render Pass
        {
            let mut scene_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene_render_pass"),
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

            let car_vertex_buffer = self.car_vertex_buffer.as_ref().unwrap();
            let car_bind_group = self.car_bind_group.as_ref().unwrap();

            scene_render_pass.set_pipeline(&self.scene_pipeline);
            scene_render_pass.set_bind_group(0, &self.global_bind_group, &[]);
            scene_render_pass.set_bind_group(1, car_bind_group, &[]);
            scene_render_pass.set_vertex_buffer(0, car_vertex_buffer.position_buffer.slice(..));
            scene_render_pass.set_vertex_buffer(1, car_vertex_buffer.normal_buffer.slice(..));
            scene_render_pass.set_vertex_buffer(2, car_vertex_buffer.uv_buffer.slice(..));
            scene_render_pass.set_vertex_buffer(3, self.instance_uniform_buffer.buffer.slice(..));
            scene_render_pass.draw(0..car_vertex_buffer.vertex_count as u32, 0..(model_count as u32));   
        }

        {
            let mut surface_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("surface_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.render_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            surface_render_pass.set_pipeline(&self.surface_pipeline);
            surface_render_pass.set_bind_group(0, &self.surface_bind_group, &[]);
            surface_render_pass.draw(0..3, 0..1);        
        }

        // Post Process Render Pass
        {
            let mut post_process_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post_process_render_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 25.0 / 255.0,
                            g: 27.0 / 255.0,
                            b: 43.0 / 255.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            // Compute viewport        
            let viewport = {
                let surface_size: UVec2 = (self.context.config.width, self.context.config.height).into();
                compute_fixed_viewport(surface_size)
            };
            post_process_render_pass.set_viewport(viewport.x, viewport.y, viewport.z, viewport.w, 0.0, 1.0);
        
            post_process_render_pass.set_pipeline(&self.post_process_pipeline);
            post_process_render_pass.set_bind_group(0, &self.post_process_bind_group, &[]);
            post_process_render_pass.draw(0..3, 0..1);
        }

        // Submit queue and present
        self.context.queue.submit(Some(encoder.finish()));
        output.present();

        Ok(())
    }
}