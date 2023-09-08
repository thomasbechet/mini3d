use std::ops::{Index, IndexMut};

use mini3d_derive::Serialize;

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SlotVersion(u8);

impl SlotVersion {
    pub fn next(&self) -> Self {
        Self(self.0.wrapping_add(1))
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq, Serialize)]
pub struct SlotId(u32);

impl SlotId {
    fn new(index: usize, version: SlotVersion) -> Self {
        Self(index as u32 | ((version.0 as u32) << 24))
    }

    fn index(&self) -> usize {
        (self.0 & 0x00ff_ffff) as usize
    }

    fn version(&self) -> SlotVersion {
        SlotVersion((self.0 >> 24) as u8)
    }

    pub fn null() -> Self {
        Self(0xffff_ffff)
    }

    pub fn is_null(&self) -> bool {
        self.0 & 0x00ff_ffff == 0x00ff_ffff
    }
}

impl Default for SlotId {
    fn default() -> Self {
        Self::null()
    }
}

struct SlotEntry<V> {
    value: V,
    slot: SlotId,
}

pub struct SlotMap<V> {
    entries: Vec<SlotEntry<V>>,
    free: SlotId,
}

impl<V> Default for SlotMap<V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            free: SlotId::null(),
        }
    }
}

impl<V> SlotMap<V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
            free: SlotId::null(),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.free = SlotId::null();
    }

    pub fn add(&mut self, value: V) -> SlotId {
        if self.free.is_null() {
            let index = self.entries.len();
            let slot = SlotId::new(index, SlotVersion::default());
            self.entries.push(SlotEntry { value, slot });
            slot
        } else {
            let index = self.free.index();
            let entry = &mut self.entries[index];
            self.free = entry.slot;
            entry.value = value;
            entry.slot = SlotId::new(index, entry.slot.version());
            entry.slot
        }
    }

    pub fn remove(&mut self, slot: SlotId) {
        let index = slot.index();
        // Check slot validity
        if index >= self.entries.len() || self.entries[index].slot != slot {
            return;
        }
        // Mark slot as free and update version
        self.entries[index].slot = SlotId::new(self.free.index(), slot.version().next());
        // Keep reference to the slot
        self.free = slot;
    }

    pub fn contains(&self, slot: SlotId) -> bool {
        self.get(slot).is_some()
    }

    fn get_unchecked(&self, index: usize) -> Option<&V> {
        Some(&self.entries[index].value)
    }

    fn get_mut_unchecked(&mut self, index: usize) -> Option<&mut V> {
        Some(&mut self.entries[index].value)
    }

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        let index = slot.index();
        if index >= self.entries.len() || self.entries[index].slot != slot {
            None
        } else {
            self.get_unchecked(index)
        }
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        let index = slot.index();
        if index >= self.entries.len() || self.entries[index].slot != slot {
            None
        } else {
            self.get_mut_unchecked(index)
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if index == entry.slot.index() {
                    Some((entry.slot, &entry.value))
                } else {
                    None
                }
            })
    }

    pub fn keys(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| {
                if index == entry.slot.index() {
                    Some(entry.slot)
                } else {
                    None
                }
            })
    }

    pub fn next(&self, slot: SlotId) -> Option<SlotId> {
        let mut index = slot.index() + 1;
        while index < self.entries.len() {
            if self.entries[index].slot.index() == index {
                return Some(self.entries[index].slot);
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
                if index == entry.slot.index() {
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
                if index == entry.slot.index() {
                    Some(&mut entry.value)
                } else {
                    None
                }
            })
    }
}

impl<V> Index<SlotId> for SlotMap<V> {
    type Output = V;

    fn index(&self, slot: SlotId) -> &Self::Output {
        self.get(slot).unwrap()
    }
}

impl<V> IndexMut<SlotId> for SlotMap<V> {
    fn index_mut(&mut self, slot: SlotId) -> &mut Self::Output {
        self.get_mut(slot).unwrap()
    }
}

// impl<V: Serialize> Serialize for SlotMap<V> {
//     type Header = V::Header;

//     fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
//         encoder.write_u32(self.entries.len() as u32)?;
//         for entry in &self.entries {
//             match entry {
//                 SlotEntry::Value(value) => {
//                     encoder.write_byte(1)?; // Has value
//                     value.serialize(encoder)?;
//                 }
//                 SlotEntry::Free(free) => {
//                     encoder.write_byte(0)?; // Has no value
//                     free.serialize(encoder)?;
//                 }
//             }
//         }
//         self.free.serialize(encoder)?;
//         Ok(())
//     }

//     fn deserialize(
//         decoder: &mut impl Decoder,
//         header: &Self::Header,
//     ) -> Result<Self, DecoderError> {
//         let len = decoder.read_u32()? as usize;
//         let mut map = SlotMap::with_capacity(len);
//         for _ in 0..len {
//             let has_value = decoder.read_byte()? != 0;
//             if has_value {
//                 let value = V::deserialize(decoder, header)?;
//                 map.entries.push(SlotEntry::Value(value));
//             } else {
//                 let free = SlotId::deserialize(decoder, &Default::default())?;
//                 map.entries.push(SlotEntry::Free(free));
//             }
//         }
//         map.free = SlotId::deserialize(decoder, &Default::default())?;
//         Ok(map)
//     }
// }

#[derive(Debug)]
struct DenseSlotMapMeta {
    slot_to_index: u32,
    index_to_slot: u32, // or free slot if unused
    version: SlotVersion,
}

pub struct DenseSlotMap<V> {
    data: Vec<V>,
    meta: Vec<DenseSlotMapMeta>,
    free_count: u32,
}

impl<V> DenseSlotMap<V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            meta: Vec::with_capacity(capacity),
            free_count: 0,
        }
    }

    pub fn add(&mut self, value: V) -> SlotId {
        if self.free_count > 0 {
            let size = self.data.len();
            self.data.push(value);
            let free_id = self.meta[size].index_to_slot;
            self.meta[size].slot_to_index = size as u32;
            self.meta[size].index_to_slot = free_id;
            self.free_count -= 1;
            SlotId::new(size, self.meta[size].version)
        } else {
            let size = self.data.len();
            self.data.push(value);
            let version = SlotVersion::default();
            self.meta.push(DenseSlotMapMeta {
                slot_to_index: size as u32,
                index_to_slot: size as u32,
                version,
            });
            SlotId::new(size, version)
        }
    }

    pub fn remove(&mut self, slot: SlotId) {
        let last_index = self.data.len() - 1;
        let slot_index = slot.index();
        if slot_index < self.meta.len() && self.meta[slot_index].version == slot.version() {
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

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.data.iter().zip(self.meta.iter()).map(|(value, meta)| {
            (
                SlotId::new(meta.slot_to_index as usize, meta.version),
                value,
            )
        })
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        let index = slot.index();
        if index >= self.meta.len() || self.meta[index].version != slot.version() {
            return None;
        }
        self.data.get(self.meta[index].slot_to_index as usize)
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        let index = slot.index();
        if index >= self.meta.len() || self.meta[index].version != slot.version() {
            return None;
        }
        self.data.get_mut(self.meta[index].slot_to_index as usize)
    }

    pub fn contains(&self, slot: SlotId) -> bool {
        self.get(slot).is_some()
    }
}

impl<V> Default for DenseSlotMap<V> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<V> Index<SlotId> for DenseSlotMap<V> {
    type Output = V;

    fn index(&self, slot: SlotId) -> &Self::Output {
        self.get(slot).unwrap()
    }
}

impl<V> IndexMut<SlotId> for DenseSlotMap<V> {
    fn index_mut(&mut self, slot: SlotId) -> &mut Self::Output {
        self.get_mut(slot).unwrap()
    }
}

// impl<V: Serialize> Serialize for DenseSlotMap<V> {
//     type Header = V::Header;

//     fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
//         encoder.write_u32(self.data.len() as u32)?;
//         encoder.write_u32(self.free_count)?;
//         for (value, meta) in self.data.iter().zip(self.meta.iter()) {
//             value.serialize(encoder)?;
//             encoder.write_u32(meta.id_to_index)?;
//             encoder.write_u32(meta.index_to_id)?;
//         }
//         Ok(())
//     }

//     fn deserialize(
//         decoder: &mut impl Decoder,
//         header: &Self::Header,
//     ) -> Result<Self, DecoderError> {
//         let len = decoder.read_u32()? as usize;
//         let free_count = decoder.read_u32()?;
//         let mut data = Vec::with_capacity(len);
//         let mut meta = Vec::with_capacity(len);
//         for _ in 0..len {
//             let value = V::deserialize(decoder, header)?;
//             let id_to_index = decoder.read_u32()?;
//             let index_to_id = decoder.read_u32()?;
//             data.push(value);
//             meta.push(DenseSlotMapMeta {
//                 id_to_index,
//                 index_to_id,
//             });
//         }
//         Ok(Self {
//             data,
//             meta,
//             free_count,
//         })
//     }
// }

#[derive(Default)]
struct SecondarySlotEntry<V: Default> {
    value: V,
    slot: SlotId,
}

pub struct SecondaryMap<V: Default> {
    entries: Vec<SecondarySlotEntry<V>>,
}

impl<V: Default> SecondaryMap<V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn insert(&mut self, slot: SlotId, value: V) {
        let index = slot.index();
        assert!(!slot.is_null());
        if index >= self.entries.len() {
            self.entries.resize_with(index + 1, Default::default);
        }
        self.entries[index].value = value;
        self.entries[index].slot = slot;
    }

    pub fn remove(&mut self, slot: SlotId) {
        let index = slot.index();
        if index >= self.entries.len() || self.entries[index].slot != slot {
            return;
        }
        self.entries[index].slot = SlotId::null();
    }

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        self.entries.get(slot.index()).and_then(|entry| {
            if entry.slot.version() != slot.version() {
                None
            } else {
                Some(&entry.value)
            }
        })
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        self.entries.get_mut(slot.index()).and_then(|entry| {
            if entry.slot.version() != slot.version() {
                None
            } else {
                Some(&mut entry.value)
            }
        })
    }

    pub fn contains(&self, slot: SlotId) -> bool {
        self.get(slot).is_some()
    }
}

impl<V: Default> Default for SecondaryMap<V> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<V: Default> Index<SlotId> for SecondaryMap<V> {
    type Output = V;

    fn index(&self, slot: SlotId) -> &Self::Output {
        self.get(slot).unwrap()
    }
}

impl<V: Default> IndexMut<SlotId> for SecondaryMap<V> {
    fn index_mut(&mut self, slot: SlotId) -> &mut Self::Output {
        self.get_mut(slot).unwrap()
    }
}

struct SparseSecondaryMapEntry<V> {
    value: V,
    index: u32,
}

pub struct SparseSecondaryMap<V> {
    data: Vec<SparseSecondaryMapEntry<V>>,
    indices: Vec<SlotId>,
}

impl<V> Default for SparseSecondaryMap<V> {
    fn default() -> Self {
        Self {
            data: Vec::new(),
            indices: Vec::new(),
        }
    }
}

impl<V> SparseSecondaryMap<V> {
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

    pub fn insert(&mut self, slot: SlotId, value: V) {
        if slot.index() >= self.indices.len() {
            self.indices.resize(slot.index() + 1, SlotId::null());
        }
        let index = slot.index();
        if !self.indices[index].is_null() {
            let i = self.indices[index].index();
            self.data[i].value = value;
            self.indices[index] = SlotId::new(i, slot.version());
        } else {
            self.indices[index] = SlotId::new(self.data.len(), slot.version());
            self.data.push(SparseSecondaryMapEntry {
                value,
                index: slot.index() as u32,
            });
        }
    }

    pub fn remove(&mut self, slot: SlotId) -> Option<V> {
        if let Some(meta) = self.indices.get(slot.index()).copied() {
            if meta.is_null() || meta.version() != slot.version() {
                return None;
            } else {
                let index = meta.index();
                let id_index = self.data[index].index;
                self.indices[id_index as usize] = SlotId::null();
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

    pub fn contains(&self, slot: SlotId) -> bool {
        self.indices
            .get(slot.index())
            .map(|index| !index.is_null() && index.version() == slot.version())
            .unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.data.iter().map(|e| {
            (
                SlotId::new(e.index as usize, self.indices[e.index as usize].version()),
                &e.value,
            )
        })
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlotId, &mut V)> {
        self.data.iter_mut().map(|e| {
            (
                SlotId::new(e.index as usize, self.indices[e.index as usize].version()),
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

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        self.indices.get(slot.index()).and_then(|index| {
            if index.is_null() || index.version() != slot.version() {
                None
            } else {
                self.data.get(index.index()).map(|e| &e.value)
            }
        })
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        self.indices.get(slot.index()).and_then(|index| {
            if index.is_null() || index.version() != slot.version() {
                None
            } else {
                self.data.get_mut(index.index()).map(|e| &mut e.value)
            }
        })
    }
}

impl<V> Index<SlotId> for SparseSecondaryMap<V> {
    type Output = V;

    fn index(&self, slot: SlotId) -> &Self::Output {
        self.get(slot).unwrap()
    }
}

impl<V> IndexMut<SlotId> for SparseSecondaryMap<V> {
    fn index_mut(&mut self, slot: SlotId) -> &mut Self::Output {
        self.get_mut(slot).unwrap()
    }
}

// impl<T, V: Serialize> Serialize for SparseSecondaryMap<SlotId<T>, V> {
//     type Header = V::Header;

//     fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
//         encoder.write_u32(self.data.len() as u32)?;
//         for entry in self.data.iter() {
//             entry.value.serialize(encoder)?;
//             encoder.write_u32(entry.index)?;
//         }
//         Ok(())
//     }

//     fn deserialize(
//         decoder: &mut impl Decoder,
//         header: &Self::Header,
//     ) -> Result<Self, DecoderError> {
//         let len = decoder.read_u32()? as usize;
//         let mut map = Self::with_capacity(len);
//         for _ in 0..len {
//             let value = V::deserialize(decoder, header)?;
//             let index = decoder.read_u32()?;
//             map.insert(SlotId::new(index), value);
//         }
//         Ok(map)
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slotmap() {
        let mut sm = SlotMap::<u32>::default();
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
        let mut sm = DenseSlotMap::<u32>::default();
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
        let mut sm = SlotMap::<u32>::default();
        let mut ssm = SecondaryMap::<u32>::default();
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
        let mut sm = SlotMap::<u32>::default();
        let mut ssm = SparseSecondaryMap::<u32>::default();
        let s0 = sm.add(1);
        ssm.insert(s0, 1);
        assert_eq!(ssm.get(s0), Some(&1));
        ssm.insert(s0, 2);
        assert_eq!(ssm.get(s0), Some(&2));
        ssm.remove(s0);
        assert!(ssm.get(s0).is_none());
    }
}
