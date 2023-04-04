use std::any::Any;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};

use crate::registry::component::Component;

pub(crate) struct Singleton<C: Component> {
    pub(crate) component: RefCell<C>,
}

impl<C: Component> Singleton<C> {
    pub(crate) fn new(component: C) -> Self {
        Self { component: RefCell::new(component) }
    }
}

pub(crate) trait AnySingleton {
    fn as_any(&self) -> &dyn Any;
}

impl<C: Component> AnySingleton for Singleton<C> {
    fn as_any(&self) -> &dyn Any { self }
}

pub struct SingletonRef<'a, C: Component> {
    pub(crate) component: Ref<'a, C>,
}

impl<C: Component> Deref for SingletonRef<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.component
    }
}

pub struct SingletonMut<'a, C: Component> {
    pub(crate) component: RefMut<'a, C>,
}

impl<C: Component> Deref for SingletonMut<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.component
    }
}

impl<C: Component> DerefMut for SingletonMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}