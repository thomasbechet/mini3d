pub struct OSGroup;

impl OSGroup {
    pub const INPUT: &'static str = "default";
    pub const ASSET: &'static str = "default";
}

pub struct OSAction;

impl OSAction {
    pub const UP: &'static str = "up"; 
    pub const LEFT: &'static str = "left"; 
    pub const DOWN: &'static str = "down"; 
    pub const RIGHT: &'static str = "right";
}

pub struct OSAxis;

impl OSAxis {
    pub const CURSOR_X: &'static str = "cursor_x";
    pub const CURSOR_Y: &'static str = "cursor_y";
    pub const MOTION_X: &'static str = "motion_x";
    pub const MOTION_Y: &'static str = "motion_y";
}