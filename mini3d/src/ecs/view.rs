use std::{ops::{Index, IndexMut}, cell::{Ref, RefMut}};

use crate::registry::component::Component;

use super::{entity::Entity, container::ComponentContainer, sparse::PagedVector};

pub trait ComponentView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

pub struct ComponentViewRef<'a, C: Component> {
    components: Ref<'a, Vec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

impl<'a, C: Component> ComponentViewRef<'a, C> {

    pub(crate) fn new(container: &'a ComponentContainer<C>) -> Self {
        Self {
            components: container.components.borrow(),
            entities: &container.entities,
            indices: &container.indices,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.components.iter()
    }
}

impl<'a, C: Component> ComponentView<C> for ComponentViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.indices.get(entity.index()).copied().and_then(|index| {
            if self.entities[index] == entity {
                Some(&self.components[index])
            } else {
                None
            }
        })
    }
}

impl<'a, C: Component> Index<Entity> for ComponentViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct ComponentViewMut<'a, C: Component> {
    components: RefMut<'a, Vec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

impl<'a, C: Component> ComponentViewMut<'a, C> {

    pub(crate) fn new(container: &'a ComponentContainer<C>) -> Self {
        Self {
            components: container.components.borrow_mut(),
            entities: &container.entities,
            indices: &container.indices,
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &mut C> {
        self.components.iter_mut()
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.indices.get(entity.index()).copied().and_then(|index| {
            if self.entities[index] == entity {
                Some(&mut self.components[index])
            } else {
                None
            }
        })
    }
}

impl<'a, C: Component> ComponentView<C> for ComponentViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.indices.get(entity.index()).copied().and_then(|index| {
            if self.entities[index] == entity {
                Some(&self.components[index])
            } else {
                None
            }
        })
    }
}

impl<'a, C: Component> Index<Entity> for ComponentViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IndexMut<Entity> for ComponentViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}