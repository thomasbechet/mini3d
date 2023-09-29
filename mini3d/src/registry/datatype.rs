use crate::{
    asset::reference::AssetRef, ecs::entity::Entity, reflection::Reflect, serialize::Serialize,
};

pub struct ReferenceResolver {}

impl ReferenceResolver {
    pub(crate) fn resolve_entity(&self, entity: Entity) -> Entity {
        // TODO: resolve entity or log error ? panic in runtime ?
        entity
    }

    pub(crate) fn resolve_asset(&self, reference: AssetRef) -> AssetRef {
        // TODO: resolve asse tor log error ? panic in runtime ?
        reference
    }
}

pub trait StaticDataType: Default + Serialize + Reflect + 'static {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {}
}
