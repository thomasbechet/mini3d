use std::{
    cell::{Ref, RefMut},
    ops::{Deref, Index, IndexMut},
};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{registry::component::Component, script::reflection::PropertyId, uid::UID};

use super::{
    container::{AnyStaticComponentVec, StaticComponentContainer, StaticComponentVec},
    entity::Entity,
    sparse::PagedVector,
};

pub trait StaticComponentView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

struct StaticComponentViewRefData<'a, C: Component> {
    components: Ref<'a, StaticComponentVec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

pub struct StaticComponentViewRef<'a, C: Component> {
    view: Option<StaticComponentViewRefData<'a, C>>,
}

impl<'a, C: Component> StaticComponentViewRef<'a, C> {
    pub(crate) fn new(container: &'a StaticComponentContainer<C>) -> Self {
        Self {
            view: Some(StaticComponentViewRefData {
                components: container.components.borrow(),
                entities: &container.entities,
                indices: &container.indices,
            }),
        }
    }

    pub(crate) fn none() -> Self {
        Self { view: None }
    }

    pub fn iter(&self) -> impl Iterator<Item = &C> {
        if let Some(data) = &self.view {
            data.components.iter()
        } else {
            [].iter()
        }
    }
}

impl<'a, C: Component> StaticComponentView<C> for StaticComponentViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.view.as_ref().and_then(|data| {
            data.indices.get(entity.key()).copied().and_then(|index| {
                if data.entities[index] == entity {
                    Some(&data.components[index])
                } else {
                    None
                }
            })
        })
    }
}

impl<'a, C: Component> Index<Entity> for StaticComponentViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

struct StaticComponentViewMutData<'a, C: Component> {
    components: RefMut<'a, StaticComponentVec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

pub struct StaticComponentViewMut<'a, C: Component> {
    view: Option<StaticComponentViewMutData<'a, C>>,
}

impl<'a, C: Component> StaticComponentViewMut<'a, C> {
    pub(crate) fn new(container: &'a StaticComponentContainer<C>) -> Self {
        Self {
            view: Some(StaticComponentViewMutData {
                components: container.components.borrow_mut(),
                entities: &container.entities,
                indices: &container.indices,
            }),
        }
    }

    pub(crate) fn none() -> Self {
        Self { view: None }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &mut C> {
        if let Some(data) = &mut self.view {
            data.components.iter_mut()
        } else {
            [].iter_mut()
        }
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.view.as_mut().and_then(|data| {
            data.indices.get(entity.key()).and_then(|index| {
                if data.entities[*index] == entity {
                    Some(&mut data.components[*index])
                } else {
                    None
                }
            })
        })
    }
}

impl<'a, C: Component> StaticComponentView<C> for StaticComponentViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.view.as_ref().and_then(|data| {
            data.indices.get(entity.key()).and_then(|index| {
                if data.entities[*index] == entity {
                    Some(&data.components[*index])
                } else {
                    None
                }
            })
        })
    }
}

impl<'a, C: Component> Index<Entity> for StaticComponentViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IndexMut<Entity> for StaticComponentViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub(crate) enum AnyComponentViewRefInner<'a> {
    Static {
        components: Ref<'a, dyn AnyStaticComponentVec>,
        entities: &'a [Entity],
        indices: &'a PagedVector<usize>,
    },
    Dynamic {},
    None,
}

pub struct AnyComponentViewRef<'a>(pub(crate) AnyComponentViewRefInner<'a>);

pub(crate) enum AnyComponentViewMutInner<'a> {
    Static {
        components: RefMut<'a, dyn AnyStaticComponentVec>,
        entities: &'a [Entity],
        indices: &'a PagedVector<usize>,
    },
    Dynamic {},
    None,
}

pub struct AnyComponentViewMut<'a>(pub(crate) AnyComponentViewMutInner<'a>);

macro_rules! impl_read_property {
    ($type:ty, $read:ident) => {
        pub fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            match &self.0 {
                AnyComponentViewRefInner::Static {
                    components,
                    entities,
                    indices,
                } => indices.get(entity.key()).copied().and_then(|index| {
                    if entities[index] == entity {
                        components.$read(index, id)
                    } else {
                        None
                    }
                }),
                AnyComponentViewRefInner::Dynamic {} => None,
                AnyComponentViewRefInner::None => None,
            }
        }
    };
}

macro_rules! impl_read_write_property {
    ($type:ty, $read:ident, $write:ident) => {
        pub fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            match &self.0 {
                AnyComponentViewMutInner::Static {
                    components,
                    entities,
                    indices,
                } => indices.get(entity.key()).copied().and_then(|index| {
                    if entities[index] == entity {
                        components.$read(index, id)
                    } else {
                        None
                    }
                }),
                AnyComponentViewMutInner::Dynamic {} => None,
                AnyComponentViewMutInner::None => None,
            }
        }
        pub fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {
            match &mut self.0 {
                AnyComponentViewMutInner::Static {
                    components,
                    entities,
                    indices,
                } => {
                    if let Some(index) = indices.get(entity.key()).copied() {
                        if entities[index] == entity {
                            components.$write(index, id, value)
                        }
                    }
                }
                AnyComponentViewMutInner::Dynamic {} => {}
                AnyComponentViewMutInner::None => {}
            }
        }
    };
}

impl<'a> AnyComponentViewRef<'a> {
    pub(crate) fn none() -> Self {
        Self(AnyComponentViewRefInner::None)
    }

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

impl<'a> AnyComponentViewMut<'a> {
    pub(crate) fn none() -> Self {
        Self(AnyComponentViewMutInner::None)
    }

    impl_read_write_property!(bool, read_bool, write_bool);
    impl_read_write_property!(u8, read_u8, write_u8);
    impl_read_write_property!(i32, read_i32, write_i32);
    impl_read_write_property!(u32, read_u32, write_u32);
    impl_read_write_property!(f32, read_f32, write_f32);
    impl_read_write_property!(f64, read_f64, write_f64);
    impl_read_write_property!(Vec2, read_vec2, write_vec2);
    impl_read_write_property!(IVec2, read_ivec2, write_ivec2);
    impl_read_write_property!(Vec3, read_vec3, write_vec3);
    impl_read_write_property!(IVec3, read_ivec3, write_ivec3);
    impl_read_write_property!(Vec4, read_vec4, write_vec4);
    impl_read_write_property!(IVec4, read_ivec4, write_ivec4);
    impl_read_write_property!(Mat4, read_mat4, write_mat4);
    impl_read_write_property!(Quat, read_quat, write_quat);
    impl_read_write_property!(Entity, read_entity, write_entity);
    impl_read_write_property!(UID, read_uid, write_uid);
}
