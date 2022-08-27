/// Store input state
#[derive(Default)]
pub struct ButtonInput {
    /// The button is pressed or released
    pub pressed: bool,
    /// Keep the previous state to detect just pressed and released
    pub(crate) was_pressed: bool,
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