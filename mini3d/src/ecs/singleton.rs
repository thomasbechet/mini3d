use std::any::Any;
use std::cell::{Ref, RefCell, RefMut};
use std::ops::{Deref, DerefMut};

use crate::registry::component::Component;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

pub(crate) struct StaticSceneSingleton<C: Component> {
    pub(crate) component: RefCell<C>,
}

impl<C: Component> StaticSceneSingleton<C> {
    pub(crate) fn new(component: C) -> Self {
        Self {
            component: RefCell::new(component),
        }
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

pub(crate) trait AnySceneSingleton {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
}

impl<C: Component> AnySceneSingleton for StaticSceneSingleton<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }
    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        self.serialize(&mut encoder)
    }
    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        self.deserialize(&mut decoder)
    }
}

pub struct StaticSingletonRef<'a, C: Component> {
    pub(crate) component: Ref<'a, C>,
}

impl<C: Component> Deref for StaticSingletonRef<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.component
    }
}

pub struct StaticSingletonMut<'a, C: Component> {
    pub(crate) component: RefMut<'a, C>,
}

impl<C: Component> Deref for StaticSingletonMut<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.component
    }
}

impl<C: Component> DerefMut for StaticSingletonMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.component
    }
}
