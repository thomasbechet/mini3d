use core::any::Any;

use alloc::vec::Vec;

use crate::{
    ecs::{
        component::{Component, ComponentContext, ComponentKey},
        container::{Container, SingleContainer},
        context::Context,
        entity::{Entity, EntityTable},
        query::QueryTable,
        sparse::PagedVector,
    },
    math::{
        mat::M4I32F16,
        quat::QI32F16,
        vec::{V2I32, V2I32F16, V3I32, V3I32F16, V4I32, V4I32F16},
    },
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
}

impl<C: Component> NativeSingleContainer<C> {
    pub(crate) fn with_capacity(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
            indices: PagedVector::new(),
            view_size: 0,
            removed: Vec::new(),
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

    pub(crate) fn add(&mut self, ctx: &mut Context, entity: Entity, mut component: C) {
        // Check existing component
        if let Some(index) = self.indices.get(entity.key()) {
            if self.data[*index].1 == entity {
                // Update component value
                self.data[*index].0 = component;
                return;
            }
        }
        // Hook added
        component.on_added(ComponentContext {
            input: ctx.input,
            renderer: ctx.renderer,
        });
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

    fn as_single(&self) -> &dyn SingleContainer {
        self
    }

    fn as_single_mut(&mut self) -> &mut dyn SingleContainer {
        self
    }

    fn mark_removed(&mut self, entity: Entity) {
        self.removed.push(entity);
    }

    fn remove(&mut self, ctx: &mut Context, entity: Entity) {
        if let Some(index) = self.indices.get(entity.key()).copied() {
            if self.data[index].1 == entity {
                // Hook remove
                self.data[index].0.on_removed(ComponentContext {
                    input: ctx.input,
                    renderer: ctx.renderer,
                });
                // Swap remove component
                self.data.swap_remove(index);
                // Remap swapped entity
                if index != self.data.len() {
                    let swapped_entity = self.data[index].1;
                    self.indices.set(swapped_entity.key(), index);
                }
            }
        }
    }

    fn flush_added_removed(
        &mut self,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        component: ComponentKey,
    ) {
        // Added components
        for (data, entity) in self.data[self.view_size..].iter() {
            // Move entity
            entities.move_added_entity(queries, *entity, component);
        }
        // Removed components
        while let Some(entity) = self.removed.pop() {
            // Move entity
            entities.move_removed_entity(queries, entity, component);
            // Remove component
            self.remove(entity);
        }
    }

    fn update_view_size(&mut self) {
        self.view_size = self.data.len();
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
    trait_property_impl!(V2I32F16, read_v2i32f16, write_v2i32f16);
    trait_property_impl!(V2I32, read_v2i32, write_v2i32);
    trait_property_impl!(V3I32F16, read_v3i32f16, write_v3i32f16);
    trait_property_impl!(V3I32, read_v3i32, write_ivec3);
    trait_property_impl!(V4I32F16, read_v4i32f16, write_v4i32f16);
    trait_property_impl!(V4I32, read_v4i32, write_v4i32);
    trait_property_impl!(M4I32F16, read_m4i32f16, write_m4i32f16);
    trait_property_impl!(QI32F16, read_qi32f16, write_qi32f16);
    trait_property_impl!(Entity, read_entity, write_entity);
    trait_property_impl!(UID, read_uid, write_uid);
}
