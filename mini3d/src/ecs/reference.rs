use std::ops::{Deref, DerefMut};

use crate::script::reflection::{ReadProperty, ReadWriteProperty};

pub struct AnyComponentRef<'a> {
    pub(crate) component: Box<dyn ReadProperty + 'a>,
}

impl<'a> Deref for AnyComponentRef<'a> {
    type Target = dyn ReadProperty + 'a;

    fn deref(&self) -> &Self::Target {
        &*self.component
    }
}

pub struct AnyComponentMut<'a> {
    pub(crate) component: Box<dyn ReadWriteProperty + 'a>,
}

impl<'a> Deref for AnyComponentMut<'a> {
    type Target = dyn ReadWriteProperty + 'a;

    fn deref(&self) -> &Self::Target {
        &*self.component
    }
}
