use std::{ops::{Index, IndexMut}, cell::{Ref, RefMut}};

use super::{entity::Entity, container::ComponentContainer, sparse::PagedVector, component::Component};

pub trait ComponentView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

struct ComponentViewRefData<'a, C: Component> {
    components: Ref<'a, Vec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

pub struct ComponentViewRef<'a, C: Component> {
    view: Option<ComponentViewRefData<'a, C>>,
}

impl<'a, C: Component> ComponentViewRef<'a, C> {

    pub(crate) fn new(container: &'a ComponentContainer<C>) -> Self {
        Self {
            view: Some(ComponentViewRefData {
                components: container.components.borrow(),
                entities: &container.entities,
                indices: &container.indices,
            })
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

impl<'a, C: Component> ComponentView<C> for ComponentViewRef<'a, C> {
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

impl<'a, C: Component> Index<Entity> for ComponentViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

struct ComponentViewMutData<'a, C: Component> {
    components: RefMut<'a, Vec<C>>,
    entities: &'a [Entity],
    indices: &'a PagedVector<usize>,
}

pub struct ComponentViewMut<'a, C: Component> {
    view: Option<ComponentViewMutData<'a, C>>
}

impl<'a, C: Component> ComponentViewMut<'a, C> {

    pub(crate) fn new(container: &'a ComponentContainer<C>) -> Self {
        Self {
            view: Some(ComponentViewMutData {
                components: container.components.borrow_mut(),
                entities: &container.entities,
                indices: &container.indices,
            })
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

impl<'a, C: Component> ComponentView<C> for ComponentViewMut<'a, C> {
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