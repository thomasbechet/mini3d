use std::ops::{Index, IndexMut};

// A slot map key has two state:
// - Null key:
// If the key is null, None must be returned when index() is called.
// Comparing with null with null return true.
// - Valid key:
// If the key is valid, Some(index) must be returned when index() is called.
// A valid key is given a version that must be changed when updated is called.
pub trait Key: Copy + Clone + PartialEq + Eq {
    fn new(index: usize) -> Self;
    fn null() -> Self;
    fn update(&mut self, index: usize);
    fn index(&self) -> Option<usize>;
    fn is_null(&self) -> bool {
        self.index().is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultKey(u32);

impl Default for DefaultKey {
    fn default() -> Self {
        Self::null()
    }
}

impl Key for DefaultKey {
    fn new(index: usize) -> Self {
        Self((index & 0xFFFFFF) as u32)
    }

    fn null() -> Self {
        Self(0xFFFFFFFF)
    }

    fn update(&mut self, index: usize) {
        let mut version = ((self.0 >> 24) & 0xFF) as u8;
        version = version.wrapping_add(1);
        self.0 = (index & 0xFFFFFF) as u32 | ((version as u32) << 24);
    }

    fn index(&self) -> Option<usize> {
        if self.is_null() {
            None
        } else {
            Some((self.0 & 0xFFFFFF) as usize)
        }
    }
}

struct SlotEntry<K: Key, V> {
    value: V,
    key: K,
}

pub struct SlotMap<K: Key, V> {
    entries: Vec<SlotEntry<K, V>>,
    free: Option<usize>,
}

impl<K: Key, V> Default for SlotMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            free: None,
        }
    }
}

impl<V> SlotMap<DefaultKey, V> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: None,
        }
    }
}

impl<K: Key, V> SlotMap<K, V> {
    pub fn with_key() -> Self {
        Self {
            entries: Vec::new(),
            free: None,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free: None,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.free = None;
    }

    pub fn add(&mut self, value: V) -> K {
        if let Some(free) = self.free {
            let entry = &mut self.entries[free];
            self.free = entry.key.index();
            entry.value = value;
            entry.key.update(free);
            entry.key
        } else {
            let index = self.entries.len();
            let key = K::new(index);
            self.entries.push(SlotEntry { value, key });
            key
        }
    }

    pub fn remove(&mut self, key: K) {
        if let Some(index) = key.index() {
            // Check slot validity
            if index >= self.entries.len() || self.entries[index].key != key {
                return;
            }
            // Mark slot as free and update version
            if let Some(free) = self.free {
                self.entries[index].key.update(free);
            } else {
                self.entries[index].key.update(usize::MAX);
            }
            // Keep reference to the slot
            self.free = Some(index);
        }
    }

    pub fn contains(&self, key: K) -> bool {
        self.get(key).is_some()
    }

    fn get_unchecked(&self, index: usize) -> Option<&V> {
        Some(&self.entries[index].value)
    }

    fn get_mut_unchecked(&mut self, index: usize) -> Option<&mut V> {
        Some(&mut self.entries[index].value)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        key.index().and_then(|index| {
            if index >= self.entries.len() || self.entries[index].key != key {
                None
            } else {
                self.get_unchecked(index)
            }
        })
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        key.index().and_then(|index| {
            if index >= self.entries.len() || self.entries[index].key != key {
                None
            } else {
                self.get_mut_unchecked(index)
            }
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.key.index().is_some() && index == entry.key.index().unwrap() {
                    Some((entry.key, &entry.value))
                } else {
                    None
                }
            })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.key.index().is_some() && index == entry.key.index().unwrap() {
                    Some((entry.key, &mut entry.value))
                } else {
                    None
                }
            })
    }

    pub fn keys(&self) -> impl Iterator<Item = K> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.key.index().is_some() && index == entry.key.index().unwrap() {
                    Some(entry.key)
                } else {
                    None
                }
            })
    }

    pub fn next(&self, key: K) -> Option<K> {
        let mut index = key.index().unwrap() + 1;
        while index < self.entries.len() {
            if self.entries[index].key.index().is_some()
                && self.entries[index].key.index().unwrap() == index
            {
                return Some(self.entries[index].key);
            }
            index += 1;
        }
        None
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.key.index().is_some() && index == entry.key.index().unwrap() {
                    Some(&entry.value)
                } else {
                    None
                }
            })
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.entries
            .iter_mut()
            .enumerate()
            .filter_map(|(index, entry)| {
                if entry.key.index().is_some() && index == entry.key.index().unwrap() {
                    Some(&mut entry.value)
                } else {
                    None
                }
            })
    }
}

impl<K: Key, V> Index<K> for SlotMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K: Key, V> IndexMut<K> for SlotMap<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

#[derive(Debug)]
struct DenseSlotMapMeta<K: Key> {
    slot_to_index: u32,
    index_to_slot: u32, // or free slot if unused
    key: K,
}

pub struct DenseSlotMap<K: Key, V> {
    data: Vec<V>,
    meta: Vec<DenseSlotMapMeta<K>>,
    free_count: u32,
}

impl<K: Key, V> DenseSlotMap<K, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            meta: Vec::with_capacity(capacity),
            free_count: 0,
        }
    }

    pub fn add(&mut self, value: V) -> K {
        if self.free_count > 0 {
            let size = self.data.len();
            self.data.push(value);
            let free_id = self.meta[size].index_to_slot;
            self.meta[size].slot_to_index = size as u32;
            self.meta[size].index_to_slot = free_id;
            self.meta[size].key.update(size);
            self.free_count -= 1;
            self.meta[size].key
        } else {
            let size = self.data.len();
            self.data.push(value);
            let key = K::new(size);
            self.meta.push(DenseSlotMapMeta {
                slot_to_index: size as u32,
                index_to_slot: size as u32,
                key,
            });
            key
        }
    }

    pub fn remove(&mut self, key: K) {
        let last_index = self.data.len() - 1;
        if let Some(slot_index) = key.index() {
            if slot_index < self.meta.len() && self.meta[slot_index].key == key {
                let index = self.meta[slot_index].slot_to_index as usize;
                self.data.swap_remove(index);
                let last_id = self.meta[last_index].index_to_slot;
                self.meta[last_id as usize].slot_to_index = index as u32;
                self.meta[slot_index].key.update(slot_index);
                self.meta[index].index_to_slot = last_id;
                self.meta[last_index].index_to_slot = slot_index as u32; // free slot
                self.free_count += 1;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.data
            .iter()
            .zip(self.meta.iter())
            .map(|(value, meta)| (meta.key, value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn get(&self, key: K) -> Option<&V> {
        key.index().and_then(|index| {
            if index >= self.meta.len() || self.meta[index].key != key {
                return None;
            }
            self.data.get(self.meta[index].slot_to_index as usize)
        })
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        if let Some(index) = key.index() {
            if index >= self.meta.len() || self.meta[index].key != key {
                return None;
            }
            return self.data.get_mut(self.meta[index].slot_to_index as usize);
        }
        None
    }

    pub fn contains(&self, key: K) -> bool {
        self.get(key).is_some()
    }
}

impl<K: Key, V> Default for DenseSlotMap<K, V> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<K: Key, V> Index<K> for DenseSlotMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K: Key, V> IndexMut<K> for DenseSlotMap<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

struct SecondarySlotEntry<K: Key, V: Default> {
    value: V,
    key: K,
}

impl<K: Key, V: Default> Default for SecondarySlotEntry<K, V> {
    fn default() -> Self {
        Self {
            value: V::default(),
            key: K::null(),
        }
    }
}

pub struct SecondaryMap<K: Key, V: Default> {
    entries: Vec<SecondarySlotEntry<K, V>>,
}

impl<K: Key, V: Default> SecondaryMap<K, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(index) = key.index() {
            if index >= self.entries.len() {
                self.entries.resize_with(index + 1, Default::default);
            }
            self.entries[index].value = value;
            self.entries[index].key = key;
        }
    }

    pub fn remove(&mut self, key: K) {
        if let Some(index) = key.index() {
            if index >= self.entries.len() || self.entries[index].key != key {
                return;
            }
            self.entries[index].key.update(usize::MAX);
        }
    }

    pub fn get(&self, key: K) -> Option<&V> {
        key.index().and_then(|index| {
            self.entries.get(index).and_then(|entry| {
                if entry.key != key {
                    None
                } else {
                    Some(&entry.value)
                }
            })
        })
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        key.index().and_then(|index| {
            self.entries.get_mut(index).and_then(|entry| {
                if entry.key != key {
                    None
                } else {
                    Some(&mut entry.value)
                }
            })
        })
    }

    pub fn contains(&self, key: K) -> bool {
        self.get(key).is_some()
    }
}

impl<K: Key, V: Default> Default for SecondaryMap<K, V> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<K: Key, V: Default> Index<K> for SecondaryMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K: Key, V: Default> IndexMut<K> for SecondaryMap<K, V> {
    fn index_mut(&mut self, slot: K) -> &mut Self::Output {
        self.get_mut(slot).unwrap()
    }
}

struct SparseSecondaryMapEntry<K: Key, V> {
    value: V,
    key: K,
}

pub struct SparseSecondaryMap<K: Key, V> {
    data: Vec<SparseSecondaryMapEntry<K, V>>,
    indices: Vec<Option<usize>>,
}

impl<K: Key, V> Default for SparseSecondaryMap<K, V> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl<K: Key, V> SparseSecondaryMap<K, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.indices.clear();
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn insert(&mut self, key: K, value: V) {
        if let Some(index) = key.index() {
            if index >= self.indices.len() {
                self.indices.resize(index + 1, None);
            }
            if let Some(i) = self.indices[index] {
                self.data[i].value = value;
                self.data[i].key = key;
                self.indices[index] = Some(i);
            } else {
                self.indices[index] = Some(self.data.len());
                self.data.push(SparseSecondaryMapEntry { value, key });
            }
        }
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Some(id_index) = key.index() {
            if let Some(meta) = self.indices.get(id_index).copied() {
                if let Some(index) = meta {
                    if self.data[index].key != key {
                        return None;
                    }
                    self.indices[id_index] = None;
                    // Swap with last entry
                    if index != self.data.len() - 1 {
                        let last_index = self.data.len() as u32 - 1;
                        let last_id_index = self.data[last_index as usize].key.index().unwrap();
                        self.data.swap(index, last_index as usize);
                        self.indices[last_id_index as usize] = meta;
                    }
                    return self.data.pop().map(|e| e.value);
                }
            }
        }

        None
    }

    pub fn contains(&self, key: K) -> bool {
        if key.index().is_none() {
            return false;
        }
        let meta = self.indices[key.index().unwrap()];
        meta.is_some() && self.data[meta.unwrap()].key == key
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.data.iter().map(|e| (e.key, &e.value))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.data.iter_mut().map(|e| (e.key, &mut e.value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|e| &e.value)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().map(|e| &mut e.value)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        key.index().and_then(|index| {
            self.indices.get(index).and_then(|index| {
                if let Some(index) = index {
                    if self.data[*index].key == key {
                        return Some(&self.data[*index].value);
                    }
                }
                None
            })
        })
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        key.index().and_then(|index| {
            self.indices.get_mut(index).and_then(|index| {
                if let Some(index) = index {
                    if self.data[*index].key == key {
                        return Some(&mut self.data[*index].value);
                    }
                }
                None
            })
        })
    }
}

impl<K: Key, V> Index<K> for SparseSecondaryMap<K, V> {
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K: Key, V> IndexMut<K> for SparseSecondaryMap<K, V> {
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(key).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slotmap() {
        let mut sm = SlotMap::<DefaultKey, u32>::default();
        let s0 = sm.add(1);
        sm.add(0);
        sm.add(0);
        sm.add(0);
        assert_eq!(sm.get(s0), Some(&1));
        sm.remove(s0);
        assert!(sm.get(s0).is_none());
        let s1 = sm.add(2);
        assert_eq!(sm.get(s1), Some(&2));
        assert!(!sm.contains(s0));
        sm.clear();
        assert!(!sm.contains(s1));
    }

    #[test]
    fn test_dense_slotmap() {
        let mut sm = DenseSlotMap::<DefaultKey, u32>::default();
        let s0 = sm.add(1);
        sm.add(0);
        sm.add(0);
        sm.add(0);
        assert_eq!(sm.get(s0), Some(&1));
        sm.remove(s0);
        assert!(!sm.contains(s0));
        let s1 = sm.add(2);
        assert_eq!(sm.get(s1), Some(&2));
        assert!(!sm.contains(s0));
    }

    #[test]
    fn test_secondary_map() {
        let mut sm = SlotMap::<DefaultKey, u32>::default();
        let mut ssm = SecondaryMap::<DefaultKey, u32>::default();
        let s0 = sm.add(1);
        ssm.insert(s0, 1);
        assert_eq!(ssm.get(s0), Some(&1));
        ssm.insert(s0, 2);
        assert_eq!(ssm.get(s0), Some(&2));
        ssm.remove(s0);
        assert!(ssm.get(s0).is_none());
    }

    #[test]
    fn test_sparse_secondary_map() {
        let mut sm = SlotMap::<DefaultKey, u32>::default();
        let mut ssm = SparseSecondaryMap::<DefaultKey, u32>::default();
        let s0 = sm.add(1);
        ssm.insert(s0, 1);
        assert_eq!(ssm.get(s0), Some(&1));
        ssm.insert(s0, 2);
        assert_eq!(ssm.get(s0), Some(&2));
        ssm.remove(s0);
        assert!(ssm.get(s0).is_none());
    }
}
