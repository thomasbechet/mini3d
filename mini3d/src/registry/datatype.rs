use crate::{
    ecs::entity::Entity,
    reflection::Reflect,
    serialize::Serialize,
    utils::{slotmap::SlotId, uid::UID},
};

pub struct ReferenceResolver {}

impl ReferenceResolver {
    pub(crate) fn remap_entity(&self, entity: Entity) -> Entity {
        // TODO: resolve entity or log error ? panic in runtime ?
        entity
    }

    pub(crate) fn resolve_resource_id(&self, uid: UID) -> SlotId {
        // TODO: resolve asse tor log error ? panic in runtime ?
        SlotId::null()
    }

    pub(crate) fn remap_resource_key(&self, id: SlotId) -> UID {
        UID::null()
    }
}

pub trait StaticDataType: Default + Serialize + Reflect + 'static {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {}
}
