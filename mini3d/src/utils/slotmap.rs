use std::ops::{Index, IndexMut};

#[derive(Debug, Hash, Copy, Clone, PartialEq, Eq)]
pub struct SlotId(u32);

impl SlotId {
    fn new(value: u32) -> Self {
        Self(value + 1)
    }

    fn index(&self) -> usize {
        assert!(self.0 > 0);
        (self.0 - 1) as usize
    }

    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl From<u32> for SlotId {
    fn from(value: u32) -> Self {
        Self::new(value)
    }
}

impl From<SlotId> for u32 {
    fn from(value: SlotId) -> Self {
        value.0
    }
}

impl Default for SlotId {
    fn default() -> Self {
        Self::null()
    }
}

enum SlotEntry<V> {
    Value(V),
    Free(SlotId),
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
            self.entries.push(SlotEntry::Value(value));
            SlotId::new(index as u32)
        } else {
            let free = self.free;
            match self.entries[free.index()] {
                SlotEntry::Free(next_free) => {
                    self.entries[free.index()] = SlotEntry::Value(value);
                    self.free = next_free;
                    free
                }
                _ => panic!("Invalid slot entry"),
            }
        }
    }

    pub fn remove(&mut self, id: SlotId) {
        self.entries[id.index()] = SlotEntry::Free(self.free);
        self.free = id;
    }

    pub fn get(&self, id: SlotId) -> Option<&V> {
        match &self.entries[id.index()] {
            SlotEntry::Value(value) => Some(value),
            SlotEntry::Free(_) => None,
        }
    }

    pub fn get_mut(&mut self, id: SlotId) -> Option<&mut V> {
        match &mut self.entries[id.index()] {
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

    fn index(&self, id: SlotId) -> &Self::Output {
        self.get(id).unwrap()
    }
}

impl<V> IndexMut<SlotId> for SlotMap<V> {
    fn index_mut(&mut self, id: SlotId) -> &mut Self::Output {
        self.get_mut(id).unwrap()
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
    id_to_index: u32,
    index_to_id: u32, // or free id if unused
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
            let free_id = self.meta[size].index_to_id;
            self.meta[size].id_to_index = size as u32;
            self.meta[size].index_to_id = free_id;
            self.free_count -= 1;
            return SlotId::new(size as u32);
        } else {
            let size = self.data.len();
            self.data.push(value);
            self.meta.push(DenseSlotMapMeta {
                id_to_index: size as u32,
                index_to_id: size as u32,
            });
            return SlotId::new(size as u32);
        }
    }

    pub fn remove(&mut self, id: SlotId) {
        let last_index = self.data.len() - 1;
        let index = self.meta[id.index()].id_to_index as usize;
        self.data.swap_remove(index);
        let last_id = self.meta[last_index].index_to_id;
        self.meta[last_id as usize].id_to_index = index as u32;
        self.meta[index].index_to_id = last_id;
        self.meta[last_index].index_to_id = id.index() as u32; // free id
        self.free_count += 1;
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.data
            .iter()
            .zip(self.meta.iter())
            .map(|(value, meta)| (SlotId::new(meta.id_to_index), value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter()
    }

    pub fn get(&self, id: SlotId) -> Option<&V> {
        let index = self.meta[id.index()].id_to_index as usize;
        self.data.get(index)
    }

    pub fn get_mut(&mut self, id: SlotId) -> Option<&mut V> {
        let index = self.meta[id.index()].id_to_index as usize;
        self.data.get_mut(index)
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
            .and_then(|index| index.and_then(|id| self.map.get(id)))
    }

    pub fn get_mut(&mut self, key: SlotId) -> Option<&mut V> {
        self.indices
            .get(key.index())
            .and_then(|index| index.and_then(move |id| self.map.get_mut(id)))
    }
}

impl<V> Index<SlotId> for SecondaryMap<V> {
    type Output = V;

    fn index(&self, id: SlotId) -> &Self::Output {
        self.get(id).unwrap()
    }
}

impl<V> IndexMut<SlotId> for SecondaryMap<V> {
    fn index_mut(&mut self, id: SlotId) -> &mut Self::Output {
        self.get_mut(id).unwrap()
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

    pub fn insert(&mut self, id: SlotId, value: V) {
        if id.index() >= self.indices.len() {
            self.indices.resize(id.index() + 1, None);
        }
        if let Some(index) = self.indices[id.index()] {
            self.data[index as usize].value = value;
        } else {
            self.indices[id.index()] = Some(self.data.len() as u32);
            self.data.push(SparseSecondaryMapEntry {
                value,
                index: id.index() as u32,
            });
        }
    }

    pub fn remove(&mut self, id: SlotId) -> Option<V> {
        if let Some(index) = self.indices[id.index()] {
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

    pub fn contains(&self, id: SlotId) -> bool {
        self.indices
            .get(id.index())
            .map(|index| index.is_some())
            .unwrap_or(false)
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlotId, &V)> {
        self.data
            .iter()
            .map(|e| (SlotId::new(e.index as u32), &e.value))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlotId, &mut V)> {
        self.data
            .iter_mut()
            .map(|e| (SlotId::new(e.index as u32), &mut e.value))
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.data.iter().map(|e| &e.value)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.data.iter_mut().map(|e| &mut e.value)
    }

    pub fn get(&self, id: SlotId) -> Option<&V> {
        self.indices
            .get(id.index())
            .and_then(|index| index.and_then(|id| self.data.get(id as usize).map(|e| &e.value)))
    }

    pub fn get_mut(&mut self, id: SlotId) -> Option<&mut V> {
        self.indices.get(id.index()).and_then(|index| {
            index.and_then(move |id| self.data.get_mut(id as usize).map(|e| &mut e.value))
        })
    }
}

impl<V> Index<SlotId> for SparseSecondaryMap<V> {
    type Output = V;

    fn index(&self, id: SlotId) -> &Self::Output {
        self.get(id).unwrap()
    }
}

impl<V> IndexMut<SlotId> for SparseSecondaryMap<V> {
    fn index_mut(&mut self, id: SlotId) -> &mut Self::Output {
        self.get_mut(id).unwrap()
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
