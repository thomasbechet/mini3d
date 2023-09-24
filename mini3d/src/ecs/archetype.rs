use std::ops::{Index, IndexMut, Range};

use crate::{
    registry::component::ComponentId,
    utils::slotmap::{SlotId, SlotMap},
};

use super::{
    entity::Entity,
    query::{query_archetype_match, FilterKind, FilterQuery, QueryTable},
};

pub(crate) type Archetype = SlotId;

#[derive(Debug)]
pub(crate) struct ArchetypeEntry {
    pub(crate) component_range: Range<usize>,
    last_edge: Option<ArchetypeEdgeId>,
    pub(crate) added_filter_queries: Vec<FilterQuery>,
    pub(crate) removed_filter_queries: Vec<FilterQuery>,
    pub(crate) pool: Vec<Entity>,
}

impl ArchetypeEntry {
    fn is_subset_of(&self, other: &Self, components: &[ComponentId]) -> bool {
        let self_ids = &components[self.component_range.clone()];
        let other_ids = &components[other.component_range.clone()];
        for self_id in self_ids {
            if !other_ids.contains(self_id) {
                return false;
            }
        }
        true
    }

    fn empty() -> Self {
        Self {
            component_range: 0..0,
            last_edge: None,
            added_filter_queries: Vec::new(),
            removed_filter_queries: Vec::new(),
            pool: Default::default(),
        }
    }
}

type ArchetypeEdgeId = usize;

#[derive(Debug)]
struct ArchetypeEdge {
    component: ComponentId,
    add: Option<Archetype>,
    remove: Option<Archetype>,
    previous: Option<ArchetypeEdgeId>,
}

pub(crate) struct ArchetypeTable {
    pub(crate) components: Vec<ComponentId>,
    pub(crate) entries: SlotMap<ArchetypeEntry>,
    edges: Vec<ArchetypeEdge>,
    pub(crate) empty: Archetype,
}

impl ArchetypeTable {
    pub(crate) fn new() -> Self {
        let mut table = Self {
            components: Vec::with_capacity(256),
            entries: SlotMap::default(),
            edges: Vec::with_capacity(256),
            empty: SlotId::null(),
        };
        table.empty = table.entries.add(ArchetypeEntry::empty());
        table
    }

    fn found_edge(&self, archetype: Archetype, component: ComponentId) -> Option<ArchetypeEdgeId> {
        let mut current = self.entries[archetype].last_edge;
        while let Some(edge) = current {
            if self.edges[edge].component == component {
                return Some(edge);
            }
            current = self.edges[edge].previous;
        }
        None
    }

    fn link(&mut self, a: Archetype, b: Archetype, component: ComponentId) {
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

    fn link_if_previous(&mut self, a: Archetype, b: Archetype) {
        // Check if previous
        if self.entries[a].component_range.len() == self.entries[b].component_range.len() + 1
            && self.entries[b].is_subset_of(&self.entries[b], &self.components)
        {
            // Found different component
            let a_components = &self.components[self.entries[a].component_range.clone()];
            let b_components = &self.components[self.entries[b].component_range.clone()];
            if let Some(component) = a_components.iter().find_map(|id| {
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
        queries: &mut QueryTable,
        archetype: Archetype,
        component: ComponentId,
    ) -> Archetype {
        // Find from existing edges
        if let Some(id) = self.found_edge(archetype, component) {
            if let Some(add) = self.edges[id].add {
                return add;
            }
        }
        // Create new archetype
        let archetype = self.entries.get(archetype).unwrap();
        let component_start = self.components.len();
        for i in archetype.component_range.clone() {
            self.components.push(self.components[i]);
        }
        self.components.push(component);
        let new_archetype = self.entries.add(ArchetypeEntry {
            component_range: component_start..self.components.len(),
            last_edge: None,
            added_filter_queries: Default::default(),
            removed_filter_queries: Default::default(),
            pool: Default::default(),
        });
        // Link new archetype to existing archetypes
        let mut slot = self.entries.keys().next();
        while let Some(current) = slot {
            if current != new_archetype {
                self.link_if_previous(new_archetype, current);
            }
            slot = self.entries.next(current);
        }
        // Bind existing queries to new archetype
        for (query_id, query_entry) in queries.entries.iter_mut() {
            let archetype_entry = &self.entries[new_archetype];
            if query_archetype_match(
                query_entry,
                &queries.components,
                archetype_entry,
                &self.components,
            ) {
                query_entry.archetypes.push(new_archetype);
            }
        }
        // Bind new archetype to existing filter queries
        for (filter_id, filter_query_entry) in queries.filter_queries.iter() {
            let archetype_entry = &mut self.entries[new_archetype];
            if query_archetype_match(
                &queries.entries[filter_query_entry.query.0],
                &queries.components,
                archetype_entry,
                &self.components,
            ) {
                match filter_query_entry.kind {
                    FilterKind::Added => {
                        archetype_entry
                            .added_filter_queries
                            .push(FilterQuery(filter_id));
                    }
                    FilterKind::Removed => {
                        archetype_entry
                            .removed_filter_queries
                            .push(FilterQuery(filter_id));
                    }
                    _ => todo!(),
                }
            }
        }
        new_archetype
    }

    // pub(crate) fn find_remove(
    //     &mut self,
    //     archetype: ArchetypeId,
    //     component: ComponentId,
    // ) -> ArchetypeId {
    //     // Find from existing edges
    //     if let Some(id) = self.found_edge(archetype, component) {
    //         if let Some(remove) = self.edges[id].remove {
    //             return remove;
    //         }
    //     }
    //     // Find brutforce
    //     let mut next = self.empty;
    //     for i in 0..self.entries[archetype].component_count {
    //         let id = self.components[self.entries[archetype].component_start + i];
    //         if id != component {
    //             next = self.find_add(next, id);
    //         }
    //     }
    //     next
    // }

    // pub(crate) fn find(&mut self, components: &[ComponentId]) -> ArchetypeId {
    //     let mut next = self.empty;
    //     for component in components {
    //         next = self.find_add(next, *component);
    //     }
    //     next
    // }

    pub(crate) fn components(&self, archetype: Archetype) -> &[ComponentId] {
        let archetype = &self.entries[archetype];
        &self.components[archetype.component_range.clone()]
    }

    // pub(crate) fn collect_unique_childs(
    //     &self,
    //     archetype: ArchetypeId,
    //     list: &mut Vec<ArchetypeId>,
    // ) {
    //     let mut current = self.entries[archetype].last_edge;
    //     while let Some(edge) = current {
    //         if let Some(add) = self.edges[edge].add {
    //             if !list.contains(&add) {
    //                 list.push(add);
    //                 self.collect_unique_childs(add, list);
    //             }
    //         }
    //         current = self.edges[edge].previous;
    //     }
    // }

    // pub(crate) fn collect_unique_parents(
    //     &self,
    //     archetype: ArchetypeId,
    //     list: &mut Vec<ArchetypeId>,
    // ) {
    //     let mut current = self.entries[archetype].last_edge;
    //     while let Some(edge) = current {
    //         if let Some(remove) = self.edges[edge].remove {
    //             if remove != self.empty && !list.contains(&remove) {
    //                 list.push(remove);
    //                 self.collect_unique_parents(remove, list);
    //             }
    //         }
    //         current = self.edges[edge].previous;
    //     }
    // }
}

impl Index<Archetype> for ArchetypeTable {
    type Output = ArchetypeEntry;

    fn index(&self, id: Archetype) -> &Self::Output {
        self.entries.get(id).unwrap()
    }
}

impl IndexMut<Archetype> for ArchetypeTable {
    fn index_mut(&mut self, id: Archetype) -> &mut Self::Output {
        self.entries.get_mut(id).unwrap()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_archetype_table() {
        use super::*;
        let mut archetypes = ArchetypeTable::new();
        assert!(!archetypes.empty.is_null());
        let mut components = SlotMap::default();
        let a = ComponentId(components.add(()));
        let b = ComponentId(components.add(()));
        let c = ComponentId(components.add(()));
        // archetypes.find(&[a]);
        // let archa = archetypes.find(&[a, b]);
        for (id, archetype) in archetypes.entries.iter() {
            println!("{:?} {:?}", id, archetype);
        }
        println!("{:?}", archetypes.components);
        for edge in archetypes.edges {
            println!("{:?}", edge);
        }
    }
}
