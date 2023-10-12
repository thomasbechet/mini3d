use std::any::Any;

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{
    ecs::{
        container::{ComponentFlags, ComponentStatus},
        entity::Entity,
        sparse::PagedVector,
    },
    reflection::PropertyId,
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::uid::UID,
};

macro_rules! trait_property {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type>;
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type);
    };
}

macro_rules! trait_property_impl {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            None
        }
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {}
    };
}

pub(crate) trait SingleContainer {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn remove(&mut self, entity: Entity);
    fn clear_changed(&mut self);
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
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

pub(crate) struct NativeSingleContainer<C: Component> {
    data: Vec<(C, Entity, ComponentFlags)>,
    indices: PagedVector<usize>, // Entity -> Index
}

impl<C: Component> NativeSingleContainer<C> {
    pub(crate) fn with_capacity(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
            indices: PagedVector::new(),
        }
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&C> {
        self.indices.get(entity.key()).and_then(|index| {
            if self.data[*index].1 == entity
                && self.data[*index].2.status() != ComponentStatus::Removed
            {
                Some(&self.data[*index].0)
            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity, cycle: u32) -> Option<&mut C> {
        self.indices.get(entity.key()).and_then(|index| {
            let tuple = &mut self.data[*index];
            if tuple.1 == entity {
                match tuple.2.status() {
                    ComponentStatus::Unchanged => {
                        tuple.2.set(ComponentStatus::Changed, cycle);
                        self.changed.push(tuple.1);
                    }
                    ComponentStatus::Changed | ComponentStatus::Added => {
                        tuple.2.set(ComponentStatus::Changed, cycle);
                    }
                    ComponentStatus::Removed => {
                        return None;
                    }
                }
                Some(&mut tuple.0)
            } else {
                None
            }
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &C> {
        self.data
            .iter()
            .filter(|(_, _, f)| !matches!(f.status(), ComponentStatus::Removed))
            .map(|(c, _, _)| c)
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.data
            .iter_mut()
            .filter(|(_, _, f)| !matches!(f.status(), ComponentStatus::Removed))
            .map(|(c, _, _)| c)
    }

    pub(crate) fn add(&mut self, entity: Entity, component: C, cycle: u32) {
        // Append component
        self.data
            .push((component, entity, ComponentFlags::added(cycle)));
        // Update indices
        self.indices.set(entity.key(), self.data.len() - 1);
    }
}

impl<C: Component> SingleContainer for NativeSingleContainer<C> {
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
        // // Write header
        // D::Header::default().serialize(&mut encoder)?;
        // // Write component count
        // encoder.write_u32(self.data.len() as u32)?;
        // // Write components
        // for (data, entity, flags) in self.data.iter() {
        //     data.serialize(&mut encoder)?;
        //     entity.serialize(&mut encoder)?;
        //     flags.serialize(&mut encoder)?;
        // }
        Ok(())
    }

    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        // // Reset container
        // self.data.clear();
        // // Read header
        // let header = C::Header::deserialize(&mut decoder, &Default::default())?;
        // // Read component count
        // let count = decoder.read_u32()?;
        // // Read components
        // for _ in 0..count {
        //     let data = C::deserialize(&mut decoder, &header)?;
        //     let entity = Entity::deserialize(&mut decoder, &Default::default())?;
        //     let flags = ComponentFlags::deserialize(&mut decoder, &Default::default())?;
        //     self.data.push((data, entity, flags));
        // }
        // // Update indices
        // for (index, (_, entity, _)) in self.data.iter().enumerate() {
        //     self.indices.set(entity.key(), index);
        // }
        Ok(())
    }

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
