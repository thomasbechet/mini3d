use std::collections::HashMap;

use mini3d::renderer::backend::{CanvasViewportHandle, CanvasHandle};

use crate::{context::WGPUContext, model_buffer::ModelBuffer, canvas::Canvas, camera::Camera};

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUViewportData {
    world_to_clip: [f32; 16],
    _pad: [u64; 24],
}

pub(crate) const MAX_VIEWPORT_COUNT: usize = 32; 

pub(crate) struct ViewportRenderer {

    viewport_buffer: wgpu::Buffer,
    viewport_bind_group_layout: wgpu::BindGroupLayout,
    viewport_bind_group: wgpu::BindGroup,
    viewport_transfer: [GPUViewportData; MAX_VIEWPORT_COUNT],
    viewport_offsets: HashMap<CanvasViewportHandle, u32>,
}

impl ViewportRenderer {

    pub(crate) fn new(
        context: &WGPUContext,
        model_buffer: &ModelBuffer,
        sampler: &wgpu::Sampler,
    ) -> Self {
        
        let viewport_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("viewport_buffer"),
            size: std::mem::size_of::<GPUViewportData>() as u64 * MAX_VIEWPORT_COUNT as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let viewport_bind_group_layout = context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("viewport_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<GPUViewportData>() as u64), 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer { 
                        ty: wgpu::BufferBindingType::Uniform, 
                        has_dynamic_offset: false, 
                        min_binding_size: wgpu::BufferSize::new(64), 
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let viewport_bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("viewport_bind_group"),
            layout: &viewport_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: viewport_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: model_buffer.buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        });

        Self { 
            viewport_bind_group_layout, 
            viewport_bind_group,
            viewport_buffer,
            viewport_transfer: [GPUViewportData::default(); MAX_VIEWPORT_COUNT], 
            viewport_offsets: Default::default(), 
        }
    }

    pub(crate) fn write_buffer(
        &mut self,
        context: &WGPUContext,
        canvases: &HashMap<CanvasHandle, Canvas>,
        cameras: &HashMap<CanvasViewportHandle, Camera>,
    ) {
        let mut current_viewport_index = 0;
        for canvas in canvases.values() {
            for (handle, viewport) in &canvas.viewports {
                
                // Fill buffer
                let camera = cameras.get(handle).unwrap();
                let projection = camera.projection(viewport.aspect_ratio());
                let view = camera.view();
                self.viewport_transfer[current_viewport_index].world_to_clip = (projection * view).to_cols_array();
                
                // Save offset
                let offset = std::mem::size_of::<GPUViewportData>() * current_viewport_index;
                self.viewport_offsets.insert(*handle, offset as u32);

                current_viewport_index += 1;
            }
        }

        // Write buffers
        context.queue.write_buffer(&self.viewport_buffer, 0, bytemuck::cast_slice(&self.viewport_transfer[0..current_viewport_index]));
    }

    pub(crate) fn render(
        &mut self,
        canvases: &HashMap<CanvasHandle, Canvas>,
        flat_pipeline: &wgpu::RenderPipeline,
        encoder: &mut wgpu::CommandEncoder,
    ) {

        for canvas in canvases.values() {
            for (handle, viewport) in &canvas.viewports {

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

                    forward_render_pass.set_pipeline(flat_pipeline);
                    let offset = self.viewport_offsets.get(handle).unwrap();
                    forward_render_pass.set_bind_group(0, &self.viewport_bind_group, &[*offset]);
                    // forward_render_pass.set_bind_group(1, &self.forward_mesh_pass.bind_group, &[]);

                    // forward_render_pass.set_vertex_buffer(0, self.vertex_allocator.position_buffer.slice(..));
                    // forward_render_pass.set_vertex_buffer(1, self.vertex_allocator.normal_buffer.slice(..));
                    // forward_render_pass.set_vertex_buffer(2, self.vertex_allocator.uv_buffer.slice(..));

                    // // Multi draw indirect
                    // {
                    //     let mut triangle_count = 0;
                    //     for batch in &self.forward_mesh_pass.multi_instanced_batches {

                    //         // Bind materials
                    //         let material = self.materials.get(&batch.material)
                    //             .expect("Failed to get material during forward pass");
                    //         forward_render_pass.set_bind_group(2, &material.bind_group, &[]);
                        
                    //         // Indirect draw
                    //         forward_render_pass.multi_draw_indirect(
                    //             &self.forward_mesh_pass.indirect_command_buffer, 
                    //             (std::mem::size_of::<GPUDrawIndirect>() * batch.first) as u64, 
                    //             batch.count as u32,
                    //         );
                    //         triangle_count += batch.triangle_count;
                    //     }
                    //     self.statistics.draw_count = self.forward_mesh_pass.multi_instanced_batches.len();
                    //     self.statistics.triangle_count = triangle_count;
                    // }
                    
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

    }
}