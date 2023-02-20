use std::ops::{Index, IndexMut, Deref};

use crate::registry::component::Component;

use super::{entity::Entity, container::ComponentContainer, sparse::PagedVector};

pub struct ComponentView<'a, C: Component> {
    components: &'a [C],
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

impl<'a, C: Component> ComponentView<'a, C> {

    pub(crate) fn new(container: &'a ComponentContainer<C>) -> Self {
        Self {
            components: &container.components.borrow(),
            entities: &container.entities,
            indices: &container.indices,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.components.iter()
    }

    pub fn get(&self, entity: Entity) -> Option<&C> {
        self.indices.get(entity.index()).copied().and_then(|index| {
            if self.entities[index] == entity {
                Some(&self.components[index])
            } else {
                None
            }
        })      
    }
}

impl<'a, C: Component> Index<Entity> for ComponentView<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IntoIterator for &ComponentView<'a, C> {
    type Item = &'a C;
    type IntoIter = std::slice::Iter<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.iter()
    }
}

pub struct ComponentViewMut<'a, C: Component> {
    components: &'a mut [C],
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

impl<'a, C: Component> ComponentViewMut<'a, C> {

    pub(crate) fn new(container: &'a ComponentContainer<C>) -> Self {
        Self {
            components: container.components.borrow_mut().as_mut_slice(),
            entities: &container.entities,
            indices: &container.indices,
        }
    }

    pub fn iter(&mut self) -> impl Iterator<Item = &mut C> {
        self.components.iter_mut()
    }

    pub fn get(&mut self, entity: Entity) -> Option<&C> {
        self.indices.get(entity.index()).copied().and_then(|index| {
            if self.entities[index] == entity {
                Some(&self.components[index])
            } else {
                None
            }
        })
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

impl<'a, C: Component> IntoIterator for &ComponentViewMut<'a, C> {
    type Item = &'a mut C;
    type IntoIter = std::slice::IterMut<'a, C>;

    fn into_iter(self) -> Self::IntoIter {
        self.components.iter_mut()
    }
}

impl<'a, C: Component> Deref for ComponentViewMut<'a, C> {
    type Target = ComponentView<'a, C>;

    fn deref(&self) -> &Self::Target {
        &ComponentView::<'a, C> { components: &self.components, entities: self.entities, indices: self.indices }
    }
}