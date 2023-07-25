use std::ops::{Index, IndexMut};

use crate::registry::component::ComponentId;

use super::entity::Entity;

pub(crate) type ArchetypeId = usize;
pub(crate) type EntityIndex = usize;

pub(crate) struct Archetype {
    component_count: usize,
    component_start: usize,
    last_edge: Option<ArchetypeEdgeId>,
    entities: Vec<Entity>,
}

impl Archetype {
    fn is_subset_of(&self, other: &Self, components: &[ComponentId]) -> bool {
        let self_ids =
            &components[self.component_start..(self.component_start + self.component_count)];
        let other_ids =
            &components[other.component_start..(other.component_start + other.component_count)];
        for self_id in self_ids {
            if !other_ids.contains(self_id) {
                return false;
            }
        }
        return true;
    }

    fn empty() -> Self {
        Self {
            component_count: 0,
            component_start: 0,
            last_edge: None,
            entities: Vec::new(),
        }
    }

    pub(crate) fn push(&mut self, entity: Entity) -> EntityIndex {
        self.entities.push(entity);
        self.entities.len() - 1
    }

    pub(crate) fn swap_remove(&mut self, index: EntityIndex) {
        self.entities.swap_remove(index);
    }
}

type ArchetypeEdgeId = usize;

struct ArchetypeEdge {
    component: ComponentId,
    add: Option<ArchetypeId>,
    remove: Option<ArchetypeId>,
    previous: Option<ArchetypeEdgeId>,
}

pub(crate) struct ArchetypeTable {
    components: Vec<ComponentId>,
    entries: Vec<Archetype>,
    edges: Vec<ArchetypeEdge>,
}

impl ArchetypeTable {
    pub(crate) fn new() -> Self {
        Self {
            components: Vec::with_capacity(256),
            entries: vec![Archetype::empty()],
            edges: Vec::with_capacity(256),
        }
    }

    fn found_edge(
        &self,
        archetype: ArchetypeId,
        component: ComponentId,
    ) -> Option<ArchetypeEdgeId> {
        let mut current = self.entries[archetype].last_edge;
        while let Some(edge) = current {
            if self.edges[edge].component == component {
                return Some(edge);
            }
            current = self.edges[edge].previous;
        }
        None
    }

    fn link(&mut self, a: ArchetypeId, b: ArchetypeId, component: ComponentId) {
        assert!(a != b);
        // Link a to b (add)
        if let Some(id) = self.found_edge(a, component) {
            self.edges[id].add = Some(b);
        } else {
            let edge = ArchetypeEdge {
                component,
                add: Some(b),
                remove: None,
                previous: self.entries[a].last_edge,
            };
            self.entries[a].last_edge = Some(self.edges.len());
            self.edges.push(edge);
        }
        // Link b to a (remove)
        if let Some(id) = self.found_edge(b, component) {
            self.edges[id].remove = Some(a);
        } else {
            let edge = ArchetypeEdge {
                component,
                add: None,
                remove: Some(a),
                previous: self.entries[b].last_edge,
            };
            self.entries[b].last_edge = Some(self.edges.len());
            self.edges.push(edge);
        }
    }

    fn link_if_previous(&mut self, a: ArchetypeId, b: ArchetypeId) {
        if self.entries[a].component_count == self.entries[b].component_count + 1
            && self.entries[b].is_subset_of(&self.entries[b], &self.components)
        {
            // Found different component
            let a_components = &self.components[self.entries[a].component_start
                ..(self.entries[a].component_start + self.entries[a].component_count)];
            let b_components = &self.components[self.entries[b].component_start
                ..(self.entries[b].component_start + self.entries[b].component_count)];
            if let Some(component) = a_components.iter().enumerate().find_map(|(i, id)| {
                if !b_components.contains(id) {
                    Some(*id)
                } else {
                    None
                }
            }) {
                self.link(b, a, component);
            }
        }
    }

    pub(crate) fn find_add(
        &mut self,
        archetype: ArchetypeId,
        component: ComponentId,
    ) -> ArchetypeId {
        // Find from existing edges
        if let Some(id) = self.found_edge(archetype, component) {
            if let Some(add) = self.edges[id].add {
                return add;
            }
        }
        // Create new archetype
        let component_count = self.entries[archetype].component_count + 1;
        let component_start = self.components.len();
        self.components.extend_from_slice(
            &self.components[self.entries[archetype].component_start
                ..(self.entries[archetype].component_start
                    + self.entries[archetype].component_count)],
        );
        self.components.push(component);
        self.entries.push(Archetype {
            component_count,
            component_start,
            last_edge: None,
            entities: Vec::new(),
        });
        let new_archetype = self.entries.len() - 1;
        // Link new archetype to existing archetypes
        for i in 0..new_archetype {
            self.link_if_previous(i, new_archetype);
        }
        new_archetype
    }

    pub(crate) fn find_remove(
        &mut self,
        archetype: ArchetypeId,
        component: ComponentId,
    ) -> ArchetypeId {
        // Find from existing edges
        if let Some(id) = self.found_edge(archetype, component) {
            if let Some(remove) = self.edges[id].remove {
                return remove;
            }
        }
        // Find brutforce
        let mut next = 0; // Empty archetype
        for i in 0..self.entries[archetype].component_count {
            let id = self.components[self.entries[archetype].component_start + i];
            if id != component {
                next = self.find_add(next, id);
            }
        }
        next
    }

    pub(crate) fn find(&mut self, components: &[ComponentId]) -> ArchetypeId {
        let mut next = 0; // Empty archetype
        for component in components {
            next = self.find_add(next, *component);
        }
        next
    }
}

impl Index<ArchetypeId> for ArchetypeTable {
    type Output = Archetype;

    fn index(&self, index: ArchetypeId) -> &Self::Output {
        &self.entries[index]
    }
}

impl IndexMut<ArchetypeId> for ArchetypeTable {
    fn index_mut(&mut self, index: ArchetypeId) -> &mut Self::Output {
        todo!()
    }
}
