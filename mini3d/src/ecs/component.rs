use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};
use mini3d_derive::Serialize;

use super::{entity::Entity, error::SceneError, sparse::PagedVector};
use crate::{
    registry::component::{Component, ComponentHandle, ComponentId, ComponentRegistry},
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SparseSecondaryMap,
};
use crate::{script::reflection::PropertyId, utils::uid::UID};
use core::{any::Any, cell::RefCell};

#[derive(Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ComponentStatus {
    #[default]
    Unchanged = 0b00,
    Changed = 0b01,
    Added = 0b10,
    Removed = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
struct ComponentFlags(u32);

impl ComponentFlags {
    fn added(cycle: u32) -> Self {
        let mut flags = Self(0);
        flags.set(ComponentStatus::Added, cycle);
        flags
    }

    fn cycle(&self) -> u32 {
        self.0 >> 2
    }

    fn status(&self) -> ComponentStatus {
        match self.0 & 0b11 {
            0b00 => ComponentStatus::Unchanged,
            0b01 => ComponentStatus::Changed,
            0b10 => ComponentStatus::Added,
            0b11 => ComponentStatus::Removed,
            _ => unreachable!(),
        }
    }

    fn set(&mut self, status: ComponentStatus, cycle: u32) {
        self.0 = ((cycle << 2) & (!0b11)) | status as u32;
    }
}

macro_rules! trait_property {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type>;
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type);
    };
}

macro_rules! impl_property_static {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            None
        }
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {}
    };
}

macro_rules! impl_property_dynamic {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            None
        }
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {}
    };
}

pub(crate) trait AnyComponentContainer {
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

pub(crate) struct StaticComponentContainer<C: Component> {
    data: Vec<(C, Entity, ComponentFlags)>,
    indices: PagedVector<usize>, // Entity -> Index
    changed: Vec<Entity>,
}

impl<C: Component> StaticComponentContainer<C> {
    pub(crate) fn with_capacity(size: usize) -> Self {
        Self {
            data: Vec::with_capacity(size),
            indices: PagedVector::new(),
            changed: Vec::with_capacity(size),
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
                return None;
            }
        })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &C> {
        self.data
            .iter()
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

impl<C: Component> AnyComponentContainer for StaticComponentContainer<C> {
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

    impl_property_static!(bool, read_bool, write_bool);
    impl_property_static!(u8, read_u8, write_u8);
    impl_property_static!(i32, read_i32, write_i32);
    impl_property_static!(u32, read_u32, write_u32);
    impl_property_static!(f32, read_f32, write_f32);
    impl_property_static!(f64, read_f64, write_f64);
    impl_property_static!(Vec2, read_vec2, write_vec2);
    impl_property_static!(IVec2, read_ivec2, write_ivec2);
    impl_property_static!(Vec3, read_vec3, write_vec3);
    impl_property_static!(IVec3, read_ivec3, write_ivec3);
    impl_property_static!(Vec4, read_vec4, write_vec4);
    impl_property_static!(IVec4, read_ivec4, write_ivec4);
    impl_property_static!(Mat4, read_mat4, write_mat4);
    impl_property_static!(Quat, read_quat, write_quat);
    impl_property_static!(Entity, read_entity, write_entity);
    impl_property_static!(UID, read_uid, write_uid);
}

pub(crate) struct DynamicComponentContainer {
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: PagedVector<usize>,
}

impl AnyComponentContainer for DynamicComponentContainer {
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

    impl_property_dynamic!(bool, read_bool, write_bool);
    impl_property_dynamic!(u8, read_u8, write_u8);
    impl_property_dynamic!(i32, read_i32, write_i32);
    impl_property_dynamic!(u32, read_u32, write_u32);
    impl_property_dynamic!(f32, read_f32, write_f32);
    impl_property_dynamic!(f64, read_f64, write_f64);
    impl_property_dynamic!(Vec2, read_vec2, write_vec2);
    impl_property_dynamic!(IVec2, read_ivec2, write_ivec2);
    impl_property_dynamic!(Vec3, read_vec3, write_vec3);
    impl_property_dynamic!(IVec3, read_ivec3, write_ivec3);
    impl_property_dynamic!(Vec4, read_vec4, write_vec4);
    impl_property_dynamic!(IVec4, read_ivec4, write_ivec4);
    impl_property_dynamic!(Mat4, read_mat4, write_mat4);
    impl_property_dynamic!(Quat, read_quat, write_quat);
    impl_property_dynamic!(Entity, read_entity, write_entity);
    impl_property_dynamic!(UID, read_uid, write_uid);
}

#[derive(Default)]
pub(crate) struct ComponentTable {
    pub(crate) containers: SparseSecondaryMap<RefCell<Box<dyn AnyComponentContainer>>>,
}

impl ComponentTable {
    pub(crate) fn preallocate(&mut self, component: ComponentId, registry: &ComponentRegistry) {
        if !self.containers.contains(component.into()) {
            let container = registry
                .get(component)
                .unwrap()
                .reflection
                .create_scene_container();
            self.containers
                .insert(component.into(), RefCell::new(container));
        }
    }

    pub(crate) fn serialize(
        &self,
        registry: &ComponentRegistry,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        encoder.write_u32(self.containers.len() as u32)?;
        for (id, container) in self.containers.iter() {
            let uid = UID::new(&registry.get(id.into()).unwrap().name);
            uid.serialize(encoder)?;
            container.borrow().serialize(encoder)?;
        }
        Ok(())
    }

    pub(crate) fn deserialize(
        &mut self,
        registry: &ComponentRegistry,
        decoder: &mut impl Decoder,
    ) -> Result<(), DecoderError> {
        self.containers.clear();
        let count = decoder.read_u32()?;
        for i in 0..count {
            let uid = UID::deserialize(decoder, &Default::default())?;
            let id = registry
                .find_id(uid)
                .expect("Component ID not found while deserializing");
            self.preallocate(id, registry);
            self.containers[id.into()]
                .borrow_mut()
                .deserialize(decoder)?;
        }
        Ok(())
    }

    pub(crate) fn add_static<C: Component>(
        &mut self,
        entity: Entity,
        component: ComponentId,
        data: C,
        cycle: u32,
    ) {
        self.containers
            .get_mut(component.into())
            .expect("Component container not found while adding entity")
            .get_mut()
            .as_any_mut()
            .downcast_mut::<StaticComponentContainer<C>>()
            .expect("Component type mismatch while adding static component")
            .add(entity, data, cycle);
    }

    pub(crate) fn remove(&mut self, entity: Entity, component: ComponentId) {
        self.containers
            .get_mut(component.into())
            .expect("Component container not found while removing entity")
            .get_mut()
            .remove(entity);
    }

    pub(crate) fn view<H: ComponentHandle>(
        &self,
        component: H,
    ) -> Result<H::ViewRef<'_>, SceneError> {
        component.view_ref(self)
    }

    pub(crate) fn view_mut<H: ComponentHandle>(
        &self,
        component: H,
        cycle: u32,
    ) -> Result<H::ViewMut<'_>, SceneError> {
        component.view_mut(self, cycle)
    }
}
