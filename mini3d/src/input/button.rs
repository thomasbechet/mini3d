use slotmap::new_key_type;

use super::InputGroupId;

new_key_type! { pub struct ButtonInputId; }

pub struct ButtonInput {
    pub(crate) pressed: bool,
    pub(crate) was_pressed: bool,
    pub name: String,
    pub group: InputGroupId,
    pub id: ButtonInputId,
}

#[derive(Copy, Clone)]
pub enum ButtonState {
    Pressed,
    Released,
}

impl ButtonInput {
    
    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.pressed
    }

    pub fn is_just_pressed(&self) -> bool {
        self.pressed && !self.was_pressed
    }

    pub fn is_just_released(&self) -> bool {
        !self.pressed && self.was_pressed
    }
}