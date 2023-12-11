use alloc::vec::Vec;

use crate::{
    math::{
        mat::{M4, M4I32F16},
        quat::Q,
        vec::{V2, V2I32, V2I32F16, V3, V3I32, V3I32F16, V4, V4I32, V4I32F16},
    },
    serialize::{Serialize, SliceDecoder},
};

use super::primitive::PrimitiveType;

#[derive(Debug, Clone, Copy)]
pub struct DataId(u16);

impl From<u16> for DataId {
    fn from(value: u16) -> Self {
        Self(value)
    }
}

impl From<DataId> for u16 {
    fn from(value: DataId) -> Self {
        value.0
    }
}

struct DataEntry {
    ty: PrimitiveType,
    data: u32,
}

pub(crate) struct ConstantFormatter<'a> {
    id: DataId,
    table: &'a DataTable,
}

impl<'a> core::fmt::Display for ConstantFormatter<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let entry = &self.table.entries[self.id.0 as usize];
        match entry.ty {
            PrimitiveType::Bool => write!(f, "{}", self.table.read_bool(self.id)),
            PrimitiveType::V2I32F16 => write!(f, "{}", self.table.read_v2i32f16(self.id)),
            PrimitiveType::V2I32 => write!(f, "{}", self.table.read_v2i32(self.id)),
            PrimitiveType::V3I32F16 => write!(f, "{}", self.table.read_v3i32f16(self.id)),
            PrimitiveType::V3I32 => write!(f, "{}", self.table.read_v3i32(self.id)),
            PrimitiveType::V4I32F16 => write!(f, "{}", self.table.read_v4i32f16(self.id)),
            PrimitiveType::V4I32 => write!(f, "{}", self.table.read_v4i32(self.id)),
            PrimitiveType::M4I32F16 => write!(f, "{}", self.table.read_m4i32f16(self.id)),
            PrimitiveType::QI32F16 => write!(f, "{}", self.table.read_qi32f16(self.id)),
            PrimitiveType::String => write!(f, "{}", self.table.read_str(self.id)),
            _ => unreachable!(),
        }
    }
}

#[derive(Default)]
pub(crate) struct DataTable {
    entries: Vec<DataEntry>,
    data: Vec<u8>,
}

impl DataTable {
    pub(crate) fn clear(&mut self) {
        self.entries.clear();
        self.data.clear();
    }

    pub(crate) fn remove(&mut self, id: DataId) {
        let entry = self.entries.remove(id.0 as usize);
        match entry.ty {
            PrimitiveType::String => {}
            _ => {}
        }
        // TODO: remove from data
    }

    pub(crate) fn import(&mut self, id: DataId, table: &DataTable) -> DataId {
        let ty = table.entries[id.0 as usize].ty;
        match ty {
            PrimitiveType::Bool => self.add_bool(table.read_bool(id)),
            PrimitiveType::I32 => self.add_i32(table.read_i32(id)),
            // PrimitiveType::V2I32F16 => self.add_vec2(table.read_v2i32f16(id)),
            // PrimitiveType::V2I32 => self.add_ivec2(table.read_v2i32(id)),
            // PrimitiveType::V3I32F16 => self.add_vec3(table.read_v3i32f16(id)),
            // PrimitiveType::V3I32 => self.add_ivec3(table.read_v3i32(id)),
            // PrimitiveType::V4I32F16 => self.add_vec4(table.read_v4i32f16(id)),
            // PrimitiveType::V4I32 => self.add_ivec4(table.read_v4i32(id)),
            // PrimitiveType::M4I32F16 => self.add_mat4(table.read_m4i32f16(id)),
            // PrimitiveType::QI32F16 => self.add_quat(table.read_qi32f16(id)),
            // PrimitiveType::String => self.add_str(table.read_str(id)),
            _ => unreachable!(),
        }
    }

    pub(crate) fn format(&self, id: DataId) -> ConstantFormatter {
        ConstantFormatter { id, table: self }
    }

    pub(crate) fn add_bool(&mut self, value: bool) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Bool,
            data: value as u32,
        });
        DataId(id)
    }

    pub(crate) fn add_i32(&mut self, value: i32) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::I32,
            data: value,
        });
        DataId(id)
    }

    pub(crate) fn add_str(&mut self, value: &str) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::String,
            data: self.data.len() as u32,
        });
        self.data
            .extend_from_slice(&(value.len() as u32).to_be_bytes());
        self.data.extend_from_slice(value.as_bytes());
        DataId(id)
    }

    // pub(crate) fn add_vec2(&mut self, value: V2I32F16) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::V2I32F16,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_ivec2(&mut self, value: V2I32) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::V2I32,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_vec3(&mut self, value: V3I32F16) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::V3I32F16,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_ivec3(&mut self, value: V3I32) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::V3I32,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_vec4(&mut self, value: V4I32F16) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::V4I32F16,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_ivec4(&mut self, value: V4I32) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::V4I32,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_mat4(&mut self, value: M4I32F16) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::M4I32F16,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    // pub(crate) fn add_quat(&mut self, value: QI32F16) -> DataId {
    //     let id = self.entries.len() as u16;
    //     self.entries.push(DataEntry {
    //         ty: PrimitiveType::QI32F16,
    //         data: self.data.len() as u32,
    //     });
    //     value.serialize(&mut self.data);
    //     DataId(id)
    // }

    pub(crate) fn read_bool(&self, id: DataId) -> bool {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Bool);
        entry.data != 0
    }

    pub(crate) fn read_i32(&self, id: DataId) -> i32 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::I32);
        entry.data
    }

    pub(crate) fn read_str(&self, id: DataId) -> &str {
        let entry = &self.entries[id.0 as usize];
        let start = entry.data as usize + core::mem::size_of::<u32>();
        let len = u32::from_be_bytes(
            self.data[entry.data as usize..entry.data as usize + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        core::str::from_utf8(&self.data[start..start + len]).unwrap()
    }

    pub(crate) fn read_v2i32f16(&self, id: DataId) -> V2I32F16 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::V2I32F16);
        V2::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_v2i32(&self, id: DataId) -> V2I32 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::V2I32);
        V2::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_v3i32f16(&self, id: DataId) -> V3I32F16 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::V3I32F16);
        V3::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_v3i32(&self, id: DataId) -> V3I32 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::V3I32);
        V3::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_v4i32f16(&self, id: DataId) -> V4I32F16 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::V4I32F16);
        V4::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_v4i32(&self, id: DataId) -> V4I32 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::V4I32);
        V4::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_m4i32f16(&self, id: DataId) -> M4I32F16 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::M4I32F16);
        M4::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_qi32f16(&self, id: DataId) -> QI32F16 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::QI32F16);
        Q::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }
}
