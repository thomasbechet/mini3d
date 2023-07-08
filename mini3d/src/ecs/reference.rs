use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut},
};

use crate::registry::component::Component;

pub struct StaticComponentRef<'a, C: Component> {
    pub(crate) components: Ref<'a, Vec<C>>,
    pub(crate) index: usize,
}

impl<C: Component> Deref for StaticComponentRef<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.components[self.index]
    }
}

pub struct StaticComponentMut<'a, C: Component> {
    pub(crate) components: RefMut<'a, Vec<C>>,
    pub(crate) index: usize,
}

impl<C: Component> Deref for StaticComponentMut<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.components[self.index]
    }
}

impl<C: Component> DerefMut for StaticComponentMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components[self.index]
    }
}
