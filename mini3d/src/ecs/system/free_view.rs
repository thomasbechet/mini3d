use hecs::World;

use crate::ecs::component::{transform::TransformComponent, free_view::FreeViewComponent};

pub fn system_free_view(world: &mut World) {
    for (_, (transform, _)) 
        in world.query_mut::<(&mut TransformComponent, &FreeViewComponent)>() {
        
    }
}