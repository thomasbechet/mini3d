use std::any::Any;

use crate::{feature::core::resource::Resource, utils::slotmap::SlotMap};

use super::{key::ResourceSlotKey, ResourceEntryKey};

pub struct PrivateResourceContainerRef<'a>(pub(crate) &'a dyn ResourceContainer);
pub struct PrivateResourceContainerMut<'a>(pub(crate) &'a mut dyn ResourceContainer);

struct ResourceEntry<R: Resource> {
    data: R,
    entry_key: ResourceEntryKey,
}

#[derive(Default)]
pub(crate) struct NativeResourceContainer<R: Resource>(SlotMap<ResourceSlotKey, ResourceEntry<R>>);

impl<R: Resource> NativeResourceContainer<R> {
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self(SlotMap::with_capacity(capacity))
    }

    pub(crate) fn add(&mut self, resource: R, entry_key: ResourceEntryKey) -> ResourceSlotKey {
        self.0.insert(ResourceEntry {
            data: resource,
            entry_key,
        })
    }

    pub(crate) fn get(&self, key: ResourceSlotKey) -> Option<&R> {
        self.0.get(key).map(|e| &e.data)
    }

    pub(crate) fn get_mut(&mut self, key: ResourceSlotKey) -> Option<&mut R> {
        self.0.get_mut(key).map(|e| &mut e.data)
    }

    pub(crate) fn get_unchecked(&self, key: ResourceSlotKey) -> &R {
        &self.0[key].data
    }

    pub(crate) fn get_mut_unchecked(&mut self, key: ResourceSlotKey) -> &mut R {
        &mut self.0[key].data
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (ResourceEntryKey, &R)> {
        self.0.values().map(|entry| (entry.entry_key, entry.data))
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (ResourceSlotKey, &mut R)> + '_ {
        self.0
            .values_mut()
            .map(|entry| (entry.entry_key, entry.data))
    }
}

pub(crate) trait ResourceContainer: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, key: ResourceSlotKey);
    fn clear(&mut self);
}

impl<R: Resource> ResourceContainer for NativeResourceContainer<R> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }

    fn remove(&mut self, key: ResourceSlotKey) {
        self.0.remove(key);
    }

    fn clear(&mut self) {
        self.0.clear();
    }
}
