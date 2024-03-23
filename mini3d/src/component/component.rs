use crate::{self as mini3d, api::API};
use mini3d_db::{database::ComponentHandle, entity::Entity};
use mini3d_derive::component;

#[component]
pub struct Component {
    handle: u32,
}

impl Component {
    pub fn handle_from_entity(&self, api: &API, e: Entity) -> Option<ComponentHandle> {
        let handle = api.read(e, self.handle)?;
        Some(ComponentHandle::from_raw(handle))
    }
}
