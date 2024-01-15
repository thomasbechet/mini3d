use crate::view::SystemView;

pub struct World<'a> {}

impl<'a> World<'a> {
    pub fn view<V: SystemView>(&self) -> Option<V> {
        None
    }
}
