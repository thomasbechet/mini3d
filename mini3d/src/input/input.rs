/// Store input state
pub struct ButtonInput {
    /// The button is pressed or released
    pub pressed: bool,
    /// Keep the previous state to detect just pressed and released
    pub(crate) was_pressed: bool,
}

impl ButtonInput {

    pub fn new() -> Self {
        ButtonInput { pressed: false, was_pressed: false }
    }

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

pub enum RangeType {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    Infinite,
}

pub struct RangeInput {
    pub value: f32,
    pub range: RangeType,
}

impl RangeInput {
    
    pub fn new(range: RangeType) -> Self {
        RangeInput { value: 0.0, range: range }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = match self.range {
            RangeType::Clamped { min, max } => {
                value.max(min).min(max)
            },
            RangeType::Normalized { norm } => {
                value / norm
            },
            RangeType::ClampedNormalized { min, max, norm } => {
                value.max(min).min(max) / norm
            },
            RangeType::Infinite => {
                value
            },
        }
    }
}

pub type InputName = &'static str;