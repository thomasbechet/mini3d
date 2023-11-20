use crate::{feature::core::resource::Resource, utils::slotmap::SlotMap};

use super::{
    container::NativeResourceContainer, handle::ResourceHandle, ResourceEntry, ResourceEntryKey,
};

pub(crate) struct Wrapper {
    pub(crate) key: ResourceEntryKey,
}

pub(crate) struct TypedResourceIterator<'a> {
    pub(crate) iter: std::slice::Iter<'a, Wrapper>,
}

impl<'a> TypedResourceIterator<'a> {
    pub(crate) fn empty() -> Self {
        Self { iter: &[].iter() }
    }
}

impl<'a> Iterator for TypedResourceIterator<'a> {
    type Item = ResourceEntryKey;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|wrapper| &wrapper.key).copied()
    }
}

pub(crate) struct TypedNativeResourceIteratorMut<'a, R: Resource> {
    entries: &'a SlotMap<ResourceEntryKey, ResourceEntry>,
    container: Option<&'a mut NativeResourceContainer<R>>,
    current: ResourceHandle,
}

impl<'a, R: Resource> TypedNativeResourceIteratorMut<'a, R> {
    pub(crate) fn new(
        entries: &'a SlotMap<ResourceEntryKey, ResourceEntry>,
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
            let entry = &self.entries[self.current];
            self.current = entry.next;
            let data = &mut self.container.take().unwrap().0[entry.slot];
            Some((current, data))
        }
    }
}
