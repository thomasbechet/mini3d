pub struct CommonInput;

impl CommonInput {
    pub const GROUP: &'static str = "common";
}

pub struct CommonAction;

impl CommonAction {
    pub const CLICK: &'static str = "click";
    pub const BACK: &'static str = "back";
    pub const UP: &'static str = "up";
    pub const LEFT: &'static str = "left"; 
    pub const DOWN: &'static str = "down"; 
    pub const RIGHT: &'static str = "right";
    pub const CHANGE_CONTROL_MODE: &'static str = "change_control_mode";
    pub const TOGGLE_PROFILER: &'static str = "toggle_profiler";
}

pub struct CommonAxis;

impl CommonAxis {
    pub const CURSOR_X: &'static str = "cursor_x";
    pub const CURSOR_Y: &'static str = "cursor_y";
    pub const SCROLL_MOTION: &'static str = "scroll_motion";
    pub const CURSOR_MOTION_X: &'static str = "cursor_motion_x";
    pub const CURSOR_MOTION_Y: &'static str = "cursor_motion_y";
    pub const VIEW_X: &'static str = "view_x";
    pub const VIEW_Y: &'static str = "view_y";
    pub const MOVE_FORWARD: &'static str = "move_forward";
    pub const MOVE_BACKWARD: &'static str = "move_backward";
    pub const MOVE_LEFT: &'static str = "move_left";
    pub const MOVE_RIGHT: &'static str = "move_right";
    pub const MOVE_UP: &'static str = "move_up";
    pub const MOVE_DOWN: &'static str = "move_down";
}