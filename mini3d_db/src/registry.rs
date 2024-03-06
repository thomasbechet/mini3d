use alloc::vec::Vec;
use mini3d_utils::slotmap::SecondaryMap;

use crate::{
    bitset::{BitIndex, Bitset},
    container::ComponentId,
    entity::{Entity, EntityIndex, EntityVersion},
};

#[derive(Default)]
pub(crate) struct Registry {
    free_entities: Vec<Entity>,
    next_index: EntityIndex, // Default index is 0
    versions: Vec<EntityVersion>,
    entity_map: Bitset,
    bitsets: SecondaryMap<ComponentId, Bitset>,
}

impl Registry {
    pub(crate) fn create(&mut self) -> Entity {
        let entity = if let Some(entity) = self.free_entities.pop() {
            entity
        } else {
            let index = self.next_index;
            self.next_index += 1;
            Entity::new(index, 0)
        };
        self.entity_map.set(entity.index(), true);
        if entity.index() as usize >= self.versions.len() {
            self.versions.resize(entity.index() as usize + 1, 0);
        }
        self.versions[entity.index() as usize] = entity.version();
        entity
    }

    pub(crate) fn destroy(&mut self, entity: Entity) {
        self.entity_map.set(entity.index(), false);
        self.free_entities.push(Entity::new(
            entity.index(),
            entity.version().wrapping_add(1),
        ));
    }

    pub(crate) fn add_bitset(&mut self, id: ComponentId) {
        self.bitsets.insert(id, Bitset::default());
    }

    pub(crate) fn remove_bitset(&mut self, id: ComponentId) {
        self.bitsets.remove(id);
    }

    pub(crate) fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entity_map
            .iter()
            .map(move |index| Entity::new(index, self.versions[index as usize]))
    }

    pub(crate) fn entity_version(&self, index: EntityIndex) -> EntityVersion {
        self.versions[index as usize]
    }

    pub(crate) fn has(&self, e: Entity, c: ComponentId) -> bool {
        self.bitsets
            .get(c)
            .map(|bitset| bitset.is_set(e.index() as BitIndex))
            .unwrap_or(false)
    }

    pub(crate) fn find_next_component(&self, e: Entity, mut c: ComponentId) -> Option<ComponentId> {
        while let Some(n) = self.bitsets.next(c) {
            if self.has(e, n) {
                return Some(n);
            }
            c = n;
        }
        None
    }

    pub(crate) fn set(&mut self, e: Entity, c: ComponentId) {
        self.bitsets
            .get_mut(c)
            .unwrap()
            .set(e.index() as BitIndex, true);
    }

    pub(crate) fn unset(&mut self, e: Entity, c: ComponentId) {
        self.bitsets
            .get_mut(c)
            .unwrap()
            .set(e.index() as BitIndex, false);
    }

    pub(crate) fn mask(&self, c: ComponentId, index: usize) -> u32 {
        self.bitsets[c].mask(index)
    }
}
