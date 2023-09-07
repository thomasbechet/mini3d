use std::ops::{Index, IndexMut};

use mini3d_derive::Serialize;

#[derive(Default, Copy, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct SlotVersion(u8);

impl SlotVersion {
    pub fn next(&mut self) -> Self {
        let old = *self;
        self.0 = self.0.wrapping_add(1);
        old
    }
}

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub struct SlotId(u32);

impl SlotId {
    fn new(index: u32, version: SlotVersion) -> Self {
        Self((index + 1) | ((version.0 as u32) << 24))
    }

    fn index(&self) -> usize {
        let index = self.0 & 0x00ff_ffff;
        assert!(index > 0);
        (index - 1) as usize
    }

    fn version(&self) -> SlotVersion {
        SlotVersion((self.0 >> 24) as u8)
    }

    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        (self.0 & 0x00ff_ffff) == 0
    }
}

impl Default for SlotId {
    fn default() -> Self {
        Self::null()
    }
}

struct SlotEntry<V> {
    value: V,
    meta: SlotId, // if version 'is_free' then
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
            let index = self.entries.len() as u32;
            let slot = SlotId::new(index, SlotVersion::default());
            self.entries.push(SlotEntry { value, meta: slot });
            slot
        } else {
            let free = self.free;
            let entry = &mut self.entries[free.index()];
            assert!(entry.meta.is_free());
            entry.value = value;
            entry.meta = SlotId::new(free.index() as u32, entry.meta.version().next());
            self.free = entry.meta;
            entry.meta
        }
    }

    pub fn remove(&mut self, slot: SlotId) {
        self.entries[slot.index()] = SlotEntry::Free(self.free);
        self.free = slot;
    }

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        match &self.entries[slot.index()] {
            SlotEntry::Value(value) => Some(value),
            SlotEntry::Free(_) => None,
        }
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        match &mut self.entries[slot.index()] {
            SlotEntry::Value(value) => Some(value),
            SlotEntry::Free(_) => None,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                SlotEntry::Value(value) => Some((SlotId::new(index as u32), value)),
                SlotEntry::Free(_) => None,
            })
    }

    pub fn keys(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.entries
            .iter()
            .enumerate()
            .filter_map(|(index, entry)| match entry {
                SlotEntry::Value(_) => Some(SlotId::new(index as u32)),
                SlotEntry::Free(_) => None,
            })
    }

    pub fn next(&self, slot: SlotId) -> Option<SlotId> {
        let mut index = slot.index() + 1;
        while index < self.entries.len() {
            if let SlotEntry::Value(_) = self.entries[index] {
                return Some(SlotId::new(index as u32));
            }
            index += 1;
        }
        None
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().filter_map(|entry| match entry {
            SlotEntry::Value(value) => Some(value),
            SlotEntry::Free(_) => None,
        })
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.entries.iter_mut().filter_map(|entry| match entry {
            SlotEntry::Value(value) => Some(value),
            SlotEntry::Free(_) => None,
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

struct DenseSlotMapMeta {
    slot_to_index: u32,
    index_to_slot: u32, // or free slot if unused
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
            SlotId::new(size as u32)
        } else {
            let size = self.data.len();
            self.data.push(value);
            self.meta.push(DenseSlotMapMeta {
                slot_to_index: size as u32,
                index_to_slot: size as u32,
            });
            SlotId::new(size as u32)
        }
    }

    pub fn remove(&mut self, slot: SlotId) {
        let last_index = self.data.len() - 1;
        let index = self.meta[slot.index()].slot_to_index as usize;
        self.data.swap_remove(index);
        let last_id = self.meta[last_index].index_to_slot;
        self.meta[last_id as usize].slot_to_index = index as u32;
        self.meta[index].index_to_slot = last_id;
        self.meta[last_index].index_to_slot = slot.index() as u32; // free slot
        self.free_count += 1;
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.data
            .iter()
            .zip(self.meta.iter())
            .map(|(value, meta)| (SlotId::new(meta.slot_to_index), value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut()
    }

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        let index = self.meta[slot.index()].slot_to_index as usize;
        self.data.get(index)
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        let index = self.meta[slot.index()].slot_to_index as usize;
        self.data.get_mut(index)
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

pub struct SecondaryMap<V> {
    map: SlotMap<V>,
    indices: Vec<Option<SlotId>>,
}

impl<V> SecondaryMap<V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            map: SlotMap::with_capacity(capacity),
            indices: Vec::with_capacity(capacity),
        }
    }

    pub fn insert(&mut self, key: SlotId, value: V) {
        if key.index() >= self.indices.len() {
            self.indices.resize(key.index() + 1, None);
        }
        if let Some(index) = self.indices[key.index()] {
            self.map.remove(index);
        }
        self.indices[key.index()] = Some(self.map.add(value));
    }

    pub fn remove(&mut self, key: SlotId) {
        if let Some(index) = self.indices[key.index()] {
            self.map.remove(index);
        }
        self.indices[key.index()] = None;
    }

    pub fn get(&self, key: SlotId) -> Option<&V> {
        self.indices
            .get(key.index())
            .and_then(|index| index.and_then(|slot| self.map.get(slot)))
    }

    pub fn get_mut(&mut self, key: SlotId) -> Option<&mut V> {
        if let Some(Some(slot)) = self.indices.get(key.index()) {
            return self.map.get_mut(*slot);
        }
        None
    }

    pub fn contains(&self, key: SlotId) -> bool {
        self.get(key).is_some()
    }

    pub fn clear(&mut self) {
        self.map.clear();
        self.indices.clear();
    }
}

impl<V> Default for SecondaryMap<V> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<V> Index<SlotId> for SecondaryMap<V> {
    type Output = V;

    fn index(&self, slot: SlotId) -> &Self::Output {
        self.get(slot).unwrap()
    }
}

impl<V> IndexMut<SlotId> for SecondaryMap<V> {
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
    indices: Vec<Option<u32>>,
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
            self.indices.resize(slot.index() + 1, None);
        }
        if let Some(index) = self.indices[slot.index()] {
            self.data[index as usize].value = value;
        } else {
            self.indices[slot.index()] = Some(self.data.len() as u32);
            self.data.push(SparseSecondaryMapEntry {
                value,
                index: slot.index() as u32,
            });
        }
    }

    pub fn remove(&mut self, slot: SlotId) -> Option<V> {
        if let Some(index) = self.indices[slot.index()] {
            let id_index = self.data[index as usize].index;
            self.indices[id_index as usize] = None;
            // Swap with last entry
            if index != self.data.len() as u32 - 1 {
                let last_index = self.data.len() as u32 - 1;
                let last_id_index = self.data[last_index as usize].index;
                self.data.swap(index as usize, last_index as usize);
                self.indices[last_id_index as usize] = Some(index);
            }
            return self.data.pop().map(|e| e.value);
        }
        None
    }

    pub fn contains(&self, slot: SlotId) -> bool {
        self.indices
            .get(slot.index())
            .map(|index| index.is_some())
            .unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.data.iter().map(|e| (SlotId::new(e.index), &e.value))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlotId, &mut V)> {
        self.data
            .iter_mut()
            .map(|e| (SlotId::new(e.index), &mut e.value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|e| &e.value)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().map(|e| &mut e.value)
    }

    pub fn get(&self, slot: SlotId) -> Option<&V> {
        self.indices
            .get(slot.index())
            .and_then(|index| index.and_then(|slot| self.data.get(slot as usize).map(|e| &e.value)))
    }

    pub fn get_mut(&mut self, slot: SlotId) -> Option<&mut V> {
        if let Some(Some(slot)) = self.indices.get(slot.index()) {
            return self
                .data
                .get_mut(*slot as usize)
                .map(|entry| &mut entry.value);
        }
        None
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
