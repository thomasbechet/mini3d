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

    pub const MOVE_UP: &'static str = "move_up"; 
    pub const MOVE_DOWN: &'static str = "move_down"; 
    pub const MOVE_LEFT: &'static str = "move_left"; 
    pub const MOVE_RIGHT: &'static str = "move_right";
    
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