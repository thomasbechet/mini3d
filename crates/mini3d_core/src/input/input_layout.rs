use std::{collections::HashMap, fmt::Display};

use glam::IVec2;

use crate::math::rect::IRect;

use super::direction::Direction;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct AreaId(u32);

impl Display for AreaId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Clone)]
struct Area {
    extent: IRect,
    directions: [Option<AreaId>; Direction::COUNT],
}

impl Area {
    pub fn new(
        extent: IRect, 
        up: Option<AreaId>,
        down: Option<AreaId>,
        left: Option<AreaId>,
        right: Option<AreaId>,
    ) -> Self {
        Area { 
            extent,
            directions: [up, down, left, right],
        }
    }
}

#[derive(Default, Clone)]
pub struct InputLayout {
    areas: HashMap<AreaId, Area>,
    default_area: Option<AreaId>,
}

impl InputLayout {
    pub fn new() -> Self {
        InputLayout {
            areas: HashMap::from([
                (AreaId(0), Area::new(IRect::new(5, 5, 100, 50), None, None, None, Some(AreaId(1)))),
                (AreaId(1), Area::new(IRect::new(120, 5, 100, 50), None, None, Some(AreaId(0)), None)),
            ]),
            default_area: Some(AreaId(0))
        }
    }

    pub fn get_area_extent(&self, id: AreaId) -> Option<IRect> {
        if let Some(area) = self.areas.get(&id) {
            Some(area.extent)
        } else {
            None
        }
    }

    pub fn move_next(&mut self, id: Option<AreaId>, direction: Direction) -> Option<AreaId> {
        if let Some(id) = id {
            if let Some(next) = self.areas.get(&id).unwrap().directions[direction as usize] {
                Some(next)
            } else {
                Some(id)
            }
        } else {
            self.default_area
        }
    }

    pub fn find_area(&self, p: IVec2) -> Option<AreaId> {
        // Find the area that contains the point
        self.areas.iter().find(|&x| { x.1.extent.contains(p) })
            .map(|x| { *x.0 }) // Return the area id only
    }
}