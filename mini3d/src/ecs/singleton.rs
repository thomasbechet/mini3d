use std::any::Any;
use std::cell::{RefCell, Ref, RefMut};
use std::ops::{Deref, DerefMut};

use crate::registry::component::Component;
use crate::serialize::{Serialize, Encoder, EncoderError, Decoder, DecoderError};

pub(crate) struct ComponentSingleton<C: Component> {
    pub(crate) component: RefCell<C>,
}

impl<C: Component> ComponentSingleton<C> {
    
    pub(crate) fn new(component: C) -> Self {
        Self { component: RefCell::new(component) }
    }

    pub(crate) fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        let component = self.component.borrow();
        C::Header::default().serialize(encoder)?;
        component.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        let header = <C as Serialize>::Header::deserialize(decoder, &Default::default())?;
        self.component.replace(C::deserialize(decoder, &header)?);
        Ok(())
    }
}

pub(crate) trait AnyComponentSingleton {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
}

impl<C: Component> AnyComponentSingleton for ComponentSingleton<C> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }
    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        self.serialize(&mut encoder)
    }
    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        self.deserialize(&mut decoder)
    }
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