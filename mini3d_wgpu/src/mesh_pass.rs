use std::collections::HashMap;

use mini3d::{uid::UID, renderer::backend::MaterialHandle};

use crate::{Object, model_buffer::ModelIndex, context::WGPUContext, vertex_allocator::VertexBufferDescriptor, error::WGPURendererError};

pub(crate) fn create_mesh_pass_bind_group_layout(
    context: &WGPUContext
) -> wgpu::BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("mesh_pass_bind_group_layout"),
        entries: &[
            // Instances Data
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer { 
                    ty: wgpu::BufferBindingType::Uniform, 
                    has_dynamic_offset: false, 
                    min_binding_size: wgpu::BufferSize::new(64), 
                },
                count: None,
            },
            // Commands Data
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
        ],
    })
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUInstanceData {
    // Use to get object information (like bounding box)
    pub(crate) model_id: u32,
    // Use to identify the associated batch
    pub(crate) batch_id: u32,
    // Ensure 16 bytes alignment
    _pad0: u64,
}

#[repr(C)]
#[derive(Default, Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct GPUDrawIndirect {
    pub(crate) vertex_count: u32,
    pub(crate) instance_count: u32,
    pub(crate) base_vertex: u32,
    pub(crate) base_instance: u32,
}

struct PassObject {
    // instance_id: usize,
    sort_key: u32,
}

pub(crate) struct RenderBatch {
    pub(crate) submesh: UID,
    pub(crate) material: MaterialHandle,
    pub(crate) model_index: ModelIndex,
}

pub(crate) struct InstancedRenderBatch {
    pub(crate) submesh: UID,
    pub(crate) material: MaterialHandle,
    pub(crate) first_instance: usize,
    pub(crate) instance_count: usize,
    pub(crate) triangle_count: usize,
}

pub(crate) struct MultiInstancedRenderBatch {
    pub(crate) material: MaterialHandle,
    pub(crate) first: usize,
    pub(crate) count: usize,
    pub(crate) triangle_count: usize,
}

pub(crate) struct MeshPass {

    max_pass_object_count: usize,
    max_pass_command_count: usize,

    pass_objects: HashMap<UID, PassObject>,

    // Non-instanced sorted batches
    batches: Vec<RenderBatch>,
    // Instanced sorted batches
    pub(crate) instanced_batches: Vec<InstancedRenderBatch>,
    // Multi-Instanced sorted batches
    pub(crate) multi_instanced_batches: Vec<MultiInstancedRenderBatch>,
    
    // Keep mapping between objects and batches
    instances: Box<[GPUInstanceData]>,
    pub(crate) instance_buffer: wgpu::Buffer,

    // Use only when use indirect draw calls
    indirect_commands: Box<[GPUDrawIndirect]>,
    pub(crate) indirect_command_buffer: wgpu::Buffer,

    // Bind group
    pub(crate) bind_group: wgpu::BindGroup,

    out_of_date: bool,
}

impl MeshPass {

    pub(crate) fn new(
        context: &WGPUContext,
        layout: &wgpu::BindGroupLayout,
        max_pass_object_count: usize, 
        max_pass_command_count: usize,
    ) -> Self {

        let instance_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instance_buffer"), // TODO: custom name
            size: (std::mem::size_of::<GPUInstanceData>() * max_pass_object_count) as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let indirect_command_buffer = context.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("indirect_buffer"), // TODO: custom name
            size: (std::mem::size_of::<GPUDrawIndirect>() * max_pass_command_count) as u64,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = context.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mesh_pass_bind_group"), // TODO: custom name
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: instance_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: indirect_command_buffer.as_entire_binding(),
                }
            ],
        });

        Self { 
            max_pass_object_count,
            max_pass_command_count,

            pass_objects: Default::default(),

            batches: Default::default(),
            instanced_batches: Default::default(),
            multi_instanced_batches: Default::default(),

            instances: vec![GPUInstanceData::default(); max_pass_object_count].into_boxed_slice(),
            instance_buffer,

            indirect_commands: vec![GPUDrawIndirect::default(); max_pass_command_count].into_boxed_slice(), 
            indirect_command_buffer,

            bind_group,

            out_of_date: true,
        }
    }

    pub(crate) fn add(&mut self, uid: UID) -> Result<(), WGPURendererError> {
        if self.pass_objects.len() >= self.max_pass_object_count {
            return Err(WGPURendererError::MaxPassObjectReached);
        }
        self.pass_objects.insert(uid, PassObject { 
            sort_key: 0
        });
        self.out_of_date = true;
        Ok(())
    }

    pub(crate) fn remove(&mut self, uid: UID) {
        self.pass_objects.remove(&uid).expect("Pass object not found");
        self.out_of_date = true;
    }

    pub(crate) fn out_of_date(&self) -> bool {
        self.out_of_date
    }

    pub(crate) fn build(
        &mut self,
        objects: &HashMap<UID, Object>,
        submeshes: &HashMap<UID, VertexBufferDescriptor>,
    ) {

        // Create sorted batches from object pass
        {
            // Collect batches info (old batches are erased)
            self.batches = self.pass_objects.keys().map(|id| RenderBatch {
                    // TODO: use key to change draw order ?
                    submesh: objects.get(id).unwrap().submesh,
                    material: objects.get(id).unwrap().material,
                    model_index: objects.get(id).unwrap().model_index,
                })
                .collect::<Vec<_>>();

            // Sort batches by material then by submesh
            self.batches.sort_by_key(|r| (r.material, r.submesh));
        }

        // Build compact instanced batches
        {
            // Clear batches
            self.instanced_batches.clear();

            // Insert first batch, will be used for the first comparison
            if !self.batches.is_empty() {
                self.instanced_batches.push(InstancedRenderBatch {
                    submesh: self.batches.first().unwrap().submesh,
                    material: self.batches.first().unwrap().material,
                    first_instance: 0,
                    instance_count: 0,
                    triangle_count: 0,
                });
            }

            // Prepare instance object id
            for (instance_id, batch) in self.batches.iter().enumerate() {
                            
                // Compare with previous batch
                let same_submesh = batch.submesh == self.instanced_batches.last().unwrap().submesh;
                let same_material = batch.material == self.instanced_batches.last().unwrap().material;
            
                // Compare the batch
                if same_submesh && same_material {
                    self.instanced_batches.last_mut().unwrap().instance_count += 1;
                } else {
                    self.instanced_batches.push(InstancedRenderBatch { 
                        submesh: batch.submesh, 
                        material: batch.material, 
                        first_instance: instance_id, 
                        instance_count: 1,
                        triangle_count: 0,
                    });
                }

                // Write instance data
                self.instances[instance_id].model_id = batch.model_index as u32;
                self.instances[instance_id].batch_id = (self.instanced_batches.len() - 1) as u32;
            }
        }

        // Write indirect command and compute total triangle count
        for (batch_id, batch) in self.instanced_batches.iter_mut().enumerate() {
            self.indirect_commands[batch_id].base_instance  = batch.first_instance as u32;
            self.indirect_commands[batch_id].instance_count = batch.instance_count as u32;
            let descriptor = submeshes.get(&batch.submesh).unwrap();
            self.indirect_commands[batch_id].base_vertex = descriptor.base_index;
            self.indirect_commands[batch_id].vertex_count = descriptor.vertex_count;
            batch.triangle_count = (descriptor.vertex_count / 3) as usize * batch.instance_count;
        }

        // Build multi instanced batches
        {
            // Clear batches
            self.multi_instanced_batches.clear();

            // Insert first group
            if !self.instanced_batches.is_empty() {
                self.multi_instanced_batches.push(MultiInstancedRenderBatch { 
                    material: self.instanced_batches.first().unwrap().material,
                    first: 0,
                    count: 0,
                    triangle_count: 0,
                });
            }

            // Build multi instanced render batches
            for (batch_id, batch) in self.instanced_batches.iter().enumerate() {
                if batch.material == self.multi_instanced_batches.last().unwrap().material {
                    let multi_batch = self.multi_instanced_batches.last_mut().unwrap();
                    multi_batch.count += 1;
                    multi_batch.triangle_count += batch.triangle_count;
                } else {
                    self.multi_instanced_batches.push(MultiInstancedRenderBatch { 
                        material: batch.material, 
                        first: batch_id, 
                        count: 1,
                        triangle_count: batch.triangle_count,
                    });
                }
            }
        }

        // Reset flag
        self.out_of_date = false;
    }

    pub(crate) fn write_buffers(&self, context: &WGPUContext) {
        context.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&self.instances));
        context.queue.write_buffer(&self.indirect_command_buffer, 0, bytemuck::cast_slice(&self.indirect_commands));
    }
}