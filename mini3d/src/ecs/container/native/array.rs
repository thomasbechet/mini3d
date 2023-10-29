use std::any::Any;

use crate::{
    ecs::{
        container::{ArrayContainer, Container},
        entity::Entity,
        sparse::PagedVector,
    },
    feature::core::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
};

struct NativeArrayEntry {
    entity: Entity,
    chunk_index: usize,
}

pub(crate) struct NativeArrayContainer<C: Component> {
    chunk_size: usize,
    data: Vec<C>,
    entries: Vec<NativeArrayEntry>,
    indices: PagedVector<usize>, // Entity -> Entry Index
}

impl<C: Component> NativeArrayContainer<C> {
    pub(crate) fn with_capacity(size: usize, chunk_size: usize) -> Self {
        Self {
            chunk_size,
            data: Vec::with_capacity(size * chunk_size),
            entries: Vec::with_capacity(size),
            indices: PagedVector::new(),
        }
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&[C]> {
        self.indices.get(entity.key()).and_then(|index| {
            if self.entries[*index].entity == entity {
                let start = self.entries[*index].chunk_index * self.chunk_size;
                Some(&self.data[start..start + self.chunk_size])
            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity) -> Option<&mut [C]> {
        self.indices.get(entity.key()).and_then(|index| {
            let entry = &mut self.entries[*index];
            if entry.entity == entity {
                let start = entry.chunk_index * self.chunk_size;
                Some(&mut self.data[start..start + self.chunk_size])
            } else {
                None
            }
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &[C]> {
        let chunks = self.data.chunks_exact(self.chunk_size);
        self.entries
            .iter()
            .zip(chunks.into_iter())
            .map(|(_, data)| data)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut [C]> {
        let chunks = self.data.chunks_exact_mut(self.chunk_size);
        self.entries
            .iter()
            .zip(chunks.into_iter())
            .map(|(_, data)| data)
    }

    pub(crate) fn add<const S: usize>(&mut self, entity: Entity, components: [C; S], cycle: u32) {
        // Allocate chunk
        let chunk_index = self.data.len() / self.chunk_size;
        // Fill chunk
        self.data.extend(components);
        // Append entry
        self.entries.push(NativeArrayEntry {
            entity,
            chunk_index,
        });
        // Update indices
        self.indices.set(entity.key(), self.data.len() - 1);
    }
}

impl<C: Component> Container for NativeArrayContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }

    fn remove(&mut self, entity: Entity) {
        let index = *self.indices.get(entity.key()).expect("Entity not found");
        let chunk_index = self.entries[index].chunk_index;
        // Swap remove chunk
        if index != self.entries.len() - 1 {
            let start = chunk_index * self.chunk_size;
            let last_start = (self.entries.len() - 1) * self.chunk_size;
            for i in 0..self.chunk_size {
                self.data.swap(start + i, last_start + i);
            }
        }
        self.data.truncate(self.entries.len() * self.chunk_size);
        // Swap remove entry
        self.entries.swap_remove(index);
        // Remap swapped entity
        if index != self.entries.len() {
            let swapped_entity = self.entries[index].entity;
            self.indices.set(swapped_entity.key(), index);
        }
    }

    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        Ok(())
    }
}

impl<C: Component> ArrayContainer for NativeArrayContainer<C> {}

pub(crate) struct DynamicArrayContainer {
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: PagedVector<usize>,
}

impl Container for DynamicArrayContainer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, entity: Entity) {
        todo!()
    }

    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        todo!()
    }

    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        todo!()
    }
}

impl ArrayContainer for DynamicArrayContainer {}
