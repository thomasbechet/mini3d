use alloc::vec::Vec;
use mini3d_utils::{slot_map_key, slotmap::SlotMap, string::AsciiArray};

use crate::{
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, FieldEntry, FieldType},
};

slot_map_key!(ComponentId);
pub(crate) type FieldIndex = u8;

pub(crate) struct ComponentEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) fields: Vec<FieldEntry>,
}

impl ComponentEntry {
    fn new(name: &str, fields: &[ComponentField]) -> Self {
        Self {
            name: AsciiArray::from(name),
            fields: fields
                .iter()
                .map(|f| FieldEntry {
                    name: AsciiArray::from(f.name),
                    data: f.create_storage(),
                })
                .collect(),
        }
    }

    fn add_default(&mut self, e: Entity) {
        for field in self.fields.iter_mut() {
            field.data.add_default(e);
        }
    }

    fn remove(&mut self, e: Entity) {
        for field in self.fields.iter_mut() {
            field.data.remove(e);
        }
    }
}

#[derive(Default)]
pub(crate) struct ComponentTable {
    pub(crate) entries: SlotMap<ComponentId, ComponentEntry>,
}

impl ComponentTable {
    pub(crate) fn find_component(&self, name: &str) -> Option<ComponentId> {
        self.entries.iter().find_map(|(id, entry)| {
            if entry.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub(crate) fn find_field(&self, c: ComponentId, name: &str) -> Option<FieldIndex> {
        self.entries[c]
            .fields
            .iter()
            .enumerate()
            .find_map(|(id, field)| {
                if field.name.as_str() == name {
                    Some(id as u8)
                } else {
                    None
                }
            })
    }

    pub(crate) fn register_tag(&mut self, name: &str) -> Result<ComponentId, ComponentError> {
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        Ok(self.entries.add(ComponentEntry::new(name, &[])))
    }

    pub fn register(
        &mut self,
        name: &str,
        fields: &[ComponentField],
    ) -> Result<ComponentId, ComponentError> {
        if self.find_component(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        Ok(self.entries.add(ComponentEntry::new(name, fields)))
    }

    pub(crate) fn add_default(&mut self, e: Entity, c: ComponentId) {
        let entry = &mut self.entries[c];
        entry.add_default(e)
    }

    pub(crate) fn remove(&mut self, e: Entity, c: ComponentId) {
        let entry = &mut self.entries[c];
        entry.remove(e)
    }

    pub(crate) fn read<T: FieldType>(&self, e: Entity, c: ComponentId, f: FieldIndex) -> Option<T> {
        let field = &self.entries[c].fields[f as usize];
        T::read(field, e)
    }

    pub(crate) fn write<T: FieldType>(&mut self, e: Entity, c: ComponentId, f: FieldIndex, v: T) {
        let field = &mut self.entries[c].fields[f as usize];
        T::write(field, e, v)
    }
}
