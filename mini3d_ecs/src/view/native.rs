use crate::{component::Component, container::native::NativeSingleContainer};

pub struct NativeSingleMut<C: Component> {
    pub(crate) ptr: *mut NativeSingleContainer<C>,
}

pub struct NativeSingleRef<C: Component> {
    pub(crate) ptr: *const NativeSingleContainer<C>,
}
