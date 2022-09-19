use wgpu::include_wgsl;

use crate::context::WGPUContext;

pub(crate) fn create_blit_shader_module(
    context: &WGPUContext,
) -> wgpu::ShaderModule {
    context.device.create_shader_module(include_wgsl!("shader/blit.wgsl"))
}

pub(crate) fn create_blit_pipeline_layout(
    context: &WGPUContext,
    blit_bind_group_layout: &wgpu::BindGroupLayout
) -> wgpu::PipelineLayout {
    context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("blit_pipeline_layout"),
        bind_group_layouts: &[blit_bind_group_layout],
        push_constant_ranges: &[],
    })
}

pub(crate) fn create_blit_pipeline(
    context: &WGPUContext,
    blit_pipeline_layout: &wgpu::PipelineLayout,
    blit_shader_module: &wgpu::ShaderModule,
    target_format: wgpu::TextureFormat,
    blend_state: wgpu::BlendState,
    label: &str,
) -> wgpu::RenderPipeline {

    // Create pipeline
    context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(&blit_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &blit_shader_module,
            entry_point: "vs_main",
            buffers: &[],
        },
        fragment: Some(wgpu::FragmentState {
            module: &blit_shader_module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: target_format,
                blend: Some(blend_state),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            polygon_mode: wgpu::PolygonMode::Fill,
            unclipped_depth: false,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}