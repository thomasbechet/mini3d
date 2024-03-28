use mini3d_db::entity::Entity;

use crate::{
    api::API,
    event::{Event, UserEventHandle},
    handle_component,
};

handle_component!(UserEvent, UserEventHandle, "user_event");

impl UserEvent {
    pub fn register(api: &mut API) {
        let handle = api.register_component_handle(Self::NAME);
        api.register_system(
            "delete_user_event",
            Event::ComponentRemoved(handle),
            Default::default(),
            delete_user_event,
        );
    }

    pub fn add(&self, api: &mut API, e: Entity, name: &str) {
        api.add_default(e, self);
        let handle = api.state.events.add_user_event(api.scheduler, name, e);
        api.write_handle(e, self, handle);
        api.state.rebuild_scheduler = true;
    }
}

fn delete_user_event(api: &mut API, user_event: &UserEvent) {
    if let Some(handle) = user_event.handle(api, api.event_entity()) {
        api.state.events.remove_user_event(api.scheduler, handle);
    }
}
