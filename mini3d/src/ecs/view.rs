use std::{
    cell::{Ref, RefMut},
    ops::{Index, IndexMut},
};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{registry::component::Component, script::reflection::PropertyId, utils::uid::UID};

use super::{
    container::{AnyStaticComponentVec, StaticComponentVec, StaticSceneContainer},
    entity::Entity,
    sparse::PagedVector,
};

pub trait StaticSceneComponentView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

struct StaticSceneComponentViewRefData<'a, C: Component> {
    components: Ref<'a, StaticComponentVec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

pub struct StaticSceneComponentViewRef<'a, C: Component> {
    view: Option<StaticSceneComponentViewRefData<'a, C>>,
}

impl<'a, C: Component> StaticSceneComponentViewRef<'a, C> {
    pub(crate) fn new(container: &'a StaticSceneContainer<C>) -> Self {
        Self {
            view: Some(StaticSceneComponentViewRefData {
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

impl<'a, C: Component> StaticSceneComponentView<C> for StaticSceneComponentViewRef<'a, C> {
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

impl<'a, C: Component> Index<Entity> for StaticSceneComponentViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

struct StaticSceneComponentViewMutData<'a, C: Component> {
    components: RefMut<'a, StaticComponentVec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

pub struct StaticSceneComponentViewMut<'a, C: Component> {
    view: Option<StaticSceneComponentViewMutData<'a, C>>,
}

impl<'a, C: Component> StaticSceneComponentViewMut<'a, C> {
    pub(crate) fn new(container: &'a StaticSceneContainer<C>) -> Self {
        Self {
            view: Some(StaticSceneComponentViewMutData {
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

impl<'a, C: Component> StaticSceneComponentView<C> for StaticSceneComponentViewMut<'a, C> {
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

impl<'a, C: Component> Index<Entity> for StaticSceneComponentViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IndexMut<Entity> for StaticSceneComponentViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub(crate) enum SceneComponentViewRefInner<'a> {
    Static {
        components: Ref<'a, dyn AnyStaticComponentVec>,
        entities: &'a [Entity],
        indices: &'a PagedVector<usize>,
    },
    Dynamic {},
    None,
}

pub struct SceneComponentViewRef<'a>(pub(crate) SceneComponentViewRefInner<'a>);

pub(crate) enum SceneComponentViewMutInner<'a> {
    Static {
        components: RefMut<'a, dyn AnyStaticComponentVec>,
        entities: &'a [Entity],
        indices: &'a PagedVector<usize>,
    },
    Dynamic {},
    None,
}

pub struct SceneComponentViewMut<'a>(pub(crate) SceneComponentViewMutInner<'a>);

macro_rules! impl_read_property {
    ($type:ty, $read:ident) => {
        pub fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            match &self.0 {
                SceneComponentViewRefInner::Static {
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
                SceneComponentViewRefInner::Dynamic {} => None,
                SceneComponentViewRefInner::None => None,
            }
        }
    };
}

macro_rules! impl_read_write_property {
    ($type:ty, $read:ident, $write:ident) => {
        pub fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            match &self.0 {
                SceneComponentViewMutInner::Static {
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
                SceneComponentViewMutInner::Dynamic {} => None,
                SceneComponentViewMutInner::None => None,
            }
        }
        pub fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {
            match &mut self.0 {
                SceneComponentViewMutInner::Static {
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
                SceneComponentViewMutInner::Dynamic {} => {}
                SceneComponentViewMutInner::None => {}
            }
        }
    };
}

impl<'a> SceneComponentViewRef<'a> {
    pub(crate) fn none() -> Self {
        Self(SceneComponentViewRefInner::None)
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

impl<'a> SceneComponentViewMut<'a> {
    pub(crate) fn none() -> Self {
        Self(SceneComponentViewMutInner::None)
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
