use slotmap::Key;

use crate::input::axis::AxisInputId;

pub struct FreeViewComponent {
    pub look_x: AxisInputId,
    pub look_y: AxisInputId,
    pub move_forward: AxisInputId,
    pub move_backward: AxisInputId,
    pub move_up: AxisInputId,
    pub move_down: AxisInputId,
    pub move_left: AxisInputId,
    pub move_right: AxisInputId,
    pub free_mode: bool,
    pub yaw: f32,
    pub pitch: f32,
    pub fov: f32,
}

impl FreeViewComponent {

    pub const NORMAL_SPEED: f32 = 10.0 / 1000.0;
    pub const SLOW_SPEED: f32 = 10.0 / 1000.0;
    pub const FAST_SPEED: f32 = 10.0 / 1000.0;
    pub const ROLL_SPEED: f32 = 10.0 / 1000.0;
    pub const ROTATION_SENSIBILITY: f32 = 10.0 / 1000.0;
    pub const ZOOM_SPEED: f32 = 10.0 / 1000.0;

    pub fn new() -> Self {
        Self {
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
            fov: 80.0,
        }   
    }
}