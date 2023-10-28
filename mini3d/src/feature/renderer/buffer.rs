use crate::define_resource_handle;

define_resource_handle!(BufferHandle);

pub enum BufferType {}

pub struct BufferDescriptor {
    pub buffer_type: BufferType,
    pub size: usize,
}
