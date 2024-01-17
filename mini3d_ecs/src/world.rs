use crate::{container::ContainerTable, view::SystemView};

pub struct World<'a> {
    pub(crate) containers: &'a mut ContainerTable,
}

impl<'a> World<'a> {
    pub fn view<V: SystemView>(&self) -> Option<V> {
        None
    }
}
