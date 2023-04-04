use std::{cell::{Ref, RefMut}, ops::{Deref, DerefMut}};

use crate::registry::component::Component;

pub struct ComponentRef<'a, C: Component> {
    pub(crate) components: Ref<'a, Vec<C>>,
    pub(crate) index: usize,
}

impl<C: Component> Deref for ComponentRef<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.components[self.index]
    }
}

pub struct ComponentMut<'a, C: Component> {
    pub(crate) components: RefMut<'a, Vec<C>>,
    pub(crate) index: usize,
}

impl<C: Component> Deref for ComponentMut<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.components[self.index]
    }
}

impl<C: Component> DerefMut for ComponentMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components[self.index]
    }
}