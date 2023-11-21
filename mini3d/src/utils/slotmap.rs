use std::ops::{Index, IndexMut};

pub(crate) trait KeyVersion: Copy + Clone + PartialEq + Eq + Default {
    fn next(&self) -> Self;
}

pub(crate) trait KeyIndex: Copy + Clone + From<usize> {
    fn into_index(self) -> usize;
}

pub trait Key: Copy + Clone + PartialEq + Eq {
    type Version: KeyVersion;
    type Index: KeyIndex;
    fn new(index: Self::Index, version: Self::Version) -> Self;
    fn index(&self) -> Self::Index;
    fn version(&self) -> Self::Version;
    fn null() -> Self;
    fn is_null(&self) -> bool;
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultKeyVersion(u8);

impl KeyVersion for DefaultKeyVersion {
    fn next(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultKeyIndex(u32);

impl From<usize> for DefaultKeyIndex {
    fn from(index: usize) -> Self {
        Self(index as u32)
    }
}

impl KeyIndex for DefaultKeyIndex {
    fn into_index(self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DefaultKey(u32);

impl Default for DefaultKey {
    fn default() -> Self {
        Self(0xFFFFFFFF)
    }
}

impl Key for DefaultKey {
    type Version = DefaultKeyVersion;
    type Index = DefaultKeyIndex;

    fn new(index: Self::Index, version: Self::Version) -> Self {
        Self(index.0 as u32 | ((version.0 as u32) << 24))
    }

    fn index(&self) -> Self::Index {
        DefaultKeyIndex(self.0 & 0xFFFFFF)
    }

    fn version(&self) -> Self::Version {
        DefaultKeyVersion(((self.0 >> 24) & 0xFF) as u8)
    }

    fn null() -> Self {
        Self::default()
    }

    fn is_null(&self) -> bool {
        self.0 & 0xFFFFFF == 0xFFFFFF
    }
}

struct SlotEntry<K: Key, V> {
    value: V,
    key: K,
}

pub struct SlotMap<K: Key, V> {
    entries: Vec<SlotEntry<K, V>>,
    free: K,
}

impl<K: Key, V> Default for SlotMap<K, V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            free: Key::null(),
        }
    }
}

impl<V> SlotMap<DefaultKey, V> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            free: Key::null(),
        }
    }
}

impl<K: Key, V> SlotMap<K, V> {
    pub fn with_key() -> Self {
        Self {
            entries: Vec::new(),
            free: Key::null(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free: Key::null(),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.free = Key::null();
    }

    pub fn add(&mut self, value: V) -> K {
        if self.free.is_null() {
            let index = self.entries.len();
            let key = K::new(index.into(), K::Version::default());
            self.entries.push(SlotEntry { value, key });
            key
        } else {
            let index = self.free.index();
            let entry = &mut self.entries[index.into_index()];
            self.free = entry.key;
            entry.value = value;
            entry.key = K::new(index, entry.key.version());
            entry.key
        }
    }

    pub(crate) fn add_with_version(&mut self, value: V, version: K::Version) -> K {
        if self.free.is_null() {
            let index = self.entries.len();
            let key = K::new(index.into(), version);
            self.entries.push(SlotEntry { value, key });
            key
        } else {
            let index = self.free.index();
            let entry = &mut self.entries[index.into_index()];
            self.free = entry.key;
            entry.value = value;
            entry.key = K::new(index, version);
            entry.key
        }
    }

    pub fn remove(&mut self, key: K) {
        let index = key.index().into_index();
        // Check slot validity
        if index >= self.entries.len() || self.entries[index].key != key {
            return;
        }
        // Mark slot as free and update version
        self.entries[index].key = K::new(self.free.index(), key.version().next());
        // Keep reference to the slot
        self.free = key;
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
        let index = key.index().into_index();
        if index >= self.entries.len() || self.entries[index].key != key {
            None
        } else {
            self.get_unchecked(index)
        }
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let index = key.index().into_index();
        if index >= self.entries.len() || self.entries[index].key != key {
            None
        } else {
            self.get_mut_unchecked(index)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if index == entry.key.index().into_index() {
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
                if index == entry.key.index().into_index() {
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
                if index == entry.key.index().into_index() {
                    Some(entry.key)
                } else {
                    None
                }
            })
    }

    pub fn next(&self, key: K) -> Option<K> {
        let mut index = key.index().into_index() + 1;
        while index < self.entries.len() {
            if self.entries[index].key.index().into_index() == index {
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
                if index == entry.key.index().into_index() {
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
                if index == entry.key.index().into_index() {
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
    slot_to_index: K::Index,
    index_to_slot: K::Index, // or free slot if unused
    version: K::Version,
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
            self.meta[size].slot_to_index = size.into();
            self.meta[size].index_to_slot = free_id;
            self.free_count -= 1;
            K::new(size.into(), self.meta[size].version)
        } else {
            let size = self.data.len();
            self.data.push(value);
            let version = K::Version::default();
            self.meta.push(DenseSlotMapMeta {
                slot_to_index: size.into(),
                index_to_slot: size.into(),
                version,
            });
            K::new(size.into(), version)
        }
    }

    pub fn remove(&mut self, key: K) {
        let last_index = self.data.len() - 1;
        let slot_index = key.index();
        if slot_index < self.meta.len() && self.meta[slot_index].version == key.version() {
            let index = self.meta[slot_index].slot_to_index as usize;
            self.data.swap_remove(index);
            let last_id = self.meta[last_index].index_to_slot;
            self.meta[last_id as usize].slot_to_index = index as u32;
            self.meta[slot_index].version = self.meta[slot_index].version.next();
            self.meta[index].index_to_slot = last_id;
            self.meta[last_index].index_to_slot = slot_index as u32; // free slot
            self.free_count += 1;
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
            .map(|(value, meta)| (K::new(meta.slot_to_index as usize, meta.version), value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn get(&self, key: K) -> Option<&V> {
        let index = key.index();
        if index >= self.meta.len() || self.meta[index].version != key.version() {
            return None;
        }
        self.data.get(self.meta[index].slot_to_index as usize)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        let index = key.index();
        if index >= self.meta.len() || self.meta[index].version != key.version() {
            return None;
        }
        self.data.get_mut(self.meta[index].slot_to_index as usize)
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
        let index = key.index();
        assert!(!key.is_null());
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, Default::default);
        }
        self.entries[index].value = value;
        self.entries[index].key = key;
    }

    pub fn remove(&mut self, key: K) {
        let index = key.index();
        if index >= self.entries.len() || self.entries[index].key != key {
            return;
        }
        self.entries[index].key = K::null();
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.entries.get(key.index()).and_then(|entry| {
            if entry.key.version() != key.version() {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.entries.get_mut(key.index()).and_then(|entry| {
            if entry.key.version() != key.version() {
                None
            } else {
                Some(&mut entry.value)
            }
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
    index: K::Index,
}

pub struct SparseSecondaryMap<K: Key, V> {
    data: Vec<SparseSecondaryMapEntry<K, V>>,
    indices: Vec<u32>,
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
        if key.index() >= self.indices.len() {
            self.indices.resize(key.index() + 1, Default::default());
        }
        let index = key.index();
        if !self.indices[index].is_null() {
            let i = self.indices[index].index();
            self.data[i].value = value;
            self.indices[index] = K::new(i, key.version());
        } else {
            self.indices[index] = K::new(self.data.len(), key.version());
            self.data.push(SparseSecondaryMapEntry {
                value,
                index: key.index(),
            });
        }
    }

    pub fn remove(&mut self, key: K) -> Option<V> {
        if let Some(meta) = self.indices.get(key.index()).copied() {
            if meta.is_null() || meta.version() != key.version() {
                return None;
            } else {
                let index = meta.index();
                let id_index = self.data[index].index;
                self.indices[id_index as usize] = Key::null();
                // Swap with last entry
                if index != self.data.len() - 1 {
                    let last_index = self.data.len() as u32 - 1;
                    let last_id_index = self.data[last_index as usize].index;
                    self.data.swap(index, last_index as usize);
                    self.indices[last_id_index as usize] = meta;
                }
                return self.data.pop().map(|e| e.value);
            }
        }
        None
    }

    pub fn contains(&self, key: K) -> bool {
        self.indices
            .get(key.index())
            .map(|index| !index.is_null() && index.version() == key.version())
            .unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (K, &V)> {
        self.data.iter().map(|e| {
            (
                K::new(e.index, self.indices[e.index as usize].version()),
                &e.value,
            )
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (K, &mut V)> {
        self.data.iter_mut().map(|e| {
            (
                K::new(e.index, self.indices[e.index as usize].version()),
                &mut e.value,
            )
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|e| &e.value)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().map(|e| &mut e.value)
    }

    pub fn get(&self, key: K) -> Option<&V> {
        self.indices.get(key.index()).and_then(|index| {
            if index.is_null() || index.version() != key.version() {
                None
            } else {
                self.data.get(index.index()).map(|e| &e.value)
            }
        })
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut V> {
        self.indices.get(key.index()).and_then(|index| {
            if index.is_null() || index.version() != key.version() {
                None
            } else {
                self.data.get_mut(index.index()).map(|e| &mut e.value)
            }
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
        let mut sm = SlotMap::<u32, u32>::default();
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
        let mut sm = DenseSlotMap::<u32, u32>::default();
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
        let mut sm = SlotMap::<u32, u32>::default();
        let mut ssm = SecondaryMap::<u32, u32>::default();
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
        let mut sm = SlotMap::<u32, u32>::default();
        let mut ssm = SparseSecondaryMap::<u32, u32>::default();
        let s0 = sm.add(1);
        ssm.insert(s0, 1);
        assert_eq!(ssm.get(s0), Some(&1));
        ssm.insert(s0, 2);
        assert_eq!(ssm.get(s0), Some(&2));
        ssm.remove(s0);
        assert!(ssm.get(s0).is_none());
    }
}
