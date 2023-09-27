use std::any::Any;

use crate::{
    ecs::{entity::Entity, sparse::PagedVector},
    registry::component::ComponentData,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
};

use super::{ComponentFlags, ComponentStatus};

pub(crate) trait AnyArrayContainer {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: Entity);
    fn clear_changed(&mut self);
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
}

struct StaticArrayEntry {
    entity: Entity,
    flags: ComponentFlags,
    start: usize,
}

pub(crate) struct StaticArrayContainer<C: ComponentData> {
    array_length: usize,
    data: Vec<C>,
    entries: Vec<StaticArrayEntry>,
    indices: PagedVector<usize>, // Entity -> Entry Index
    changed: Vec<Entity>,
}

impl<C: ComponentData> StaticArrayContainer<C> {
    pub(crate) fn with_capacity(size: usize, array_length: usize) -> Self {
        Self {
            array_length,
            data: Vec::with_capacity(size * array_length),
            entries: Vec::with_capacity(size),
            indices: PagedVector::new(),
            changed: Vec::with_capacity(size),
        }
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&[C]> {
        self.indices.get(entity.key()).and_then(|index| {
            if self.entries[*index].entity == entity
                && self.entries[*index].flags.status() != ComponentStatus::Removed
            {
                let start = self.entries[*index].start;
                Some(&self.data[start..start + self.array_length])
            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity, cycle: u32) -> Option<&mut [C]> {
        self.indices.get(entity.key()).and_then(|index| {
            let entry = &mut self.entries[*index];
            if entry.entity == entity {
                match entry.flags.status() {
                    ComponentStatus::Unchanged => {
                        entry.flags.set(ComponentStatus::Changed, cycle);
                        self.changed.push(entry.entity);
                    }
                    ComponentStatus::Changed | ComponentStatus::Added => {
                        entry.flags.set(ComponentStatus::Changed, cycle);
                    }
                    ComponentStatus::Removed => {
                        return None;
                    }
                }
                let start = entry.start;
                Some(&mut self.data[start..start + self.array_length])
            } else {
                None
            }
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &[C]> {
        self.entries
            .iter()
            .filter(|entry| !matches!(entry.flags.status(), ComponentStatus::Removed))
            .map(|entry| &self.data[entry.start..entry.start + self.array_length])
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut [C]> {
        self.entries
            .iter()
            .filter(|entry| !matches!(entry.flags.status(), ComponentStatus::Removed))
            .map(|entry| &mut self.data[entry.start..entry.start + self.array_length])
    }

    pub(crate) fn add(&mut self, entity: Entity, components: &[C], cycle: u32) {
        // Append component
        self.data
            .push((component, entity, ComponentFlags::added(cycle)));
        // Update indices
        self.indices.set(entity.key(), self.data.len() - 1);
    }
}

impl<C: ComponentData> AnyArrayContainer for StaticArrayContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }

    fn remove(&mut self, entity: Entity) {
        let index = *self.indices.get(entity.key()).expect("Entity not found");
        // Swap remove component
        self.data.swap_remove(index);
        // Remap swapped entity
        if index != self.data.len() {
            let swapped_entity = self.data[index].1;
            self.indices.set(swapped_entity.key(), index);
        }
    }

    fn clear_changed(&mut self) {
        self.changed.clear();
    }

    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        // Write header
        C::Header::default().serialize(&mut encoder)?;
        // Write component count
        encoder.write_u32(self.data.len() as u32)?;
        // Write components
        for (data, entity, flags) in self.data.iter() {
            data.serialize(&mut encoder)?;
            entity.serialize(&mut encoder)?;
            flags.serialize(&mut encoder)?;
        }
        Ok(())
    }

    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        // Reset container
        self.data.clear();
        // Read header
        let header = C::Header::deserialize(&mut decoder, &Default::default())?;
        // Read component count
        let count = decoder.read_u32()?;
        // Read components
        for _ in 0..count {
            let data = C::deserialize(&mut decoder, &header)?;
            let entity = Entity::deserialize(&mut decoder, &Default::default())?;
            let flags = ComponentFlags::deserialize(&mut decoder, &Default::default())?;
            self.data.push((data, entity, flags));
        }
        // Update indices
        for (index, (_, entity, _)) in self.data.iter().enumerate() {
            self.indices.set(entity.key(), index);
        }
        Ok(())
    }
}

pub(crate) struct DynamicArrayContainer {
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: PagedVector<usize>,
}

impl AnyArrayContainer for DynamicArrayContainer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, entity: Entity) {
        todo!()
    }

    fn clear_changed(&mut self) {
        todo!()
    }

    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        todo!()
    }

    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        todo!()
    }
}
