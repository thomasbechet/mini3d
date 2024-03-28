use mini3d_db::entity::Entity;
use mini3d_input::{action::InputActionHandle, axis::{InputAxisHandle, InputAxisRange}, text::InputTextHandle};

use crate::{api::API, event::Event, handle_component};

handle_component!(InputAction, InputActionHandle, "input_action");
handle_component!(InputAxis, InputAxisHandle, "input_axis");
handle_component!(InputText, InputTextHandle, "input_text");

impl InputAction {
    pub fn register(api: &mut API) {
        let handle = api.register_component_handle(Self::NAME);
        api.register_system(
            "delete_input_action",
            Event::ComponentRemoved(handle),
            Default::default(),
            delete_input_action,
        );
    }

    pub fn add(&self, api: &mut API, e: Entity, name: &str) {
        let handle = api.input.create_action(name).unwrap();
        api.add_default(e, self);
        api.write_handle(e, self, handle);
    }
}

fn delete_input_action(api: &mut API, input_action: &InputAction) {
    if let Some(handle) = input_action.handle(api, api.event_entity()) {
        api.input.delete_action(handle).unwrap();
    }
}

impl InputAxis {
    pub fn register(api: &mut API) {
        let handle = api.register_component_handle(Self::NAME);
        api.register_system(
            "delete_input_axis",
            Event::ComponentRemoved(handle),
            Default::default(),
            delete_input_axis,
        );
    }

    pub fn add(&self, api: &mut API, e: Entity, name: &str, range: InputAxisRange) {
        let handle = api.input.create_axis(name, range).unwrap();
        api.add_default(e, self);
        api.write_handle(e, self, handle);
    }
}

fn delete_input_axis(api: &mut API, input_axis: &InputAxis) {
    if let Some(handle) = input_axis.handle(api, api.event_entity()) {
        api.input.delete_axis(handle).unwrap();
    }
}

impl InputText {
    pub fn register(api: &mut API) {
        let handle = api.register_component_handle(Self::NAME);
        api.register_system(
            "delete_input_text",
            Event::ComponentRemoved(handle),
            Default::default(),
            delete_input_text,
        );
    }

    pub fn add(&self, api: &mut API, e: Entity, name: &str) {
        // let handle = api.input.create_text(name).unwrap();
        api.add_default(e, self);
        // api.write_handle(e, self, handle);
    }
}

fn delete_input_text(api: &mut API, input_text: &InputText) {
    // if let Some(handle) = input_axis.handle(api, api.event_entity()) {
    //     api.input.delete_axis(handle).unwrap();
    // }
}
