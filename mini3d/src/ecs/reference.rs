use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::uid::UID;

use super::entity::Entity;
use crate::ecs::view::SceneComponentViewRef;
use crate::ecs::view::SceneComponentViewRefInner;
use crate::script::reflection::PropertyId;

pub struct SceneComponentRef<'a> {
    _lifetime: core::marker::PhantomData<&'a ()>,
    entity: Entity,
    index: usize,
}

macro_rules! impl_read_property {
    ($type:ty, $read:ident) => {
        pub fn $read(&self, id: PropertyId, view: &SceneComponentViewRef) -> Option<$type> {
            match &view.0 {
                SceneComponentViewRefInner::Static {
                    components,
                    entities,
                    indices: _,
                } => {
                    if entities[self.index] == self.entity {
                        components.$read(self.index, id)
                    } else {
                        None
                    }
                }
                SceneComponentViewRefInner::Dynamic {} => None,
                SceneComponentViewRefInner::None => None,
            }
        }
    };
}

impl<'a> SceneComponentRef<'a> {
    impl_read_property!(bool, read_bool);
    impl_read_property!(u8, read_u8);
    impl_read_property!(i32, read_i32);
    impl_read_property!(u32, read_u32);
    impl_read_property!(f32, read_f32);
    impl_read_property!(f64, read_f64);
    impl_read_property!(Vec2, read_vec2);
    impl_read_property!(IVec2, read_ivec2);
    impl_read_property!(Vec3, read_vec3);
    impl_read_property!(IVec3, read_ivec3);
    impl_read_property!(Vec4, read_vec4);
    impl_read_property!(IVec4, read_ivec4);
    impl_read_property!(Mat4, read_mat4);
    impl_read_property!(Quat, read_quat);
    impl_read_property!(Entity, read_entity);
    impl_read_property!(UID, read_uid);
}

pub struct SceneComponentMut<'a> {
    _lifetime: core::marker::PhantomData<&'a ()>,
    entity: Entity,
    index: usize,
}

macro_rules! impl_read_write_property {
    ($type:ty, $read:ident, $write:ident) => {
        pub fn $read(&self, id: PropertyId, view: &AnyComponentViewMut) -> Option<$type> {
            match &self.0 {
                SceneComponentViewMutInner::Static {
                    components,
                    entities,
                    indices,
                } => {
                    if entities[self.index] == self.entity {
                        components.$read(self.index, id)
                    } else {
                        None
                    }
                }
                SceneComponentViewMutInner::Dynamic {} => None,
                SceneComponentViewMutInner::None => None,
            }
        }
        pub fn $write(&self, id: PropertyId, value: $type, view: &mut AnyComponentViewMut) {
            match &self.0 {
                SceneComponentViewMutInner::Static {
                    components,
                    entities,
                    indices,
                } => {
                    if entities[self.index] == self.entity {
                        components.$write(self.index, id)
                    } else {
                        None
                    }
                }
                SceneComponentViewMutInner::Dynamic {} => None,
                SceneComponentViewMutInner::None => None,
            }
        }
    };
}
