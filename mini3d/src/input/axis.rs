use slotmap::new_key_type;

use super::InputGroupId;

new_key_type! { pub struct AxisInputId; }

pub enum AxisKind {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    Infinite,
}

pub struct AxisInput {
    pub value: f32,
    pub kind: AxisKind,
    pub name: String,
    pub group: InputGroupId,
    pub id: AxisInputId,
}

impl AxisInput {

    pub const CURSOR_X: &'static str = "cursor_x";
    pub const CURSOR_Y: &'static str = "cursor_y";
    pub const MOTION_X: &'static str = "motion_x";
    pub const MOTION_Y: &'static str = "motion_y";
    
    pub fn set_value(&mut self, value: f32) {
        self.value = match self.kind {
            AxisKind::Clamped { min, max } => {
                value.max(min).min(max)
            },
            AxisKind::Normalized { norm } => {
                value / norm
            },
            AxisKind::ClampedNormalized { min, max, norm } => {
                value.max(min).min(max) / norm
            },
            AxisKind::Infinite => {
                value
            },
        }
    }
}