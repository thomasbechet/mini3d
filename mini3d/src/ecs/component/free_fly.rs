use slotmap::Key;

use crate::input::{axis::AxisInputId, button::ButtonInputId};

pub struct FreeFlyComponent {

    // Inputs
    pub switch_mode: ButtonInputId,
    pub roll_left: ButtonInputId,
    pub roll_right: ButtonInputId,
    pub look_x: AxisInputId,
    pub look_y: AxisInputId,
    pub move_forward: AxisInputId,
    pub move_backward: AxisInputId,
    pub move_up: AxisInputId,
    pub move_down: AxisInputId,
    pub move_left: AxisInputId,
    pub move_right: AxisInputId,
    
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
    pub const ROTATION_SENSIBILITY: f32 = 10.0;
    pub const ZOOM_SPEED: f32 = 10.0;

    pub fn new() -> Self {
        Self {
            switch_mode: ButtonInputId::null(),
            roll_left: ButtonInputId::null(),
            roll_right: ButtonInputId::null(),
            look_x: AxisInputId::null(),
            look_y: AxisInputId::null(),
            move_forward: AxisInputId::null(),
            move_backward: AxisInputId::null(),
            move_up: AxisInputId::null(),
            move_down: AxisInputId::null(),
            move_left: AxisInputId::null(),
            move_right: AxisInputId::null(),
            free_mode: false,
            yaw: 0.0,
            pitch: 0.0,
        }   
    }
}