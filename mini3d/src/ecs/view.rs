use std::{
    cell::{Ref, RefMut},
    ops::{Deref, Index, IndexMut},
};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::{registry::component::Component, script::reflection::PropertyId, uid::UID};

use super::{container::StaticComponentContainer, entity::Entity, sparse::PagedVector};

pub trait StaticComponentView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

struct StaticComponentViewRefData<'a, C: Component> {
    components: Ref<'a, Vec<C>>,
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
    components: RefMut<'a, Vec<C>>,
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

#[allow(unused)]
pub trait ReadPropertyView {
    fn read_bool(&self, entity: Entity, id: PropertyId) -> Option<bool> {
        None
    }
    fn read_u8(&self, entity: Entity, id: PropertyId) -> Option<u8> {
        None
    }
    fn read_i32(&self, entity: Entity, id: PropertyId) -> Option<i32> {
        None
    }
    fn read_u32(&self, entity: Entity, id: PropertyId) -> Option<u32> {
        None
    }
    fn read_f32(&self, entity: Entity, id: PropertyId) -> Option<f32> {
        None
    }
    fn read_f64(&self, entity: Entity, id: PropertyId) -> Option<f64> {
        None
    }
    fn read_vec2(&self, entity: Entity, id: PropertyId) -> Option<Vec2> {
        None
    }
    fn read_ivec2(&self, entity: Entity, id: PropertyId) -> Option<IVec2> {
        None
    }
    fn read_vec3(&self, entity: Entity, id: PropertyId) -> Option<Vec3> {
        None
    }
    fn read_ivec3(&self, entity: Entity, id: PropertyId) -> Option<IVec3> {
        None
    }
    fn read_vec4(&self, entity: Entity, id: PropertyId) -> Option<Vec4> {
        None
    }
    fn read_ivec4(&self, entity: Entity, id: PropertyId) -> Option<IVec4> {
        None
    }
    fn read_mat4(&self, entity: Entity, id: PropertyId) -> Option<Mat4> {
        None
    }
    fn read_quat(&self, entity: Entity, id: PropertyId) -> Option<Quat> {
        None
    }
    fn read_entity(&self, entity: Entity, id: PropertyId) -> Option<Entity> {
        None
    }
    fn read_uid(&self, entity: Entity, id: PropertyId) -> Option<UID> {
        None
    }
}

#[allow(unused)]
pub trait ReadWritePropertyView: ReadPropertyView {
    fn as_read_view(&self) -> &dyn ReadPropertyView;
    fn write_bool(&mut self, entity: Entity, id: PropertyId, value: bool) {}
    fn write_u8(&mut self, entity: Entity, id: PropertyId, value: u8) {}
    fn write_i32(&mut self, entity: Entity, id: PropertyId, value: i32) {}
    fn write_u32(&mut self, entity: Entity, id: PropertyId, value: u32) {}
    fn write_f32(&mut self, entity: Entity, id: PropertyId, value: f32) {}
    fn write_f64(&mut self, entity: Entity, id: PropertyId, value: f64) {}
    fn write_vec2(&mut self, entity: Entity, id: PropertyId, value: Vec2) {}
    fn write_ivec2(&mut self, entity: Entity, id: PropertyId, value: IVec2) {}
    fn write_vec3(&mut self, entity: Entity, id: PropertyId, value: Vec3) {}
    fn write_ivec3(&mut self, entity: Entity, id: PropertyId, value: IVec3) {}
    fn write_vec4(&mut self, entity: Entity, id: PropertyId, value: Vec4) {}
    fn write_ivec4(&mut self, entity: Entity, id: PropertyId, value: IVec4) {}
    fn write_mat4(&mut self, entity: Entity, id: PropertyId, value: Mat4) {}
    fn write_quat(&mut self, entity: Entity, id: PropertyId, value: Quat) {}
    fn write_entity(&mut self, entity: Entity, id: PropertyId, value: Entity) {}
    fn write_uid(&mut self, entity: Entity, id: PropertyId, value: UID) {}
}

macro_rules! read_property {
    ($type:ty, $read:ident) => {
        fn $read(&self, entity: Entity, id: PropertyId) -> Option<$type> {
            self.get(entity).and_then(|c| c.$read(id))
        }
    };
}

macro_rules! write_property {
    ($type:ty, $write:ident) => {
        fn $write(&mut self, entity: Entity, id: PropertyId, value: $type) {
            self.get_mut(entity).map(|c| c.$write(id, value));
        }
    };
}

impl<'a, C: Component> ReadPropertyView for StaticComponentViewRef<'a, C> {
    read_property!(bool, read_bool);
    read_property!(u8, read_u8);
    read_property!(i32, read_i32);
    read_property!(u32, read_u32);
    read_property!(f32, read_f32);
    read_property!(f64, read_f64);
    read_property!(Vec2, read_vec2);
    read_property!(IVec2, read_ivec2);
    read_property!(Vec3, read_vec3);
    read_property!(IVec3, read_ivec3);
    read_property!(Vec4, read_vec4);
    read_property!(IVec4, read_ivec4);
    read_property!(Mat4, read_mat4);
    read_property!(Quat, read_quat);
    read_property!(Entity, read_entity);
    read_property!(UID, read_uid);
}

impl<'a, C: Component> ReadPropertyView for StaticComponentViewMut<'a, C> {
    read_property!(bool, read_bool);
    read_property!(u8, read_u8);
    read_property!(i32, read_i32);
    read_property!(u32, read_u32);
    read_property!(f32, read_f32);
    read_property!(f64, read_f64);
    read_property!(Vec2, read_vec2);
    read_property!(IVec2, read_ivec2);
    read_property!(Vec3, read_vec3);
    read_property!(IVec3, read_ivec3);
    read_property!(Vec4, read_vec4);
    read_property!(IVec4, read_ivec4);
    read_property!(Mat4, read_mat4);
    read_property!(Quat, read_quat);
    read_property!(Entity, read_entity);
    read_property!(UID, read_uid);
}

impl<'a, C: Component> ReadWritePropertyView for StaticComponentViewMut<'a, C> {
    fn as_read_view(&self) -> &dyn ReadPropertyView {
        self
    }
    write_property!(bool, write_bool);
    write_property!(u8, write_u8);
    write_property!(i32, write_i32);
    write_property!(u32, write_u32);
    write_property!(f32, write_f32);
    write_property!(f64, write_f64);
    write_property!(Vec2, write_vec2);
    write_property!(IVec2, write_ivec2);
    write_property!(Vec3, write_vec3);
    write_property!(IVec3, write_ivec3);
    write_property!(Vec4, write_vec4);
    write_property!(IVec4, write_ivec4);
    write_property!(Mat4, write_mat4);
    write_property!(Quat, write_quat);
    write_property!(Entity, write_entity);
    write_property!(UID, write_uid);
}

pub struct AnyComponentViewRef<'a> {
    pub(crate) view: Box<dyn ReadPropertyView + 'a>,
}

impl<'a> AnyComponentViewRef<'a> {
    pub(crate) fn none() -> Self {
        struct AnyComponentViewRefNone {}
        impl ReadPropertyView for AnyComponentViewRefNone {}
        Self {
            view: Box::new(AnyComponentViewRefNone {}),
        }
    }
}

impl<'a> Deref for AnyComponentViewRef<'a> {
    type Target = dyn ReadPropertyView + 'a;

    fn deref(&self) -> &Self::Target {
        &*self.view
    }
}

pub struct AnyComponentViewMut<'a> {
    pub(crate) view: Box<dyn ReadWritePropertyView + 'a>,
}

impl<'a> AnyComponentViewMut<'a> {
    pub(crate) fn none() -> Self {
        struct AnyComponentViewMutNone {}
        impl ReadWritePropertyView for AnyComponentViewMutNone {
            fn as_read_view(&self) -> &dyn ReadPropertyView {
                self
            }
        }
        impl ReadPropertyView for AnyComponentViewMutNone {}
        Self {
            view: Box::new(AnyComponentViewMutNone {}),
        }
    }
}
