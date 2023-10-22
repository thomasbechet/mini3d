use mini3d_derive::{Resource, Serialize};

#[derive(Serialize, Clone, Resource)]
pub struct InputTextState {
    pub value: String,
}
