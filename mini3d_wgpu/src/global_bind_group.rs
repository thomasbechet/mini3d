use crate::{context::WGPUContext, global_uniform_buffer::GlobalUniformBuffer};

pub(crate) fn create_global_bind_group_layout(
    context: &WGPUContext
) -> wgpu::BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("global_bind_group_layout"),
        entries: &[
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
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

pub(crate) fn create_global_bind_group(
    context: &WGPUContext, 
    layout: &wgpu::BindGroupLayout, 
    global_uniform: &GlobalUniformBuffer,
    sampler: &wgpu::Sampler
) -> wgpu::BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("global_bind_group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: global_uniform.binding_resource(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
        ],
    })
}