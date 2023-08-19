use mini3d_derive::{Component, Reflect, Serialize};

use crate::utils::uid::UID;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct FreeFly {
    // Control if free fly is active
    pub active: bool,

    // Inputs
    pub switch_mode: UID,
    pub roll_left: UID,
    pub roll_right: UID,
    pub view_x: UID,
    pub view_y: UID,
    pub move_forward: UID,
    pub move_backward: UID,
    pub move_up: UID,
    pub move_down: UID,
    pub move_left: UID,
    pub move_right: UID,
    pub move_fast: UID,
    pub move_slow: UID,

    // View data
    pub free_mode: bool,
    pub yaw: f32,
    pub pitch: f32,
}

impl FreeFly {
    pub const NORMAL_SPEED: f32 = 10.0;
    pub const FAST_SPEED: f32 = 25.0;
    pub const SLOW_SPEED: f32 = 3.0;
    pub const ROLL_SPEED: f32 = 60.0;
    pub const ROTATION_SENSIBILITY: f32 = 180.0;
    pub const ZOOM_SPEED: f32 = 10.0;
}
