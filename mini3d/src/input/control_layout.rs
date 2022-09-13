use glam::IVec2;
use slotmap::{SlotMap, SecondaryMap, new_key_type};

use crate::math::rect::IRect;

use super::direction::Direction;

new_key_type! { pub struct ControlId; }

#[derive(Default, Clone)]
pub struct ControlLayout {
    extents: SlotMap<ControlId, IRect>,
    directions: SecondaryMap<ControlId, [Option<ControlId>; Direction::COUNT]>,    
    default_control: Option<ControlId>,
}

impl ControlLayout {

    pub fn new() -> Self {
        let mut control_layout = ControlLayout {
            extents: SlotMap::default(),
            directions: SecondaryMap::new(),
            default_control: None
        };

        let default = control_layout.add_control(IRect::new(5, 5, 100, 40));
        control_layout.add_control(IRect::new(5, 45, 100, 40));
        control_layout.default_control = Some(default);

        control_layout.compute_directions();

        control_layout
    }

    pub fn add_control(&mut self, extent: IRect) -> ControlId {
        self.extents.insert(extent)
    }

    fn compute_directions(&mut self) {

        for id in self.extents.keys() {

            let extent = self.extents.get(id).unwrap();

            let tl = extent.tl();
            let br = extent.br();

            let mut current: [Option<ControlId>; Direction::COUNT] = [None, None, None, None];

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
                    current[Direction::Up as usize] = current[Direction::Up as usize]
                        .map_or(Some(n_id), |x| {
                            let concurrent = self.extents.get(x).unwrap();
                            if n_extent.br().y > concurrent.br().y {
                                Some(n_id)
                            } else {
                                Some(x)
                            }
                        });
                } else if v[0] && h[2] { // Top-Right

                } else if v[1] && h[0] { // Left
                    current[Direction::Left as usize] = current[Direction::Left as usize]
                        .map_or(Some(n_id), |x| {
                            let concurrent = self.extents.get(x).unwrap();
                            if n_extent.br().x > concurrent.br().x {
                                Some(n_id)
                            } else {
                                Some(x)
                            }
                        });
                } else if v[1] && h[2] { // Right
                    current[Direction::Right as usize] = current[Direction::Right as usize]
                        .map_or(Some(n_id), |x| {
                            let concurrent = self.extents.get(x).unwrap();
                            if n_extent.tl().x < concurrent.tl().x {
                                Some(n_id)
                            } else {
                                Some(x)
                            }
                        });
                } else if v[2] && h[0] { // Bottom-Left

                } else if v[2] && h[1] { // Bottom
                    current[Direction::Down as usize] = current[Direction::Down as usize]
                        .map_or(Some(n_id), |x| {
                            let concurrent = self.extents.get(x).unwrap();
                            if n_extent.tl().y < concurrent.tl().y {
                                Some(n_id)
                            } else {
                                Some(x)
                            }
                        });
                } else if v[2] && h[2] { // Bottom-Right

                } else {
                    panic!("Invalid extent position.")
                }

            }

            // Save new directions
            self.directions.insert(id, current);

        }
        
    }

    pub fn get_control_extent(&self, id: ControlId) -> Option<IRect> {
        self.extents.get(id).copied()
    }

    pub fn get_control_from_direction(&mut self, id: Option<ControlId>, direction: Direction) -> Option<ControlId> {
        id.map_or(self.default_control, |x_id| {
            self.directions.get(x_id).unwrap()[direction as usize].map_or(Some(x_id), |x| {
                Some(x)
            })
        })
    }

    pub fn get_control_from_position(&self, p: IVec2) -> Option<ControlId> {
        // Find the area that contains the point
        self.extents.iter().find(|(_, &extent)| { extent.contains(p) })
            .map(|(id, _)| { id }) // Return the area id only
    }
}