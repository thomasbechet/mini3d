use wgpu::{include_wgsl, vertex_attr_array};

use crate::{context::WGPUContext, viewport::VIEWPORT_COLOR_FORMAT};

pub(crate) fn create_flat_pipeline(
    context: &WGPUContext,
    global_bind_group_layout: &wgpu::BindGroupLayout,
    mesh_pass_bind_group_layout: &wgpu::BindGroupLayout,
    flat_material_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    
    // Pipeline layout
    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("flat_pipeline_layout"),
        bind_group_layouts: &[
            global_bind_group_layout,
            mesh_pass_bind_group_layout, 
            flat_material_bind_group_layout
        ],
        push_constant_ranges: &[],
    });
    
    // Vertex layouts
    let position_layout = wgpu::VertexBufferLayout {
        array_stride: wgpu::VertexFormat::Float32x3.size() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![0 => Float32x3]
    };
    let normal_layout = wgpu::VertexBufferLayout {
        array_stride: wgpu::VertexFormat::Float32x3.size() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![1 => Float32x3]
    };
    let uv_layout = wgpu::VertexBufferLayout {
        array_stride: wgpu::VertexFormat::Float32x2.size() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &vertex_attr_array![2 => Float32x2]
    };

    // Compile modules
    let module = context.device.create_shader_module(include_wgsl!("shader/flat.wgsl"));

    // Create pipeline
    context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("flat_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: "vs_main",
            buffers: &[position_layout, normal_layout, uv_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: VIEWPORT_COLOR_FORMAT,
                blend: Some(wgpu::BlendState::REPLACE),
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
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::Less,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        multiview: None,
    })
}