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
        RangeInput { value: 0.0, range }
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