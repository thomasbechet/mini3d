use anyhow::{Result, anyhow};
use slotmap::{new_key_type, SlotMap, Key};

use crate::{event::input::{InputEvent, TextEvent}, program::ProgramId, app::App, graphics::{SCREEN_WIDTH, SCREEN_HEIGHT}};

use self::{axis::{AxisInput, AxisKind, AxisInputId}, button::{ButtonInput, ButtonState, ButtonInputId}};

pub mod control_layout;
pub mod axis;
pub mod button;
pub mod macros;

new_key_type! { pub struct InputGroupId; }

pub struct InputGroup {
    pub name: String,
    pub id: InputGroupId,
    pub owner: ProgramId,
}

pub struct InputManager {
    text: String,
    buttons: SlotMap<ButtonInputId, ButtonInput>,
    axis: SlotMap<AxisInputId, AxisInput>,
    groups: SlotMap<InputGroupId, InputGroup>,
    default_group: InputGroupId,
    pub(crate) reload_bindings: bool,
}

impl Default for InputManager {
    fn default() -> Self {
        // Default manager
        let mut manager = Self {
            text: Default::default(),
            buttons: Default::default(),
            axis: Default::default(),
            groups: Default::default(),
            default_group: InputGroupId::null(),
            reload_bindings: false,
        };
        // Register default input group
        manager.default_group = manager.register_group("default", ProgramId::null())
            .expect("Failed to register default group");
        // Register default buttons and axis
        manager.register_axis(AxisInput::CURSOR_X, manager.default_group, AxisKind::Clamped { min: 0.0, max: SCREEN_WIDTH as f32 }).unwrap();
        manager.register_axis(AxisInput::CURSOR_Y, manager.default_group, AxisKind::Clamped { min: 0.0, max: SCREEN_HEIGHT as f32 }).unwrap();
        manager.register_axis(AxisInput::MOTION_X, manager.default_group, AxisKind::Infinite).unwrap();
        manager.register_axis(AxisInput::MOTION_Y, manager.default_group, AxisKind::Infinite).unwrap();
        manager.register_button(ButtonInput::UP, manager.default_group).unwrap();
        manager.register_button(ButtonInput::DOWN, manager.default_group).unwrap();
        manager.register_button(ButtonInput::LEFT, manager.default_group).unwrap();
        manager.register_button(ButtonInput::RIGHT, manager.default_group).unwrap();
        // Return the manager
        manager
    }
}

impl InputManager {

    /// Reset button states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous button state
        for (_, button) in self.buttons.iter_mut() {
            button.was_pressed = button.pressed;
        }

        // Reset text for current frame
        self.text.clear();
    }

    /// Process input events
    pub(crate) fn dispatch_event(&mut self, event: &InputEvent) {

        match event {
            InputEvent::Button(button) => {
                if let Some(action) = self.buttons.get_mut(button.id) {
                    match button.state {
                        ButtonState::Pressed => {
                            action.pressed = true;
                        },
                        ButtonState::Released => {
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
            self.reload_bindings = true;
            Ok(new_group)
        }
    }

    pub fn find_button(&self, name: &str) -> Option<&ButtonInput> {
        self.buttons.iter()
            .find(|(_, e)| e.name.as_str() == name)
            .map(|(_, e)| e)
    }

    pub fn find_axis(&self, name: &str) -> Option<&AxisInput> {
        self.axis.iter()
            .find(|(_, e)| e.name.as_str() == name)
            .map(|(_, e)| e)
    }

    pub fn register_button(&mut self, name: &str, group: InputGroupId) -> Result<ButtonInputId> {
        if self.find_axis(name).is_some() {
            Err(anyhow!("Button input name '{}' already exists", name))
        } else {
            let id = self.buttons.insert(ButtonInput { 
                pressed: false, 
                was_pressed: false, 
                name: name.to_string(),
                group,
                id: ButtonInputId::null(),
            });
            self.buttons.get_mut(id).unwrap().id = id;
            self.reload_bindings = true;
            Ok(id)
        }
    }

    pub fn register_axis(&mut self, name: &str, group: InputGroupId, axis: AxisKind) -> Result<AxisInputId> {
        if self.find_axis(name).is_some() {
            Err(anyhow!("Axis input name '{}' already exists", name))
        } else {
            let id = self.axis.insert(AxisInput { 
                name: name.to_string(),
                id: AxisInputId::null(),
                value: 0.0,
                group,
                kind: axis,
            });
            self.axis.get_mut(id).unwrap().id = id;
            self.reload_bindings = true;
            Ok(id)
        }
    }

    pub fn group(&self, id: InputGroupId) -> Option<&InputGroup> {
        self.groups.get(id)
    }

    pub fn button(&self, id: ButtonInputId) -> Option<&ButtonInput> {
        self.buttons.get(id)
    }

    pub fn axis(&self, id: AxisInputId) -> Option<&AxisInput> {
        self.axis.get(id)
    }

    pub fn iter_buttons(&self) -> impl Iterator<Item = &ButtonInput> {
        self.buttons.values()
    } 

    pub fn iter_axis(&self) -> impl Iterator<Item = &AxisInput> {
        self.axis.values()
    }
}

pub struct InputDatabase;

impl InputDatabase {
    pub fn iter_buttons(app: &App) -> impl Iterator<Item = &ButtonInput> {
        app.input_manager.iter_buttons()
    }
    pub fn iter_axis(app: &App) -> impl Iterator<Item = &AxisInput> {
        app.input_manager.iter_axis()
    }
    pub fn group(app: &App, id: InputGroupId) -> Option<&InputGroup> {
        app.input_manager.group(id)
    }
}