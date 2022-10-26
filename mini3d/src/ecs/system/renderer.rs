use hecs::World;
use slotmap::Key;

use crate::{backend::renderer::{RendererBackend, ModelHandle, CameraHandle}, ecs::component::{transform::TransformComponent, camera::CameraComponent, lifecycle::LifecycleComponent, model::ModelComponent}, asset::AssetManager};

pub fn system_renderer_check_lifecycle(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
    asset: &AssetManager,
) {
    // Camera
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if l.alive && c.id.is_null() {
            c.id = renderer.add_camera();
        } else if !l.alive && !c.id.is_null() {
            renderer.remove_camera(c.id);
            c.id = CameraHandle::null();
        }
    }

    // Model
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if l.alive && m.handle.is_null() {
            m.handle = renderer.add_model(&m.model, asset);
        } else if !l.alive && !m.handle.is_null() {
            renderer.remove_model(m.handle);
            m.handle = ModelHandle::null();
        }
    }
}

pub fn system_renderer_transfer_transforms(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) {
    for (_, (t, m)) in world.query_mut::<(&TransformComponent, &ModelComponent)>() {
        renderer.transfer_model_transform(m.handle, t.matrix());
    }
}

pub fn system_renderer_update_camera(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) {
    for (_, (t, c)) in world.query_mut::<(&TransformComponent, &CameraComponent)>() {
        renderer.update_camera(c.id, t.translation, t.forward(), t.up(), c.fov);
    }
}