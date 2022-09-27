use crate::context::WGPUContext;

pub(crate) fn create_flat_material_bind_group_layout(
    context: &WGPUContext
) -> wgpu::BindGroupLayout {
    context.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("flat_material_bind_group_layout"),
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
        ],
    })
}

pub(crate) fn create_flat_material_bind_group(
    context: &WGPUContext, 
    layout: &wgpu::BindGroupLayout,
    texture_view: &wgpu::TextureView,
    label: &str,
) -> wgpu::BindGroup {
    context.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            },
        ],
    })
}