use crate::ecs::container::SingleContainer;
use crate::math::mat::M4I32F16;
use crate::math::quat::QI32F16;
use crate::math::vec::{V2I32, V2I32F16, V3I32, V3I32F16, V4I32, V4I32F16};
use crate::reflection::PropertyId;
use crate::{ecs::entity::Entity, utils::uid::UID};

macro_rules! trait_property_ref_impl {
    ($type:ty, $read:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            unsafe { &*self.container }.$read(entity, id)
        }
    };
}

macro_rules! trait_property_mut_impl {
    ($type:ty, $read:ident, $write:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            unsafe { &*self.container }.$read(entity, id)
        }
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {
            unsafe { &mut *self.container }.$write(entity, id, value)
        }
    };
}

// Property single reference

pub struct PropertySingleViewRef {
    pub(crate) container: *const dyn SingleContainer,
}

impl PropertySingleViewRef {
    // pub fn resolve(
    //     &mut self,
    //     resolver: &mut SystemResolver,
    //     component: impl ToUID,
    // ) -> Result<(), ResolverError> {
    //     let id = resolver.read(component)?;
    //     self.container = resolver
    //         .containers
    //         .entries
    //         .get(id.0)
    //         .unwrap()
    //         .container
    //         .get_mut()
    //         .as_single();
    //     Ok(())
    // }

    trait_property_ref_impl!(bool, read_bool);
    trait_property_ref_impl!(u8, read_u8);
    trait_property_ref_impl!(i32, read_i32);
    trait_property_ref_impl!(u32, read_u32);
    trait_property_ref_impl!(V2I32F16, read_v2i32f16);
    trait_property_ref_impl!(V2I32, read_v2i32);
    trait_property_ref_impl!(V3I32F16, read_v3i32f16);
    trait_property_ref_impl!(V3I32, read_v3i32);
    trait_property_ref_impl!(V4I32F16, read_v4i32f16);
    trait_property_ref_impl!(V4I32, read_v4i32);
    trait_property_ref_impl!(M4I32F16, read_m4i32f16);
    trait_property_ref_impl!(QI32F16, read_qi32f16);
    trait_property_ref_impl!(Entity, read_entity);
    trait_property_ref_impl!(UID, read_uid);
}

// Property single mutable reference

pub struct PropertySingleViewMut {
    pub(crate) container: *mut dyn SingleContainer,
}

impl PropertySingleViewMut {
    // pub fn resolve(
    //     &mut self,
    //     resolver: &mut SystemResolver,
    //     component: impl ToUID,
    // ) -> Result<(), ResolverError> {
    //     let id = resolver.write(component)?;
    //     self.container = resolver
    //         .containers
    //         .entries
    //         .get_mut(id.0)
    //         .unwrap()
    //         .container
    //         .get_mut()
    //         .as_single_mut();
    //     Ok(())
    // }

    trait_property_mut_impl!(bool, read_bool, write_bool);
    trait_property_mut_impl!(u8, read_u8, write_u8);
    trait_property_mut_impl!(i32, read_i32, write_i32);
    trait_property_mut_impl!(u32, read_u32, write_u32);
    trait_property_mut_impl!(V2I32F16, read_v2i32f16, write_v2i32f16);
    trait_property_mut_impl!(V2I32, read_v2i32, write_v2i32);
    trait_property_mut_impl!(V3I32F16, read_v3i32f16, write_v3i32f16);
    trait_property_mut_impl!(V3I32, read_v3i32, write_ivec3);
    trait_property_mut_impl!(V4I32F16, read_v4i32f16, write_v4i32f16);
    trait_property_mut_impl!(V4I32, read_v4i32, write_v4i32);
    trait_property_mut_impl!(M4I32F16, read_m4i32f16, write_m4i32f16);
    trait_property_mut_impl!(QI32F16, read_qi32f16, write_qi32f16);
    trait_property_mut_impl!(Entity, read_entity, write_entity);
    trait_property_mut_impl!(UID, read_uid, write_uid);
}
