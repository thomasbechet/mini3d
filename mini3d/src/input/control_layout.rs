use glam::Vec2;
use slotmap::{SlotMap, SecondaryMap, new_key_type, Key};

use crate::{math::rect::IRect, graphics::{CommandBuffer, SCREEN_RESOLUTION}};

use super::{InputManager, action::ActionInputId, axis::AxisInputId};

new_key_type! { 
    pub struct ControlId;
    pub struct ControlProfileId;
}

#[derive(Clone, Copy)]
enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    pub(crate) const COUNT: usize = 4;
}

pub struct ControlInputs {

    // Selection inputs
    pub up: ActionInputId,
    pub down: ActionInputId,
    pub left: ActionInputId,
    pub right: ActionInputId,

    // Cursor inputs
    pub cursor_x: AxisInputId,
    pub cursor_y: AxisInputId,
    pub motion_x: AxisInputId,
    pub motion_y: AxisInputId,
}

enum ControlMode {
    Selection { selected: ControlId },
    Cursor { position: Vec2 },
}

struct ControlProfile {
    mode: ControlMode,
    inputs: ControlInputs,
    last_cursor_position: Vec2,
}

pub struct ControlLayout {

    // Layout data
    extents: SlotMap<ControlId, IRect>,
    directions: SecondaryMap<ControlId, [ControlId; Direction::COUNT]>,    
    default_control: ControlId,

    // Profiles
    profiles: SlotMap<ControlProfileId, ControlProfile>,
}

impl ControlLayout {

    fn compute_directions(&mut self) {

        for (id, extent) in self.extents.iter() {
            let tl = extent.tl();
            let br = extent.br();

            let mut current = [ControlId::null(); Direction::COUNT];

            for (n_id, &n_extent) in self.extents.iter() {

                // Ignore itself
                if n_id.0 == id.0 { continue };

                let ntl = n_extent.tl();
                let nbr = n_extent.br();

                let h: [bool; 3] = [
                    ntl.x < tl.x && nbr.x < tl.x,
                    !(nbr.x < tl.x || ntl.x > br.x),
                    nbr.x > br.x && ntl.x > tl.x,
                ];

                let v: [bool; 3] = [
                    ntl.y < tl.y && nbr.y < tl.y,
                    !(nbr.y < tl.y || ntl.y > br.y),
                    nbr.y > br.y && ntl.y > tl.y,
                ];

                if v[0] && h[0] { // Top-Left

                } else if v[0] && h[1] { // Top
                    if let Some(concurrent) = self.extents.get(current[Direction::Up as usize]) {
                        if n_extent.br().y > concurrent.br().y {
                            current[Direction::Up as usize] = n_id;
                        }
                    } else {
                        current[Direction::Up as usize] = n_id;
                    }
                } else if v[0] && h[2] { // Top-Right

                } else if v[1] && h[0] { // Left
                    if let Some(concurrent) = self.extents.get(current[Direction::Left as usize]) {
                        if n_extent.br().x > concurrent.br().x {
                            current[Direction::Left as usize] = n_id;
                        }
                    } else {
                        current[Direction::Left as usize] = n_id;
                    }
                } else if v[1] && h[2] { // Right
                    if let Some(concurrent) = self.extents.get(current[Direction::Right as usize]) {
                        if n_extent.tl().x < concurrent.tl().x {
                            current[Direction::Right as usize] = n_id;
                        }
                    } else {
                        current[Direction::Right as usize] = n_id;
                    }
                } else if v[2] && h[0] { // Bottom-Left

                } else if v[2] && h[1] { // Bottom                    
                    if let Some(concurrent) = self.extents.get(current[Direction::Down as usize]) {
                        if n_extent.tl().y < concurrent.tl().y {
                            current[Direction::Down as usize] = n_id;
                        }
                    } else {
                        current[Direction::Down as usize] = n_id;
                    }
                } else if v[2] && h[2] { // Bottom-Right

                } else {
                    panic!("Invalid extent position.")
                }
            }

            // Save new directions
            self.directions.insert(id, current);
        }
    }

    pub fn new() -> Self {
        Self { 
            extents: Default::default(),
            directions: Default::default(),
            default_control: ControlId::null(),
            profiles: Default::default(),
        }
    }

    pub fn add_control(&mut self, extent: IRect) -> ControlId {
        let id = self.extents.insert(extent);
        self.compute_directions();
        if self.default_control.is_null() {
            self.default_control = id;
        }
        id
    }

    pub fn add_profile(&mut self, inputs: ControlInputs) -> ControlProfileId {
        self.profiles.insert(ControlProfile {
            mode: ControlMode::Selection { selected: ControlId::null() },
            inputs,
            last_cursor_position: Default::default(),
        })
    }

    pub fn target_control(&self, id: ControlProfileId) -> ControlId {
        match self.profiles.get(id).unwrap().mode {
            ControlMode::Selection { selected } => selected,
            ControlMode::Cursor { position } => {
                // Find the area that contains the point
                self.extents.iter().find(|(_, &extent)| { extent.contains(position.as_ivec2()) })
                    .map_or(ControlId::null(), |(id, _)| id)    
            },
        }
    }

    pub fn update(&mut self, input: &InputManager) {

        // Update per profile
        for (_, profile) in self.profiles.iter_mut() {

            // Selection inputs
            let up = input.action(profile.inputs.up).map_or_else(|| false, |b| b.is_just_pressed());
            let down = input.action(profile.inputs.down).map_or_else(|| false, |b| b.is_just_pressed());
            let left = input.action(profile.inputs.left).map_or_else(|| false, |b| b.is_just_pressed());
            let right = input.action(profile.inputs.right).map_or_else(|| false, |b| b.is_just_pressed());
            
            // Cursor inputs
            let cursor_x = input.axis(profile.inputs.cursor_x).map_or(profile.last_cursor_position.x, |a| a.value);
            let cursor_y = input.axis(profile.inputs.cursor_y).map_or(profile.last_cursor_position.y, |a| a.value);
            let motion_x = input.axis(profile.inputs.motion_x).map_or(0.0, |a| a.value);
            let motion_y = input.axis(profile.inputs.motion_y).map_or(0.0, |a| a.value);
            
            // Update detection
            let selection_update = up || down || left || right;
            let motion_update = motion_x != 0.0 || motion_y != 0.0;
            let cursor_update = cursor_x != profile.last_cursor_position.x || cursor_y != profile.last_cursor_position.y;

            // Selection or cursor mode
            if selection_update {

                // Find the current selection
                let current_selection = match profile.mode {
                    ControlMode::Selection { selected } => selected,
                    ControlMode::Cursor { .. } => { ControlId::null() },
                };

                // Find the new selection
                let new_selection = {
                    if current_selection.is_null() {
                        self.default_control   
                    } else {
                        // Find the next selection
                        let next_selection = {
                            if up {
                                self.directions.get(current_selection).map_or(current_selection, |d| d[Direction::Up as usize]) 
                            } else if down {
                                self.directions.get(current_selection).map_or(current_selection, |d| d[Direction::Down as usize]) 
                            } else if left {
                                self.directions.get(current_selection).map_or(current_selection, |d| d[Direction::Left as usize]) 
                            } else if right {
                                self.directions.get(current_selection).map_or(current_selection, |d| d[Direction::Right as usize]) 
                            } else {
                                ControlId::null()
                            }
                        };
                        // Check no direction assigned (just keep the old selection)
                        if next_selection.is_null() {
                            current_selection
                        } else {
                            next_selection
                        }
                    }
                };
                
                // Update mode
                profile.mode = ControlMode::Selection { selected: new_selection };
            } else if motion_update || cursor_update {
                
                // Find the current cursor position
                let mut current_position = match profile.mode {
                    ControlMode::Selection { selected } => {
                        // Find the center of the selection. This help the user 
                        // by navigating with the selection mode then using the 
                        // cursor to reach the target.
                        if let Some(rect) = self.extents.get(selected) {
                            rect.center().as_vec2()
                        } else {
                            profile.last_cursor_position
                        }
                    },
                    ControlMode::Cursor { position } => position,
                };

                // Update the position according the event
                if cursor_update {
                    current_position = Vec2::new(cursor_x, cursor_y);
                    profile.last_cursor_position = current_position;
                } else if motion_update {
                    current_position += Vec2::new(motion_x, motion_y);
                }

                // Update the mode
                profile.mode = ControlMode::Cursor { 
                    position: current_position.clamp(Vec2::ZERO, SCREEN_RESOLUTION.as_vec2()), 
                };
            }
        }
    }

    pub fn render(&self) -> CommandBuffer {
        let mut cbb = CommandBuffer::builder();

        // Render profiles
        for (_, (_, profile)) in self.profiles.iter().enumerate() {

            // Display selection box or cursor
            // TODO: each profile have an associated color
            // TODO: two selection box on the same extent have special design
            match &profile.mode {
                ControlMode::Selection { selected } => {
                    if let Some(extent) = self.extents.get(*selected) {
                        cbb.draw_rect(*extent);
                    }
                },
                ControlMode::Cursor { position } => {
                    let sp = position.as_ivec2();
                    cbb.fill_rect(IRect::new(sp.x, sp.y, 3, 3));
                },
            }
        }

        cbb.build()
    }
}