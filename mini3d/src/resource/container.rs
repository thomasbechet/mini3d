use std::any::Any;

use crate::{feature::core::resource::Resource, utils::slotmap::SlotMap};

use super::{key::ResourceSlotKey, ResourceEntryKey};

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

    pub(crate) fn add(&mut self, resource: R, key: ResourceEntryKey) -> ResourceSlotKey {
        self.0.add(ResourceEntry {
            data: resource,
            entry_key: key,
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

    pub(crate) fn iter(&self) -> impl Iterator<Item = (ResourceSlotKey, &R)> {
        self.0
            .iter()
            .map(|(slot_key, entry)| (slot_key, &entry.data))
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (ResourceSlotKey, &mut R)> + '_ {
        self.0
            .iter_mut()
            .map(|(slot_key, entry)| (slot_key, &mut entry.data))
    }
}

pub(crate) trait NativeContainer: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, key: ResourceSlotKey);
    fn clear(&mut self);
    fn get_entry_key(&self, key: ResourceSlotKey) -> Option<ResourceEntryKey>;
    fn iter_keys(&self) -> Box<dyn Iterator<Item = (ResourceEntryKey, ResourceSlotKey)> + '_>;
}

impl<R: Resource> NativeContainer for NativeResourceContainer<R> {
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

    fn get_entry_key(&self, key: ResourceSlotKey) -> Option<ResourceEntryKey> {
        self.0.get(key).map(|e| e.entry_key)
    }

    fn iter_keys(&self) -> Box<dyn Iterator<Item = (ResourceEntryKey, ResourceSlotKey)> + '_> {
        Box::new(
            self.0
                .iter()
                .map(|(slot_key, entry)| (entry.entry_key, slot_key)),
        )
    }
}
