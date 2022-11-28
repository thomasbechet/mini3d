use anyhow::{Result, anyhow, Context};
use glam::{Vec2, IVec2};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{math::rect::IRect, graphics::{SCREEN_RESOLUTION, command_buffer::{CommandBuffer, Command}}, uid::UID, input::InputManager};

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

fn alpha(last_time: f64, time: f64) -> f64 {
    let mut x = (time - last_time) / 0.4;
    x = 1.0 - (1.0 - x).powi(3);
    x.clamp(0.0, 1.0)
}

#[derive(Serialize, Deserialize)]
pub struct NavigationLayoutInputs {

    // Selection inputs
    pub up: UID,
    pub down: UID,
    pub left: UID,
    pub right: UID,

    // Cursor inputs
    pub cursor_x: UID,
    pub cursor_y: UID,
    pub cursor_motion_x: UID,
    pub cursor_motion_y: UID,
}

#[derive(Serialize, Deserialize)]
struct VisualSelection {
    source_extent: IRect,
    target_extent: IRect,
    source_time: f64,
}

impl VisualSelection {
    fn new(extent: IRect) -> Self {
        Self { source_extent: extent, target_extent: extent, source_time: 0.0 }
    }
}

#[derive(Serialize, Deserialize)]
enum NavigationMode {
    Selection { uid: UID, visual: VisualSelection },
    Cursor { position: Vec2 },
}

#[derive(Serialize, Deserialize)]
struct Profile {
    name: String,
    mode: Option<NavigationMode>,
    inputs: NavigationLayoutInputs,
    last_cursor_position: Vec2,
}

#[derive(Serialize, Deserialize)]
struct Area {
    name: String,
    extent: IRect,
    directions: [Option<UID>; Direction::COUNT],
}

#[derive(Default, Serialize, Deserialize)]
pub struct NavigationLayout {
    areas: HashMap<UID, Area>,
    default_area: Option<UID>,
    profiles: HashMap<UID, Profile>,
}

impl NavigationLayout {

    fn render_selection(extent: IRect, time: f64, cb: &mut CommandBuffer) {
        let offset = if (time % 1.0) > 0.5 { 1 } else { 0 };
        let length = 2;

        let tl = extent.tl() + IVec2::new(-offset, -offset);
        let tr = extent.tr() + IVec2::new(offset, -offset); 
        let bl = extent.bl() + IVec2::new(-offset, offset); 
        let br = extent.br() + IVec2::new(offset, offset); 

        cb.push(Command::DrawHLine { y: tl.y, x0: tl.x, x1: tl.x + length });
        cb.push(Command::DrawVLine { x: tl.x, y0: tl.y, y1: tl.y + length });
        
        cb.push(Command::DrawHLine { y: tr.y, x0: tr.x - length, x1: tr.x });
        cb.push(Command::DrawVLine { x: tr.x, y0: tr.y, y1: tr.y + length });

        cb.push(Command::DrawHLine { y: bl.y, x0: bl.x, x1: bl.x + length });
        cb.push(Command::DrawVLine { x: bl.x, y0: bl.y - length, y1: bl.y });

        cb.push(Command::DrawHLine { y: br.y, x0: br.x - length, x1: br.x });
        cb.push(Command::DrawVLine { x: br.x, y0: br.y - length, y1: br.y });
    }

    fn render_cursor(position: IVec2, _time: f64, cb: &mut CommandBuffer) {
        cb.push(Command::DrawHLine { y: position.y, x0: position.x - 1, x1: position.x + 1 });
        cb.push(Command::DrawVLine { x: position.x, y0: position.y - 1, y1: position.y + 1 });
    }

    fn compute_directions(&mut self) {

        for uid in self.areas.keys().copied().collect::<Vec<_>>() {
            let area = self.areas.get(&uid).unwrap();
            let tl = area.extent.tl();
            let br = area.extent.br();

            let mut current = [Option::<UID>::default(); Direction::COUNT];

            for (n_id, n_area) in self.areas.iter() {

                // Ignore itself
                if *n_id == uid { continue };

                let ntl = n_area.extent.tl();
                let nbr = n_area.extent.br();

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
                    if let Some(concurrent) = current[Direction::Up as usize].and_then(|uid| self.areas.get(&uid)) {
                        if n_area.extent.br().y > concurrent.extent.br().y {
                            current[Direction::Up as usize] = Some(*n_id);
                        }
                    } else {
                        current[Direction::Up as usize] = Some(*n_id);
                    }
                } else if v[0] && h[2] { // Top-Right

                } else if v[1] && h[0] { // Left
                    if let Some(concurrent) = current[Direction::Left as usize].and_then(|uid| self.areas.get(&uid)) {
                        if n_area.extent.br().x > concurrent.extent.br().x {
                            current[Direction::Left as usize] = Some(*n_id);
                        }
                    } else {
                        current[Direction::Left as usize] = Some(*n_id);
                    }
                } else if v[1] && h[2] { // Right
                    if let Some(concurrent) = current[Direction::Right as usize].and_then(|uid| self.areas.get(&uid)) {
                        if n_area.extent.tl().x < concurrent.extent.tl().x {
                            current[Direction::Right as usize] = Some(*n_id);
                        }
                    } else {
                        current[Direction::Right as usize] = Some(*n_id);
                    }
                } else if v[2] && h[0] { // Bottom-Left

                } else if v[2] && h[1] { // Bottom                    
                    if let Some(concurrent) = current[Direction::Down as usize].and_then(|uid| self.areas.get(&uid)) {
                        if n_area.extent.tl().y < concurrent.extent.tl().y {
                            current[Direction::Down as usize] = Some(*n_id);
                        }
                    } else {
                        current[Direction::Down as usize] = Some(*n_id);
                    }
                } else if v[2] && h[2] { // Bottom-Right

                } else {
                    panic!("Invalid extent position.")
                }
            }

            // Save new directions
            let area = self.areas.get_mut(&uid).unwrap();
            area.directions = current;
        }
    }

    pub fn add_area(&mut self, name: &str, extent: IRect) -> Result<UID> {
        let uid = UID::new(name);
        if self.areas.contains_key(&uid) { return Err(anyhow!("Area name already exists")); }
        self.areas.insert(uid, Area { name: name.to_string(), extent, directions: Default::default() });
        self.compute_directions();
        if self.default_area.is_none() {
            self.default_area = Some(uid);
        }
        Ok(uid)
    }

    pub fn add_profile(&mut self, name: &str, inputs: NavigationLayoutInputs) -> Result<UID> {
        let uid = UID::new(name);
        if self.profiles.contains_key(&uid) { return Err(anyhow!("Profile name already exists")); }
        self.profiles.insert(uid, Profile {
            name: name.to_string(),
            mode: None,
            inputs,
            last_cursor_position: Default::default(),
        });
        Ok(uid)
    }

    pub fn hovered_area(&self, profile: UID) -> Result<Option<UID>> {
        let profile = self.profiles.get(&profile).with_context(|| "Profile not found")?;
        Ok(profile.mode.as_ref().and_then(|mode| {
            match mode {
                NavigationMode::Selection { uid, visual: _ } => Some(*uid),
                NavigationMode::Cursor { position } => {
                    // Find the area that contains the point
                    self.areas.iter().find(|(_, area)| { area.extent.contains(position.as_ivec2()) }).map(|(uid, _)| *uid)
                },
            }
        }))
    }

    pub fn update(&mut self, input: &InputManager, time: f64) -> Result<()> {

        // Update per profile
        for profile in self.profiles.values_mut() {

            // Selection inputs
            let direction = {
                if input.action(profile.inputs.up)?.is_just_pressed() {
                    Some(Direction::Up)
                } else if input.action(profile.inputs.down)?.is_just_pressed() {
                    Some(Direction::Down)
                } else if input.action(profile.inputs.left)?.is_just_pressed() {
                    Some(Direction::Left)
                } else if input.action(profile.inputs.right)?.is_just_pressed() {
                    Some(Direction::Right)
                } else {
                    None
                }
            };
            
            // Cursor inputs
            let cursor_x = input.axis(profile.inputs.cursor_x)?.value;
            let cursor_y = input.axis(profile.inputs.cursor_y)?.value;
            let motion_x = input.axis(profile.inputs.cursor_motion_x)?.value;
            let motion_y = input.axis(profile.inputs.cursor_motion_y)?.value;
            
            // Update detection
            let motion_update = motion_x != 0.0 || motion_y != 0.0;
            let cursor_update = cursor_x != profile.last_cursor_position.x || cursor_y != profile.last_cursor_position.y;

            // Update last cursor
            profile.last_cursor_position = Vec2::new(cursor_x, cursor_y);

            if let Some(mode) = &mut profile.mode {
                match mode {
                    NavigationMode::Selection { uid, visual } => {
                        // Next direction
                        if let Some(direction) = direction {
                            let current_area = self.areas.get(uid).unwrap();
                            if let Some(next) = current_area.directions[direction as usize] {
                                *uid = next;
                                visual.source_extent = visual.source_extent.lerp(&visual.target_extent, alpha(visual.source_time, time) as f32);
                                visual.target_extent = self.areas.get(&next).unwrap().extent;
                                visual.source_time = time;
                            }
                        }
                        // Change mode
                        if cursor_update || motion_update {
                            let center = self.areas.get(uid).unwrap().extent.center();
                            *mode = NavigationMode::Cursor { position: center.as_vec2() }
                        }
                    },
                    NavigationMode::Cursor { position } => {
                        // Update cursor position
                        if cursor_update {
                            *position = Vec2::new(cursor_x, cursor_y);
                        } else if motion_update {
                            *position += Vec2::new(motion_x, motion_y);
                        }
                        *position = position.clamp(Vec2::ZERO, SCREEN_RESOLUTION.as_vec2());
                        // Change mode
                        if direction.is_some() && self.default_area.is_some() {
                            let area_uid = self.default_area.unwrap();
                            let area = self.areas.get(&area_uid).unwrap();
                            *mode = NavigationMode::Selection { uid: area_uid, visual: VisualSelection::new(area.extent) };
                        }
                    },
                }
            } else if direction.is_some() && self.default_area.is_some() {
                let area_uid = self.default_area.unwrap();
                let area = self.areas.get(&area_uid).unwrap();                
                profile.mode = Some(NavigationMode::Selection { uid: area_uid, visual: VisualSelection::new(area.extent) });
            } else if cursor_update || motion_update {
                profile.mode = Some(NavigationMode::Cursor { position: Vec2::new(cursor_x, cursor_y) });
            }
        }
        Ok(())
    }

    pub fn render(&self, time: f64) -> CommandBuffer {
        let mut cb = CommandBuffer::empty();

        // Render profiles
        for (_, (_, profile)) in self.profiles.iter().enumerate() {

            // Display selection box or cursor
            // TODO: each profile have an associated color
            // TODO: two selection box on the same extent have special design
            if let Some(mode) = &profile.mode {
                match mode {
                    NavigationMode::Selection { uid: _, visual } => {
                        let extent = visual.source_extent.lerp(&visual.target_extent, alpha(visual.source_time, time) as f32);
                        NavigationLayout::render_selection(extent, time, &mut cb);
                    },
                    NavigationMode::Cursor { position } => {
                        NavigationLayout::render_cursor(position.as_ivec2(), time, &mut cb);
                    },
                }
            }
        }

        cb
    }
}