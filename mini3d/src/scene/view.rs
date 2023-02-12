use std::ops::{Index, IndexMut};

use crate::registry::component::Component;

use super::{container::ComponentContainer, entity::Entity};

pub struct ComponentView<'a, C: Component> {
    components: &'a [C],
    entities: &'a [Entity],
    indices: &'a [usize],
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
        let index = self.indices[entity.index() as usize] as usize;
        if self.entities[index] != entity { return None }
        Some(&self.components[index])
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
    indices: &'a [usize],
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
        let index = self.indices[entity.index() as usize] as usize;
        if self.entities[index] != entity { return None }
        Some(&self.components[index])
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        let index = self.indices[entity.index() as usize] as usize;
        if self.entities[index] != entity { return None }
        Some(&mut self.components[index])
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