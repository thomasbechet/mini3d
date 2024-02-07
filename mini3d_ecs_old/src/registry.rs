use alloc::vec::Vec;
use mini3d_utils::slotmap::SlotMap;

use crate::{
    bitset::Bitset,
    container::ContainerKey,
    context::Context,
    entity::{Entity, EntityTable},
};

pub(crate) enum RegistryCommand {
    Despawn(Entity),
    AddComponent(Entity, ContainerKey),
    RemoveComponent(Entity, ContainerKey),
}

#[derive(Default)]
pub struct Registry {
    bitsets: SlotMap<ContainerKey, Bitset>,
    commands: Vec<RegistryCommand>,
    entities: EntityTable,
}

impl Registry {
    pub(crate) fn flush(&mut self, ctx: &mut Context) {}

    pub fn spawn(&mut self) -> Entity {
        self.entities.spawn()
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.commands.push(RegistryCommand::Despawn(entity));
    }

    pub fn all(&self, keys: &[ContainerKey]) {}
}
