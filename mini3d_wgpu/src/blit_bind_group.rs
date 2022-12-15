use crate::context::WGPUContext;

pub(crate) fn create_blit_bind_group_layout(
    context: &WGPUContext,
) -> wgpu::BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("blit_bind_group_layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
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

pub(crate) fn create_blit_bind_group(
    context: &WGPUContext,
    blit_bind_group_layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    label: Option<&str>,
) -> wgpu::BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label,
        layout: blit_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(texture_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}