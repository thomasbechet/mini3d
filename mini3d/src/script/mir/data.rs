use glam::{IVec2, IVec3, IVec4, Mat4, Quat, Vec2, Vec3, Vec4};

use crate::serialize::{Serialize, SliceDecoder};

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
            PrimitiveType::Boolean => write!(f, "{}", self.table.read_bool(self.id)),
            PrimitiveType::Integer => write!(f, "{}", self.table.read_u32(self.id)),
            PrimitiveType::Float => write!(f, "{}", self.table.read_f32(self.id)),
            PrimitiveType::Vec2 => write!(f, "{}", self.table.read_vec2(self.id)),
            PrimitiveType::IVec2 => write!(f, "{}", self.table.read_ivec2(self.id)),
            PrimitiveType::Vec3 => write!(f, "{}", self.table.read_vec3(self.id)),
            PrimitiveType::IVec3 => write!(f, "{}", self.table.read_ivec3(self.id)),
            PrimitiveType::Vec4 => write!(f, "{}", self.table.read_vec4(self.id)),
            PrimitiveType::IVec4 => write!(f, "{}", self.table.read_ivec4(self.id)),
            PrimitiveType::Mat4 => write!(f, "{}", self.table.read_mat4(self.id)),
            PrimitiveType::Quat => write!(f, "{}", self.table.read_quat(self.id)),
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
            PrimitiveType::Boolean => self.add_bool(table.read_bool(id)),
            PrimitiveType::Integer => self.add_u32(table.read_u32(id)),
            PrimitiveType::Float => self.add_f32(table.read_f32(id)),
            PrimitiveType::Vec2 => self.add_vec2(table.read_vec2(id)),
            PrimitiveType::IVec2 => self.add_ivec2(table.read_ivec2(id)),
            PrimitiveType::Vec3 => self.add_vec3(table.read_vec3(id)),
            PrimitiveType::IVec3 => self.add_ivec3(table.read_ivec3(id)),
            PrimitiveType::Vec4 => self.add_vec4(table.read_vec4(id)),
            PrimitiveType::IVec4 => self.add_ivec4(table.read_ivec4(id)),
            PrimitiveType::Mat4 => self.add_mat4(table.read_mat4(id)),
            PrimitiveType::Quat => self.add_quat(table.read_quat(id)),
            PrimitiveType::String => self.add_str(table.read_str(id)),
            _ => unreachable!(),
        }
    }

    pub(crate) fn format(&self, id: DataId) -> ConstantFormatter {
        ConstantFormatter { id, table: self }
    }

    pub(crate) fn add_bool(&mut self, value: bool) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Boolean,
            data: value as u32,
        });
        DataId(id)
    }

    pub(crate) fn add_u32(&mut self, value: u32) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Integer,
            data: value,
        });
        DataId(id)
    }

    pub(crate) fn add_f32(&mut self, value: f32) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Float,
            data: value.to_bits(),
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

    pub(crate) fn add_vec2(&mut self, value: Vec2) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Vec2,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_ivec2(&mut self, value: IVec2) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::IVec2,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_vec3(&mut self, value: Vec3) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Vec3,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_ivec3(&mut self, value: IVec3) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::IVec3,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_vec4(&mut self, value: Vec4) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Vec4,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_ivec4(&mut self, value: IVec4) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::IVec4,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_mat4(&mut self, value: Mat4) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Mat4,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn add_quat(&mut self, value: Quat) -> DataId {
        let id = self.entries.len() as u16;
        self.entries.push(DataEntry {
            ty: PrimitiveType::Quat,
            data: self.data.len() as u32,
        });
        value.serialize(&mut self.data);
        DataId(id)
    }

    pub(crate) fn read_bool(&self, id: DataId) -> bool {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Boolean);
        entry.data != 0
    }

    pub(crate) fn read_u32(&self, id: DataId) -> u32 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Integer);
        entry.data
    }

    pub(crate) fn read_f32(&self, id: DataId) -> f32 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Float);
        f32::from_bits(entry.data)
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

    pub(crate) fn read_vec2(&self, id: DataId) -> Vec2 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Vec2);
        Vec2::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_ivec2(&self, id: DataId) -> IVec2 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::IVec2);
        IVec2::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_vec3(&self, id: DataId) -> Vec3 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Vec3);
        Vec3::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_ivec3(&self, id: DataId) -> IVec3 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::IVec3);
        IVec3::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_vec4(&self, id: DataId) -> Vec4 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Vec4);
        Vec4::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_ivec4(&self, id: DataId) -> IVec4 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::IVec4);
        IVec4::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_mat4(&self, id: DataId) -> Mat4 {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Mat4);
        Mat4::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }

    pub(crate) fn read_quat(&self, id: DataId) -> Quat {
        let entry = &self.entries[id.0 as usize];
        assert!(entry.ty == PrimitiveType::Quat);
        Quat::deserialize(
            &mut SliceDecoder::new(&self.data[entry.data as usize..]),
            &Default::default(),
        )
        .unwrap()
    }
}