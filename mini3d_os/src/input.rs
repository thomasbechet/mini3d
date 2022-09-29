pub struct Button;

impl Button {
    pub const UP: &'static str = "up";
    pub const DOWN: &'static str = "down";
    pub const LEFT: &'static str = "left";
    pub const RIGHT: &'static str = "right";

    pub const CLICK: &'static str = "click";
    pub const SWITCH_CONTROL_MODE: &'static str = "switch_control_mode";
}

pub struct Axis;

impl Axis {
    pub const CURSOR_X: &'static str = "cursor_x";
    pub const CURSOR_Y: &'static str = "cursor_y";
}