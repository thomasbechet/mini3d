use crate::{entity::EntityTable, view::SystemView};

pub struct World<'a> {
    pub(crate) entities: &'a mut EntityTable,
}

impl<'a> World<'a> {
    pub fn view<V: SystemView>(&self) -> Option<V> {
        None
    }
}
