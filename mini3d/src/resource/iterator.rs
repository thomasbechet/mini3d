use crate::{feature::core::resource::Resource, utils::slotmap::SlotMap};

use super::{container::NativeResourceContainer, handle::ResourceHandle, ResourceEntry};

pub struct TypedResourceIterator<'a> {
    entries: &'a SlotMap<ResourceEntry>,
    current: ResourceHandle,
}

impl<'a> TypedResourceIterator<'a> {
    pub(crate) fn new(entries: &'a SlotMap<ResourceEntry>, current: ResourceHandle) -> Self {
        Self { entries, current }
    }
}

impl<'a> Iterator for TypedResourceIterator<'a> {
    type Item = ResourceHandle;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            None
        } else {
            let entry = &self.entries[self.current.0];
            self.current = entry.next;
            Some(self.current)
        }
    }
}

pub(crate) struct TypedNativeResourceIteratorMut<'a, R: Resource> {
    entries: &'a SlotMap<ResourceEntry>,
    container: Option<&'a mut NativeResourceContainer<R>>,
    current: ResourceHandle,
}

impl<'a, R: Resource> TypedNativeResourceIteratorMut<'a, R> {
    pub(crate) fn new(
        entries: &'a SlotMap<ResourceEntry>,
        container: Option<&'a mut NativeResourceContainer<R>>,
        current: ResourceHandle,
    ) -> Self {
        Self {
            entries,
            container,
            current,
        }
    }
}

impl<'a, R: Resource> Iterator for TypedNativeResourceIteratorMut<'a, R> {
    type Item = (ResourceHandle, &'a mut R);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() || self.container.is_none() {
            None
        } else {
            let current = self.current;
            let entry = &self.entries[self.current.0];
            self.current = entry.next;
            Some((current, &mut self.container.unwrap().0[entry.slot]))
        }
    }
}
