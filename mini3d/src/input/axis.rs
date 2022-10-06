use slotmap::new_key_type;

use super::InputGroupId;

new_key_type! { pub struct AxisInputId; }

#[derive(Default, Clone)]
pub enum AxisKind {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    #[default]
    Infinite,
}

#[derive(Default, Clone)]
pub struct AxisDescriptor {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub kind: AxisKind,
}

pub struct AxisInput {
    pub descriptor: AxisDescriptor,
    pub value: f32,
    pub group: InputGroupId,
    pub id: AxisInputId,
}

impl AxisInput {
    
    pub fn set_value(&mut self, value: f32) {
        self.value = match self.descriptor.kind {
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