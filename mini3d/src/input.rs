use anyhow::{Result, anyhow};
use slotmap::{new_key_type, SlotMap, Key};

use crate::{event::input::{InputEvent, TextEvent}, program::ProgramId, app::App};

use self::{axis::{AxisInput, AxisInputId, AxisDescriptor}, action::{ActionInput, ActionState, ActionInputId, ActionDescriptor}};

pub mod control_layout;
pub mod axis;
pub mod action;

new_key_type! { pub struct InputGroupId; }

pub struct InputGroup {
    pub name: String,
    pub id: InputGroupId,
    pub owner: ProgramId,
}

pub struct InputManager {
    text: String,
    actions: SlotMap<ActionInputId, ActionInput>,
    axis: SlotMap<AxisInputId, AxisInput>,
    groups: SlotMap<InputGroupId, InputGroup>,
    pub(crate) reload_input_mapping: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            text: Default::default(),
            actions: Default::default(),
            axis: Default::default(),
            groups: Default::default(),
            reload_input_mapping: false,
        }
    }
}

impl InputManager {

    /// Reset action states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous action state
        for (_, action) in self.actions.iter_mut() {
            action.was_pressed = action.pressed;
        }

        // Reset text for current frame
        self.text.clear();
    }

    /// Process input events
    pub(crate) fn dispatch_event(&mut self, event: &InputEvent) {

        match event {
            InputEvent::Action(event) => {
                if let Some(action) = self.actions.get_mut(event.id) {
                    match event.state {
                        ActionState::Pressed => {
                            action.pressed = true;
                        },
                        ActionState::Released => {
                            action.pressed = false;
                        },
                    }
                }
            },
            InputEvent::Axis(event) => {
                if let Some(axis) = self.axis.get_mut(event.id) {
                    axis.set_value(event.value);
                }
            },
            InputEvent::Text(text_event) => {
                match text_event {
                    TextEvent::Character(char) => {
                        self.text.push(*char);
                    },
                    TextEvent::String(string) => {
                        self.text += string;
                    },
                }
            },
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn find_group(&self, name: &str) -> Option<&InputGroup> {
        self.groups.iter()
            .find(|(_, e)| e.name.as_str() == name)
            .and_then(|(_, group)| Some(group))
    }

    pub fn register_group(&mut self, name: &str, owner: ProgramId) -> Result<InputGroupId> {
        if self.find_group(&name).is_some() {
            Err(anyhow!("Input group '{}' already exists", name))
        } else {
            let new_group = self.groups.insert(InputGroup { 
                name: name.to_string(), 
                id: InputGroupId::null(), 
                owner,
            });
            self.groups.get_mut(new_group).unwrap().id = new_group;
            self.reload_input_mapping = true;
            Ok(new_group)
        }
    }

    pub fn find_action(&self, group: InputGroupId, name: &str) -> Option<&ActionInput> {
        self.actions.iter()
            .find(|(_, e)| e.descriptor.name.as_str() == name && e.group == group)
            .map(|(_, e)| e)
    }

    pub fn find_axis(&self, group: InputGroupId, name: &str) -> Option<&AxisInput> {
        self.axis.iter()
            .find(|(_, e)| e.descriptor.name.as_str() == name && e.group == group)
            .map(|(_, e)| e)
    }

    pub fn register_action(&mut self, group: InputGroupId, descriptor: ActionDescriptor) -> Result<ActionInputId> {
        if self.find_axis(group, &descriptor.name).is_some() {
            Err(anyhow!("Action input name '{}' already exists", descriptor.name))
        } else {
            let id = self.actions.insert(ActionInput { 
                descriptor,
                pressed: false, 
                was_pressed: false, 
                group,
                id: ActionInputId::null(),
            });
            self.actions.get_mut(id).unwrap().id = id;
            self.reload_input_mapping = true;
            Ok(id)
        }
    }

    pub fn register_axis(&mut self, group: InputGroupId, descriptor: AxisDescriptor) -> Result<AxisInputId> {
        if self.find_axis(group, &descriptor.name).is_some() {
            Err(anyhow!("Axis input name '{}' already exists", descriptor.name))
        } else {
            let id = self.axis.insert(AxisInput {
                descriptor,
                id: AxisInputId::null(),
                value: 0.0,
                group,
            });
            self.axis.get_mut(id).unwrap().id = id;
            self.reload_input_mapping = true;
            Ok(id)
        }
    }

    pub fn group(&self, id: InputGroupId) -> Option<&InputGroup> {
        self.groups.get(id)
    }

    pub fn action(&self, id: ActionInputId) -> Option<&ActionInput> {
        self.actions.get(id)
    }

    pub fn axis(&self, id: AxisInputId) -> Option<&AxisInput> {
        self.axis.get(id)
    }

    pub fn iter_actions(&self) -> impl Iterator<Item = &ActionInput> {
        self.actions.values()
    } 

    pub fn iter_axis(&self) -> impl Iterator<Item = &AxisInput> {
        self.axis.values()
    }
}

pub struct InputDatabase;

impl InputDatabase {

    pub fn iter_actions(app: &App) -> impl Iterator<Item = ActionInputId> + '_ {
        app.input_manager.actions.keys()
    }
    pub fn iter_axis(app: &App) -> impl Iterator<Item = AxisInputId> + '_ {
        app.input_manager.axis.keys()
    }
    pub fn iter_groups(app: &App) -> impl Iterator<Item = InputGroupId> + '_ {
        app.input_manager.groups.keys()
    }
    pub fn action(app: &App, id: ActionInputId) -> Option<&ActionInput> {
        app.input_manager.action(id)
    }
    pub fn axis(app: &App, id: AxisInputId) -> Option<&AxisInput> {
        app.input_manager.axis(id)
    }
    pub fn group(app: &App, id: InputGroupId) -> Option<&InputGroup> {
        app.input_manager.group(id)
    }
}