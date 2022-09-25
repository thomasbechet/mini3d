pub(crate) struct DrawCommandBuffer {
    pub(crate) buffer: wgpu::Buffer,
}

impl DrawCommandBuffer {
    pub(crate) fn new() -> Self {
        let command = wgpu::util::DrawIndirect {
            vertex_count: todo!(),
            instance_count: todo!(),
            base_vertex: todo!(),
            base_instance: todo!(),
        };
    }
}