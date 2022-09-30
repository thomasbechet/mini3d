use hecs::World;

use crate::{backend::renderer::RendererBackend, ecs::component::{transform::TransformComponent, camera::CameraComponent}};

pub fn system_update_camera(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) {
    for (_, (t, c)) in world.query_mut::<(&TransformComponent, &CameraComponent)>() {
        renderer.update_camera(c.id, t.translation, t.forward(), t.up(), c.fov);
    }
}