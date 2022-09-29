use glam::Vec2;
use slotmap::{SlotMap, SecondaryMap, new_key_type, Key};

use crate::{math::rect::IRect, graphics::CommandBuffer};

use super::{direction::Direction, InputManager, button::ButtonInputId, axis::AxisInputId, cursor::Cursor};

new_key_type! { 
    pub struct ControlId;
    pub struct ControlProfileId;
}

enum ControlMode {
    Selection { selected: ControlId },
    VirtualCursor { cursor: Cursor },
}

struct ControlProfile {
    mode: ControlMode,
    switch_mode: ButtonInputId,
    up: ButtonInputId,
    down: ButtonInputId,
    left: ButtonInputId,
    right: ButtonInputId,
    motion_x: AxisInputId,
    motion_y: AxisInputId,
    use_mouse: bool,
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
                    }
                } else if v[0] && h[2] { // Top-Right

                } else if v[1] && h[0] { // Left
                    if let Some(concurrent) = self.extents.get(current[Direction::Left as usize]) {
                        if n_extent.br().x > concurrent.br().x {
                            current[Direction::Left as usize] = n_id;
                        }
                    }
                } else if v[1] && h[2] { // Right
                    if let Some(concurrent) = self.extents.get(current[Direction::Right as usize]) {
                        if n_extent.tl().x < concurrent.tl().x {
                            current[Direction::Right as usize] = n_id;
                        }
                    }
                } else if v[2] && h[0] { // Bottom-Left

                } else if v[2] && h[1] { // Bottom
                    if let Some(concurrent) = self.extents.get(current[Direction::Down as usize]) {
                        if n_extent.tl().y < concurrent.tl().y {
                            current[Direction::Down as usize] = n_id;
                        }
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
        id
    }

    pub fn add_profile(&mut self,
        switch_mode: ButtonInputId,
        up: ButtonInputId,
        down: ButtonInputId,
        left: ButtonInputId,
        right: ButtonInputId,
        motion_x: AxisInputId,
        motion_y: AxisInputId,
        use_mouse: bool,
    ) -> ControlProfileId {
        self.profiles.insert(ControlProfile {
            mode: ControlMode::Selection { selected: ControlId::null() },
            switch_mode,
            up,
            down,
            left,
            right,
            motion_x,
            motion_y,
            use_mouse,
        })
    }

    pub fn target_control(&self, id: ControlProfileId) -> ControlId {
        match self.profiles.get(id).unwrap().mode {
            ControlMode::Selection { selected } => selected,
            ControlMode::VirtualCursor { cursor } => {
                // Find the area that contains the point
                self.extents.iter().find(|(_, &extent)| { extent.contains(cursor.screen_position()) })
                    .map_or(ControlId::null(), |(id, _)| id)    
            },
        }
    }

    pub fn update(&mut self, input: &InputManager) {

        // Update per profile
        for (_, profile) in self.profiles.iter_mut() {

            // Check selection mode
            if input.button(profile.switch_mode).map_or_else(|| false, |b| b.is_just_pressed()) {
                match profile.mode {
                    // TODO: switching from selection to cursor mode should place the cursor
                    // to the nearest selected extent or the middle screen position.
                    ControlMode::Selection { .. } => {
                        profile.mode = ControlMode::VirtualCursor { cursor: Cursor::default() };
                    },
                    // TODO: switching from cursor to selection mode should place the selection
                    // to the nearest extent or none (impossible ?).
                    ControlMode::VirtualCursor { .. } => {
                        profile.mode = ControlMode::Selection { selected: ControlId::null() };
                    },
                }
            }

            // Selection behaviour differ with the cursor mode
            match &mut profile.mode {
                ControlMode::Selection { selected } => {

                    // Find the new control id
                    let new_selected = {
                        if input.button(profile.up).map_or_else(|| false, |b| b.is_just_pressed()) {
                            self.directions.get(*selected).map_or(self.default_control, |d| d[Direction::Up as usize]) 
                        } else if input.button(profile.down).map_or_else(|| false, |b| b.is_just_pressed()) {
                            self.directions.get(*selected).map_or(self.default_control, |d| d[Direction::Down as usize]) 
                        } else if input.button(profile.left).map_or_else(|| false, |b| b.is_just_pressed()) {
                            self.directions.get(*selected).map_or(self.default_control, |d| d[Direction::Left as usize]) 
                        } else if input.button(profile.right).map_or_else(|| false, |b| b.is_just_pressed()) {
                            self.directions.get(*selected).map_or(self.default_control, |d| d[Direction::Right as usize]) 
                        } else {
                            *selected
                        }
                    };

                    // Update the selected extent
                    *selected = new_selected;
                },
                ControlMode::VirtualCursor { cursor } => {

                    // Check if the profile is using the mouse to control the virtual cursor
                    // or is using motion axis (such as controll inputs)
                    if profile.use_mouse {
                        cursor.reset_motion();
                        cursor.set_position(input.mouse().position());
                    } else {
                        cursor.translate(Vec2::new(
                            input.axis(profile.motion_x).map_or(0.0, |r| r.value),
                            input.axis(profile.motion_y).map_or(0.0, |r| r.value),
                        ));
                    }
                },
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
                ControlMode::VirtualCursor { cursor } => {
                    let sp = cursor.screen_position();
                    cbb.fill_rect(IRect::new(sp.x, sp.y, 3, 3));
                },
            } 
        }

        cbb.build()
    }
}