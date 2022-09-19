use wgpu::{include_wgsl, vertex_attr_array};

use crate::{context::WGPUContext, render_target::RenderTarget, instance_uniform_buffer::GPUInstanceData};

pub(crate) fn create_scene_pipeline(
    context: &WGPUContext,
    global_bind_group_layout: &wgpu::BindGroupLayout,
    material_bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::RenderPipeline {
    
    // Pipeline layout
    let pipeline_layout = context.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("scene_pipeline_layout"),
        bind_group_layouts: &[&global_bind_group_layout, &material_bind_group_layout],
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
    let instance_layout = wgpu::VertexBufferLayout {
        array_stride: (GPUInstanceData::size()) as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: wgpu::VertexFormat::Float32x4.size() * 0,
                shader_location: 3,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: wgpu::VertexFormat::Float32x4.size() * 1,
                shader_location: 4,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: wgpu::VertexFormat::Float32x4.size() * 2,
                shader_location: 5,
            },
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: wgpu::VertexFormat::Float32x4.size() * 3,
                shader_location: 6,
            },
        ]
    };

    // Compile modules
    let module = context.device.create_shader_module(include_wgsl!("shader/scene.wgsl"));

    // Create pipeline
    context.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("scene_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: "vs_main",
            buffers: &[position_layout, normal_layout, uv_layout, instance_layout],
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format: RenderTarget::format(),
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