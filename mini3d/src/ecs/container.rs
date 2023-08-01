use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use super::{
    entity::Entity,
    error::ECSError,
    sparse::PagedVector,
    view::{
        SceneComponentViewMut, SceneComponentViewMutInner, SceneComponentViewRef,
        SceneComponentViewRefInner,
    },
};
use crate::{
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
};
use crate::{script::reflection::PropertyId, utils::uid::UID};
use core::{any::Any, cell::RefCell};
use std::ops::{Deref, DerefMut};

pub(crate) struct StaticComponentVec<C: Component>(Vec<C>);

impl<C: Component> StaticComponentVec<C> {
    fn with_capacity(size: usize) -> Self {
        Self(Vec::with_capacity(size))
    }
}

impl<C: Component> Deref for StaticComponentVec<C> {
    type Target = Vec<C>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C: Component> DerefMut for StaticComponentVec<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

macro_rules! trait_property {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, index: usize, id: PropertyId) -> Option<$type>;
        fn $write(&mut self, index: usize, id: PropertyId, value: $type);
    };
}

macro_rules! impl_property {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, index: usize, id: PropertyId) -> Option<$type> {
            self.get(index).and_then(|c| c.$read(id))
        }
        fn $write(&mut self, index: usize, id: PropertyId, value: $type) {
            if let Some(c) = self.get_mut(index) {
                c.$write(id, value);
            }
        }
    };
}

pub(crate) trait AnyStaticComponentVec {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    trait_property!(bool, read_bool, write_bool);
    trait_property!(u8, read_u8, write_u8);
    trait_property!(i32, read_i32, write_i32);
    trait_property!(u32, read_u32, write_u32);
    trait_property!(f32, read_f32, write_f32);
    trait_property!(f64, read_f64, write_f64);
    trait_property!(Vec2, read_vec2, write_vec2);
    trait_property!(IVec2, read_ivec2, write_ivec2);
    trait_property!(Vec3, read_vec3, write_vec3);
    trait_property!(IVec3, read_ivec3, write_ivec3);
    trait_property!(Vec4, read_vec4, write_vec4);
    trait_property!(IVec4, read_ivec4, write_ivec4);
    trait_property!(Mat4, read_mat4, write_mat4);
    trait_property!(Quat, read_quat, write_quat);
    trait_property!(Entity, read_entity, write_entity);
    trait_property!(UID, read_uid, write_uid);
}

impl<C: Component> AnyStaticComponentVec for StaticComponentVec<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    impl_property!(bool, read_bool, write_bool);
    impl_property!(u8, read_u8, write_u8);
    impl_property!(i32, read_i32, write_i32);
    impl_property!(u32, read_u32, write_u32);
    impl_property!(f32, read_f32, write_f32);
    impl_property!(f64, read_f64, write_f64);
    impl_property!(Vec2, read_vec2, write_vec2);
    impl_property!(IVec2, read_ivec2, write_ivec2);
    impl_property!(Vec3, read_vec3, write_vec3);
    impl_property!(IVec3, read_ivec3, write_ivec3);
    impl_property!(Vec4, read_vec4, write_vec4);
    impl_property!(IVec4, read_ivec4, write_ivec4);
    impl_property!(Mat4, read_mat4, write_mat4);
    impl_property!(Quat, read_quat, write_quat);
    impl_property!(Entity, read_entity, write_entity);
    impl_property!(UID, read_uid, write_uid);
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ComponentStatus {
    #[default]
    Unchanged,
    Changed,
    Added,
    Removed,
}

pub(crate) struct StaticSceneContainer<C: Component> {
    pub(crate) components: RefCell<StaticComponentVec<C>>,
    pub(crate) entities: Vec<(Entity, ComponentStatus)>,
    pub(crate) indices: PagedVector<usize>,
    pub(crate) frame_size: usize,
    pub(crate) removed: Vec<Entity>,
    pub(crate) changed: Vec<Entity>,
}

impl<C: Component> StaticSceneContainer<C> {
    pub(crate) fn new() -> Self {
        Self {
            components: RefCell::new(StaticComponentVec::with_capacity(128)),
            entities: Vec::with_capacity(128),
            indices: PagedVector::new(),
            frame_size: 0,
            removed: Vec::new(),
            changed: Vec::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.frame_size
    }

    pub(crate) fn flush(&mut self) -> Result<(), ECSError> {
        for e in self.removed.drain(..) {
            if let Some(index) = self.indices.get_mut(e.key()) {
                // Swap remove component and meta
                self.components.get_mut().swap_remove(*index);
                self.entities.swap_remove(*index);
                // Remap swapped entity
                let (swapped_entity, _) = self.entities[*index];
                self.indices.set(swapped_entity.key(), *index);
                // Set the index unchanged, access will failed as the version entity is different
            }
        }
        self.frame_size = self.entities.len();
        self.changed.clear();
        Ok(())
    }

    pub(crate) fn add(&mut self, entity: Entity, component: C) -> Result<(), ECSError> {
        // Append component
        self.entities.push((entity, ComponentStatus::Added));
        self.components
            .try_borrow_mut()
            .map_err(|_| ECSError::ContainerBorrowMut)?
            .push(component);
        // Update indices
        self.indices.set(entity.key(), self.entities.len() - 1);
        Ok(())
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Result<(), ECSError> {
        if let Some(index) = self.indices.get(entity.key()).copied() {
            let (entity, status) = &mut self.entities[index];
            match *status {
                ComponentStatus::Unchanged | ComponentStatus::Changed | ComponentStatus::Added => {
                    *status = ComponentStatus::Removed;
                    self.removed.push(*entity);
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub(crate) fn added(&self) -> impl Iterator<Item = (Entity, &C)> {
        self.components.borrow_mut()[self.frame_size..]
            .iter()
            .enumerate()
            .filter(|(index, _)| {
                self.entities[self.frame_size + index].1 != ComponentStatus::Removed
            })
            .map(|(index, component)| (self.entities[self.frame_size + index].0, component))
    }

    pub(crate) fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // Write header
        C::Header::default().serialize(encoder)?;
        // Write entity count
        encoder.write_u32(self.entities.len() as u32)?;
        // Write components
        for component in self.components.borrow().iter() {
            component.serialize(encoder)?;
        }
        // Write entities
        for (entity, _) in self.entities.iter() {
            encoder.write_u32(entity.key())?;
        }
        Ok(())
    }

    pub(crate) fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // Reset container
        let mut components = self.components.borrow_mut();
        components.clear();
        self.entities.clear();
        // Read header
        let header = C::Header::deserialize(decoder, &Default::default())?;
        // Read entity count
        let count = decoder.read_u32()?;
        // Read components
        for _ in 0..count {
            let component = C::deserialize(decoder, &header)?;
            components.push(component);
        }
        // Read entities
        for _ in 0..count {
            self.entities
                .push((Entity(decoder.read_u32()?), ComponentStatus::Unchanged));
        }
        // Update indices
        for (index, (entity, _)) in self.entities.iter().enumerate() {
            self.indices.set(entity.key(), index);
        }
        Ok(())
    }
}

pub(crate) struct DynamicSceneContainer {
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: PagedVector<usize>,
}

pub(crate) trait AnySceneContainer {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn entity(&self, index: usize) -> Entity;
    fn contains(&self, entity: Entity) -> bool;
    fn len(&self) -> usize;
    fn remove(&mut self, entity: Entity);
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
    fn any_view(&self) -> SceneComponentViewRef<'_>;
    fn any_view_mut(&self) -> SceneComponentViewMut<'_>;
}

impl<C: Component> AnySceneContainer for StaticSceneContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }
    fn entity(&self, index: usize) -> Entity {
        self.entities[index].0
    }
    fn contains(&self, entity: Entity) -> bool {
        if let Some(index) = self.indices.get(entity.key()).copied() {
            index < self.entities.len() && self.entities[index].0 == entity
        } else {
            false
        }
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn remove(&mut self, entity: Entity) {
        self.remove(entity).unwrap();
    }
    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        self.serialize(&mut encoder)
    }
    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        self.deserialize(&mut decoder)
    }
    fn any_view(&self) -> SceneComponentViewRef<'_> {
        SceneComponentViewRef(SceneComponentViewRefInner::Static {
            components: self.components.borrow(),
            entities: &self.entities,
            indices: &self.indices,
        })
    }
    fn any_view_mut(&self) -> SceneComponentViewMut<'_> {
        SceneComponentViewMut(SceneComponentViewMutInner::Static {
            components: self.components.borrow_mut(),
            entities: &self.entities,
            indices: &self.indices,
        })
    }
}

impl AnySceneContainer for DynamicSceneContainer {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn entity(&self, index: usize) -> Entity {
        todo!()
    }
    fn contains(&self, entity: Entity) -> bool {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
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
    fn any_view(&self) -> SceneComponentViewRef<'_> {
        todo!()
    }
    fn any_view_mut(&self) -> SceneComponentViewMut<'_> {
        todo!()
    }
}
