use serde::{Serialize, Deserialize};

use crate::{input::{InputActionId, InputAxisId}};

#[derive(Serialize, Deserialize)]
pub struct FreeFlyComponent {

    // Inputs
    pub switch_mode: InputActionId,
    pub roll_left: InputActionId,
    pub roll_right: InputActionId,
    pub view_x: InputAxisId,
    pub view_y: InputAxisId,
    pub move_forward: InputAxisId,
    pub move_backward: InputAxisId,
    pub move_up: InputAxisId,
    pub move_down: InputAxisId,
    pub move_left: InputAxisId,
    pub move_right: InputAxisId,
    
    // View data
    pub free_mode: bool,
    pub yaw: f32,
    pub pitch: f32,
}

impl FreeFlyComponent {
    pub const NORMAL_SPEED: f32 = 10.0;
    pub const SLOW_SPEED: f32 = 10.0;
    pub const FAST_SPEED: f32 = 10.0;
    pub const ROLL_SPEED: f32 = 60.0;
    pub const ROTATION_SENSIBILITY: f32 = 180.0;
    pub const ZOOM_SPEED: f32 = 10.0;
}