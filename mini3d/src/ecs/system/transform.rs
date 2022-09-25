use hecs::{PreparedQuery, World};

use crate::{ecs::component::{transform::TransformComponent, model::ModelComponent}, backend::renderer::RendererBackend};

pub type TransferModelTransformsQuery<'a> = PreparedQuery<(&'a TransformComponent, &'a ModelComponent)>;

pub fn system_transfer_model_transforms(
    world: &mut World,
    query: &mut TransferModelTransformsQuery,
    renderer: &mut dyn RendererBackend,
) {
    for (_, (t, m)) in query.query_mut(world) {
        renderer.transfer_model_transform(m.id, t.matrix);
    }
}