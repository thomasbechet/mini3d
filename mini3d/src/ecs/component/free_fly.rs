use serde::{Serialize, Deserialize};

use crate::{asset::{input_action::InputAction, AssetRef, input_axis::InputAxis}};

#[derive(Serialize, Deserialize)]
pub struct FreeFlyComponent {

    // Inputs
    pub switch_mode: AssetRef<InputAction>,
    pub roll_left: AssetRef<InputAction>,
    pub roll_right: AssetRef<InputAction>,
    pub view_x: AssetRef<InputAxis>,
    pub view_y: AssetRef<InputAxis>,
    pub move_forward: AssetRef<InputAxis>,
    pub move_backward: AssetRef<InputAxis>,
    pub move_up: AssetRef<InputAxis>,
    pub move_down: AssetRef<InputAxis>,
    pub move_left: AssetRef<InputAxis>,
    pub move_right: AssetRef<InputAxis>,
    
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