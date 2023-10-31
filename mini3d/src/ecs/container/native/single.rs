use std::any::Any;

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{
    ecs::{
        container::{Container, SingleContainer},
        entity::{Entity, EntityTable},
        query::QueryTable,
        sparse::PagedVector,
    },
    feature::core::component::{Component, ComponentId},
    reflection::PropertyId,
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    trait_property_impl,
    utils::uid::UID,
};

pub(crate) struct NativeSingleContainer<C: Component> {
    data: Vec<(C, Entity)>,
    indices: PagedVector<usize>, // Entity -> Index
    view_size: usize,
    removed: Vec<Entity>,
    id: ComponentId,
}

impl<C: Component> NativeSingleContainer<C> {
    pub(crate) fn with_capacity(size: usize, id: ComponentId) -> Self {
        Self {
            data: Vec::with_capacity(size),
            indices: PagedVector::new(),
            removed: Vec::new(),
            view_size: 0,
            id,
        }
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&C> {
        self.indices.get(entity.key()).and_then(|index| {
            // TODO: check index in view size ?
            if self.data[*index].1 == entity {
                Some(&self.data[*index].0)
            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.indices.get(entity.key()).and_then(|index| {
            // TODO: check index in view size ?
            let tuple = &mut self.data[*index];
            if tuple.1 == entity {
                Some(&mut tuple.0)
            } else {
                None
            }
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &C> {
        self.data[..self.view_size].iter().map(|(c, _)| c)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.data[..self.view_size].iter_mut().map(|(c, _)| c)
    }

    pub(crate) fn add(&mut self, entity: Entity, component: C) {
        // Append component
        self.data.push((component, entity));
        // Update indices
        self.indices.set(entity.key(), self.data.len() - 1);
        // Structural change is implicit as we can find added components
        // by comparing the size of the data with the size at the beginning
        // of update.
    }
}

impl<C: Component> Container for NativeSingleContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }

    fn mark_removed(&mut self, entity: Entity) {
        self.removed.push(entity);
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

    fn flush_changes(&mut self, entities: &mut EntityTable, queries: &mut QueryTable) {
        // Added components
        for (data, entity) in self.data[self.view_size..self.data.len()].iter() {
            // TODO: notify systems ?
            // Find currrent archetype
            let current_archetype = entities.entries.get(entity.key()).unwrap().archetype;
            // Find next archetype
            let archetype = entities
                .archetypes
                .find_add(queries, current_archetype, self.id);
            // Update archetype
            entities.entries.get_mut(entity.key()).unwrap().archetype = archetype;
        }
        // Update size
        self.view_size = self.data.len();
        // Removed components
        for entity in self.removed.drain(..) {
            self.remove(entity);
        }
    }

    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        Ok(())
    }
}

impl<C: Component> SingleContainer for NativeSingleContainer<C> {
    trait_property_impl!(bool, read_bool, write_bool);
    trait_property_impl!(u8, read_u8, write_u8);
    trait_property_impl!(i32, read_i32, write_i32);
    trait_property_impl!(u32, read_u32, write_u32);
    trait_property_impl!(f32, read_f32, write_f32);
    trait_property_impl!(f64, read_f64, write_f64);
    trait_property_impl!(Vec2, read_vec2, write_vec2);
    trait_property_impl!(IVec2, read_ivec2, write_ivec2);
    trait_property_impl!(Vec3, read_vec3, write_vec3);
    trait_property_impl!(IVec3, read_ivec3, write_ivec3);
    trait_property_impl!(Vec4, read_vec4, write_vec4);
    trait_property_impl!(IVec4, read_ivec4, write_ivec4);
    trait_property_impl!(Mat4, read_mat4, write_mat4);
    trait_property_impl!(Quat, read_quat, write_quat);
    trait_property_impl!(Entity, read_entity, write_entity);
    trait_property_impl!(UID, read_uid, write_uid);
}
