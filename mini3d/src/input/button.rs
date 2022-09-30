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

    pub const UP: &'static str = "up"; 
    pub const DOWN: &'static str = "down"; 
    pub const LEFT: &'static str = "left"; 
    pub const RIGHT: &'static str = "right";
    
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