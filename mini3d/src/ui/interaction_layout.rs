use anyhow::{Result, anyhow, Context};
use glam::{Vec2, IVec2};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

use crate::{math::rect::IRect, renderer::{command_buffer::{CommandBuffer, Command}, SCREEN_VIEWPORT, SCREEN_CENTER}, uid::UID, input::InputManager};

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
enum InteractionMode {
    Selection { visual: VisualSelection },
    Cursor { position: Vec2 },
}

#[derive(Serialize, Deserialize)]
pub struct InteractionInputs {

    // Control inputs
    pub click: UID,
    pub scroll: UID,

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
struct Profile {
    name: String,
    mode: InteractionMode,
    inputs: InteractionInputs,
    target: Option<UID>,
    #[serde(skip)]
    previous_cursor_position: Vec2,
}

#[derive(Serialize, Deserialize)]
struct Area {
    active: bool,
    extent: IRect,
    directions: [Option<UID>; Direction::COUNT],
}

#[derive(Default, Serialize, Deserialize)]
pub struct InteractionLayout {
    areas: HashMap<UID, Area>,
    default_area: Option<UID>,
    profiles: HashMap<UID, Profile>,
}

#[derive(Debug)]
pub(crate) enum AreaEvent {
    Pressed { profile: UID },
    Released { profile: UID },
    Scroll { profile: UID, value: f32 },
    Enter { profile: UID },
    Leave { profile: UID },
}

#[derive(Debug)]
pub(crate) enum ProfileEvent {
    CursorMoved { position: IVec2 },
} 

#[derive(Debug)]
pub(crate) enum InteractionEvent {
    Area { area: UID, event: AreaEvent },
    Profile { profile: UID, event: ProfileEvent },
}

impl InteractionLayout {

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

                // Ignore itself or inactive
                if *n_id == uid || !n_area.active { continue };

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

    pub fn add_area(&mut self, uid: UID, extent: IRect) -> Result<()> {
        if self.areas.contains_key(&uid) { return Err(anyhow!("Area name already exists")); }
        self.areas.insert(uid, Area { active: true, extent, directions: Default::default() });
        self.compute_directions();
        if self.default_area.is_none() {
            self.default_area = Some(uid);
        }
        Ok(())
    }

    pub fn add_profile(&mut self, name: &str, inputs: InteractionInputs) -> Result<UID> {
        let uid = UID::new(name);
        if self.profiles.contains_key(&uid) { return Err(anyhow!("Profile name already exists")); }
        self.profiles.insert(uid, Profile {
            name: name.to_string(),
            mode: InteractionMode::Selection { visual: VisualSelection::new(SCREEN_VIEWPORT) },
            inputs,
            previous_cursor_position: Default::default(),
            target: None,
        });
        Ok(uid)
    }

    pub fn set_profile_target(&mut self, profile: UID, area: Option<UID>) -> Result<()> {
        let profile = self.profiles.get_mut(&profile).with_context(|| "Profile not found")?;
        profile.target = area;
        Ok(())
    }

    pub fn profile_target(&self, profile: UID) -> Result<Option<UID>> {
        let profile = self.profiles.get(&profile).with_context(|| "Profile not found")?;
        Ok(profile.target)
    }

    pub fn set_area_active(&mut self, uid: UID, active: bool) -> Result<()> {
        let area = self.areas.get_mut(&uid).with_context(|| "Area not found")?;
        area.active = active;
        if !area.active {
            for profile in self.profiles.values_mut() {
                if profile.target.is_some() && profile.target.unwrap() == uid {
                    profile.target = None;
                }
            }
        }
        Ok(())
    }

    pub fn set_area_extent(&mut self, area: UID, extent: IRect) -> Result<()> {
        let area = self.areas.get_mut(&area).with_context(|| "Area not found")?;
        area.extent = extent;
        self.compute_directions();
        Ok(())
    }

    pub(crate) fn update(&mut self, input: &InputManager, time: f64, events: &mut Vec<InteractionEvent>) -> Result<()> {

        // Update per profile
        for (profile_uid, profile) in self.profiles.iter_mut() {

            // Keep previous target to detect enter / leaving areas
            let previous_target = profile.target;

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
            let cursor_update = cursor_x != profile.previous_cursor_position.x || cursor_y != profile.previous_cursor_position.y;

            // Update last cursor
            profile.previous_cursor_position = Vec2::new(cursor_x, cursor_y);

            // Update mode
            match &profile.mode {
                InteractionMode::Selection { visual } => {
                    if let Some(target) = profile.target {
                        if motion_update || cursor_update {
                            let area = self.areas.get(&target).unwrap();
                            profile.mode = InteractionMode::Cursor { position: area.extent.center().as_vec2() };
                            profile.target = Some(target);
                        } else if let Some(direction) = direction {
                            if let Some(next) = self.areas.get(&target).unwrap().directions[direction as usize] {
                                let area = self.areas.get(&next).unwrap();
                                let intermediate_visual = VisualSelection {
                                    source_extent: visual.source_extent.lerp(&visual.target_extent, alpha(visual.source_time, time) as f32),
                                    target_extent: area.extent,
                                    source_time: time,
                                };
                                profile.mode = InteractionMode::Selection { visual: intermediate_visual };
                                profile.target = Some(next);
                            }
                        }
                    } else if motion_update || cursor_update {
                        if cursor_update {
                            let position = Vec2::new(cursor_x, cursor_y);
                            profile.mode = InteractionMode::Cursor { position };
                            profile.target = self.areas.iter().find(|(_, area)| area.extent.contains(position.as_ivec2())).map(|(uid, _)| *uid);
                        } else {
                            let position = SCREEN_CENTER.as_vec2();
                            profile.mode = InteractionMode::Cursor { position };
                            profile.target = self.areas.iter().find(|(_, area)| area.extent.contains(position.as_ivec2())).map(|(uid, _)| *uid);
                        }
                    } else if direction.is_some() {
                        if let Some(default) = self.default_area {
                            let area = self.areas.get(&default).unwrap();
                            profile.mode = InteractionMode::Selection { visual: VisualSelection::new(area.extent) };
                            profile.target = Some(default);
                        }
                    }
                },
                InteractionMode::Cursor { position } => {
                    if let Some(target) = profile.target {
                        if motion_update || cursor_update {
                            if cursor_update {
                                let position = Vec2::new(cursor_x, cursor_y);
                                profile.mode = InteractionMode::Cursor { position };
                                profile.target = self.areas.iter().find(|(_, area)| area.extent.contains(position.as_ivec2())).map(|(uid, _)| *uid);
                            } else {
                                let position = *position + Vec2::new(motion_x, motion_y);
                                profile.mode = InteractionMode::Cursor { position };
                                profile.target = self.areas.iter().find(|(_, area)| area.extent.contains(position.as_ivec2())).map(|(uid, _)| *uid);
                            }
                        } else if direction.is_some() {
                            let area = self.areas.get(&target).unwrap();
                            profile.mode = InteractionMode::Selection { visual: VisualSelection::new(area.extent) };
                            profile.target = Some(target);
                        }
                    } else if motion_update || cursor_update {
                        if cursor_update {
                            let position = Vec2::new(cursor_x, cursor_y);
                            profile.mode = InteractionMode::Cursor { position };
                            profile.target = self.areas.iter().find(|(_, area)| area.extent.contains(position.as_ivec2())).map(|(uid, _)| *uid);
                        } else {
                            let position = *position + Vec2::new(motion_x, motion_y);
                            profile.mode = InteractionMode::Cursor { position };
                            profile.target = self.areas.iter().find(|(_, area)| area.extent.contains(position.as_ivec2())).map(|(uid, _)| *uid);
                        }
                    } else if direction.is_some() {
                        if let Some(default) = self.default_area {
                            let area = self.areas.get(&default).unwrap();
                            profile.mode = InteractionMode::Selection { visual: VisualSelection::new(area.extent) };
                            profile.target = Some(default);
                        }
                    }
                },
            }
        
            // Enter / Leave events
            if previous_target != profile.target {
                if let Some(previous) = previous_target {
                    events.push(InteractionEvent::Area { area: previous, event: AreaEvent::Leave { profile: *profile_uid } });
                }
                if let Some(new) = profile.target {
                    events.push(InteractionEvent::Area { area: new, event: AreaEvent::Enter { profile: *profile_uid } });
                }
            }

            // Scroll events
            if let Some(target) = profile.target {
                let delta = input.axis(profile.inputs.scroll)?.value;
                if delta != 0.0 {
                    events.push(InteractionEvent::Area { area: target, event: AreaEvent::Scroll { profile: *profile_uid, value: delta }});
                }
            }

            // Pressed / Released events
            if let Some(target) = profile.target {
                let action = input.action(profile.inputs.click)?;
                if action.is_just_pressed() {
                    events.push(InteractionEvent::Area { area: target, event: AreaEvent::Pressed { profile: *profile_uid }});
                } else if action.is_just_released() {
                    events.push(InteractionEvent::Area { area: target, event: AreaEvent::Released { profile: *profile_uid }});
                }
            }
        }
        Ok(())
    }

    pub fn render(&self, time: f64) -> CommandBuffer {
        
        // Render profiles
        let mut cb = CommandBuffer::empty();
        for (_, (_, profile)) in self.profiles.iter().enumerate() {

            // Display selection box or cursor
            // TODO: each profile have an associated color
            // TODO: two selection box on the same extent have special design
            match &profile.mode {
                InteractionMode::Selection { visual } => {
                    if profile.target.is_some() {
                        let extent = visual.source_extent.lerp(&visual.target_extent, alpha(visual.source_time, time) as f32);
                        InteractionLayout::render_selection(extent, time, &mut cb);
                    }
                },
                InteractionMode::Cursor { position } => {
                    InteractionLayout::render_cursor(position.as_ivec2(), time, &mut cb);
                },
            }
        }
        cb
    }
}