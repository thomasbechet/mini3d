use glam::Vec3;
use hecs::World;

use crate::{ecs::component::{transform::TransformComponent, free_view::FreeViewComponent}, input::InputManager};

pub fn system_free_view(world: &mut World, input: &InputManager) {
    for (_, (transform, free_view)) in world.query_mut::<(&mut TransformComponent, &FreeViewComponent)>() {
     
        // Compute camera translation
        let direction = Vec3::ZERO;
        
    }
}