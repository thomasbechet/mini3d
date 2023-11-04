use crate::ecs::container::SingleContainer;
use crate::ecs::error::ResolverError;
use crate::ecs::system::SystemResolver;
use crate::reflection::PropertyId;
use crate::utils::uid::ToUID;
use crate::{ecs::entity::Entity, utils::uid::UID};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

macro_rules! trait_property_ref_impl {
    ($type:ty, $read:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            unsafe { *self.container }.$read(entity, id)
        }
    };
}

macro_rules! trait_property_mut_impl {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            unsafe { *self.container }.$read(entity, id)
        }
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {
            unsafe { *self.container }.$write(entity, id, value)
        }
    };
}

// Property single reference

pub struct PropertySingleViewRef {
    pub(crate) container: *const dyn SingleContainer,
}

impl PropertySingleViewRef {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.read(component)?;
        self.container = resolver
            .containers
            .entries
            .get(id.0)
            .unwrap()
            .container
            .as_single();
        Ok(())
    }

    trait_property_ref_impl!(bool, read_bool);
    trait_property_ref_impl!(u8, read_u8);
    trait_property_ref_impl!(i32, read_i32);
    trait_property_ref_impl!(u32, read_u32);
    trait_property_ref_impl!(f32, read_f32);
    trait_property_ref_impl!(f64, read_f64);
    trait_property_ref_impl!(Vec2, read_vec2);
    trait_property_ref_impl!(IVec2, read_ivec2);
    trait_property_ref_impl!(Vec3, read_vec3);
    trait_property_ref_impl!(IVec3, read_ivec3);
    trait_property_ref_impl!(Vec4, read_vec4);
    trait_property_ref_impl!(IVec4, read_ivec4);
    trait_property_ref_impl!(Mat4, read_mat4);
    trait_property_ref_impl!(Quat, read_quat);
    trait_property_ref_impl!(Entity, read_entity);
    trait_property_ref_impl!(UID, read_uid);
}

// Property single mutable reference

pub struct PropertySingleViewMut {
    pub(crate) container: *mut dyn SingleContainer,
}

impl PropertySingleViewMut {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.write(component)?;
        self.container = resolver
            .containers
            .entries
            .get_mut(id.0)
            .unwrap()
            .container
            .as_single_mut();
        Ok(())
    }

    trait_property_mut_impl!(bool, read_bool, write_bool);
    trait_property_mut_impl!(u8, read_u8, write_u8);
    trait_property_mut_impl!(i32, read_i32, write_i32);
    trait_property_mut_impl!(u32, read_u32, write_u32);
    trait_property_mut_impl!(f32, read_f32, write_f32);
    trait_property_mut_impl!(f64, read_f64, write_f64);
    trait_property_mut_impl!(Vec2, read_vec2, write_vec2);
    trait_property_mut_impl!(IVec2, read_ivec2, write_ivec2);
    trait_property_mut_impl!(Vec3, read_vec3, write_vec3);
    trait_property_mut_impl!(IVec3, read_ivec3, write_ivec3);
    trait_property_mut_impl!(Vec4, read_vec4, write_vec4);
    trait_property_mut_impl!(IVec4, read_ivec4, write_ivec4);
    trait_property_mut_impl!(Mat4, read_mat4, write_mat4);
    trait_property_mut_impl!(Quat, read_quat, write_quat);
    trait_property_mut_impl!(Entity, read_entity, write_entity);
    trait_property_mut_impl!(UID, read_uid, write_uid);
}
