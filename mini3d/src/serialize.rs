use std::collections::{BTreeMap, HashMap, VecDeque};

use glam::{IVec2, IVec3, IVec4, Mat4, Quat, UVec2, Vec2, Vec3, Vec4};
use mini3d_derive::Error;

#[derive(Debug, Error)]
pub enum EncoderError {
    #[error("Unsupported")]
    Unsupported,
}

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("Unsupported")]
    Unsupported,
    #[error("Corrupted data")]
    CorruptedData,
}

pub trait Encoder {
    fn write_byte(&mut self, value: u8) -> Result<(), EncoderError>;
    fn write_bytes(&mut self, value: &[u8]) -> Result<(), EncoderError>;
    fn write_f32(&mut self, value: f32) -> Result<(), EncoderError>;
    fn write_f64(&mut self, value: f64) -> Result<(), EncoderError>;
    fn write_u16(&mut self, value: u16) -> Result<(), EncoderError>;
    fn write_u32(&mut self, value: u32) -> Result<(), EncoderError>;
    fn write_u64(&mut self, value: u64) -> Result<(), EncoderError>;
}

pub trait Decoder {
    fn read_byte(&mut self) -> Result<u8, DecoderError>;
    fn read_bytes(&mut self, count: usize) -> Result<&[u8], DecoderError>;
    fn read_f32(&mut self) -> Result<f32, DecoderError>;
    fn read_f64(&mut self) -> Result<f64, DecoderError>;
    fn read_u16(&mut self) -> Result<u16, DecoderError>;
    fn read_u32(&mut self) -> Result<u32, DecoderError>;
    fn read_u64(&mut self) -> Result<u64, DecoderError>;
}

impl Encoder for &mut dyn Encoder {
    fn write_byte(&mut self, value: u8) -> Result<(), EncoderError> {
        (*self).write_byte(value)
    }
    fn write_bytes(&mut self, value: &[u8]) -> Result<(), EncoderError> {
        (*self).write_bytes(value)
    }
    fn write_f32(&mut self, value: f32) -> Result<(), EncoderError> {
        (*self).write_f32(value)
    }
    fn write_f64(&mut self, value: f64) -> Result<(), EncoderError> {
        (*self).write_f64(value)
    }
    fn write_u16(&mut self, value: u16) -> Result<(), EncoderError> {
        (*self).write_u16(value)
    }
    fn write_u32(&mut self, value: u32) -> Result<(), EncoderError> {
        (*self).write_u32(value)
    }
    fn write_u64(&mut self, value: u64) -> Result<(), EncoderError> {
        (*self).write_u64(value)
    }
}

impl Decoder for &mut dyn Decoder {
    fn read_byte(&mut self) -> Result<u8, DecoderError> {
        (*self).read_byte()
    }
    fn read_bytes(&mut self, count: usize) -> Result<&[u8], DecoderError> {
        (*self).read_bytes(count)
    }
    fn read_f32(&mut self) -> Result<f32, DecoderError> {
        (*self).read_f32()
    }
    fn read_f64(&mut self) -> Result<f64, DecoderError> {
        (*self).read_f64()
    }
    fn read_u16(&mut self) -> Result<u16, DecoderError> {
        (*self).read_u16()
    }
    fn read_u32(&mut self) -> Result<u32, DecoderError> {
        (*self).read_u32()
    }
    fn read_u64(&mut self) -> Result<u64, DecoderError> {
        (*self).read_u64()
    }
}

impl Encoder for Vec<u8> {
    fn write_byte(&mut self, value: u8) -> Result<(), EncoderError> {
        self.push(value);
        Ok(())
    }
    fn write_bytes(&mut self, value: &[u8]) -> Result<(), EncoderError> {
        self.extend_from_slice(value);
        Ok(())
    }
    fn write_f32(&mut self, value: f32) -> Result<(), EncoderError> {
        self.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    fn write_f64(&mut self, value: f64) -> Result<(), EncoderError> {
        self.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    fn write_u16(&mut self, value: u16) -> Result<(), EncoderError> {
        self.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    fn write_u32(&mut self, value: u32) -> Result<(), EncoderError> {
        self.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    fn write_u64(&mut self, value: u64) -> Result<(), EncoderError> {
        self.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

pub struct SliceDecoder<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> SliceDecoder<'a> {
    pub fn new(data: &'a [u8]) -> SliceDecoder {
        Self { data, pos: 0 }
    }
}

impl<'a> Decoder for SliceDecoder<'a> {
    fn read_byte(&mut self) -> Result<u8, DecoderError> {
        if self.pos >= self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = self.data[self.pos];
        self.pos += 1;
        Ok(value)
    }
    fn read_bytes(&mut self, count: usize) -> Result<&[u8], DecoderError> {
        if self.pos + count > self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = &self.data[self.pos..self.pos + count];
        self.pos += count;
        Ok(value)
    }
    fn read_f32(&mut self) -> Result<f32, DecoderError> {
        if self.pos + 4 > self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = f32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(value)
    }
    fn read_f64(&mut self) -> Result<f64, DecoderError> {
        if self.pos + 8 > self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = f64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(value)
    }
    fn read_u16(&mut self) -> Result<u16, DecoderError> {
        if self.pos + 2 > self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = u16::from_le_bytes(self.data[self.pos..self.pos + 2].try_into().unwrap());
        self.pos += 2;
        Ok(value)
    }
    fn read_u32(&mut self) -> Result<u32, DecoderError> {
        if self.pos + 4 > self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = u32::from_le_bytes(self.data[self.pos..self.pos + 4].try_into().unwrap());
        self.pos += 4;
        Ok(value)
    }
    fn read_u64(&mut self) -> Result<u64, DecoderError> {
        if self.pos + 8 > self.data.len() {
            return Err(DecoderError::CorruptedData);
        }
        let value = u64::from_le_bytes(self.data[self.pos..self.pos + 8].try_into().unwrap());
        self.pos += 8;
        Ok(value)
    }
}

pub trait Serialize: Sized {
    type Header: Serialize + Default;
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError>;
    fn deserialize(decoder: &mut impl Decoder, header: &Self::Header)
        -> Result<Self, DecoderError>;
}

impl Serialize for () {
    type Header = ();
    fn serialize(&self, _encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }
    fn deserialize(
        _decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<(), DecoderError> {
        Ok(())
    }
}

impl<A: Serialize, B: Serialize> Serialize for (A, B) {
    type Header = (A::Header, B::Header);
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.0.serialize(encoder)?;
        self.1.serialize(encoder)
    }
    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok((
            A::deserialize(decoder, &header.0)?,
            B::deserialize(decoder, &header.1)?,
        ))
    }
}

impl Serialize for bool {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_byte(if *self { 1 } else { 0 })
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(decoder.read_byte()? != 0)
    }
}

impl Serialize for char {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_byte(*self as u8)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(decoder.read_byte()? as char)
    }
}

impl Serialize for f32 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_f32(*self)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        decoder.read_f32()
    }
}

impl Serialize for f64 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_f64(*self)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        decoder.read_f64()
    }
}

impl Serialize for u8 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_byte(*self)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        decoder.read_byte()
    }
}

impl Serialize for u32 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(*self)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        decoder.read_u32()
    }
}

impl Serialize for i32 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(*self as u32)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(decoder.read_u32()? as i32)
    }
}

impl Serialize for usize {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u64(*self as u64)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(decoder.read_u64()? as usize)
    }
}

impl Serialize for &str {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.len() as u32)?;
        encoder.write_bytes(self.as_bytes())
    }

    fn deserialize(
        _decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Err(DecoderError::Unsupported)
    }
}

impl Serialize for String {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.as_str().serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let len = decoder.read_u32()? as usize;
        String::from_utf8(decoder.read_bytes(len)?.to_vec()).map_err(|_| DecoderError::Unsupported)
    }
}

impl<A: Serialize, B: Serialize, C: Serialize> Serialize for (A, B, C) {
    type Header = (A::Header, B::Header, C::Header);

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.0.serialize(encoder)?;
        self.1.serialize(encoder)?;
        self.2.serialize(encoder)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok((
            A::deserialize(decoder, &header.0)?,
            B::deserialize(decoder, &header.1)?,
            C::deserialize(decoder, &header.2)?,
        ))
    }
}

impl<A: Serialize, B: Serialize, C: Serialize, D: Serialize> Serialize for (A, B, C, D) {
    type Header = (A::Header, B::Header, C::Header, D::Header);

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.0.serialize(encoder)?;
        self.1.serialize(encoder)?;
        self.2.serialize(encoder)?;
        self.3.serialize(encoder)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok((
            A::deserialize(decoder, &header.0)?,
            B::deserialize(decoder, &header.1)?,
            C::deserialize(decoder, &header.2)?,
            D::deserialize(decoder, &header.3)?,
        ))
    }
}

impl<T: Serialize> Serialize for Option<T> {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        match self {
            Some(value) => {
                encoder.write_byte(1)?;
                value.serialize(encoder)
            }
            None => encoder.write_byte(0),
        }
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        if decoder.read_byte()? != 0 {
            Ok(Some(T::deserialize(decoder, header)?))
        } else {
            Ok(None)
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.len() as u32)?;
        for value in self {
            value.serialize(encoder)?;
        }
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let len = decoder.read_u32()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::deserialize(decoder, header)?);
        }
        Ok(vec)
    }
}

impl<T: Serialize> Serialize for VecDeque<T> {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.len() as u32)?;
        for value in self {
            value.serialize(encoder)?;
        }
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let len = decoder.read_u32()? as usize;
        let mut vec = VecDeque::with_capacity(len);
        for _ in 0..len {
            vec.push_back(T::deserialize(decoder, header)?);
        }
        Ok(vec)
    }
}

impl<T: Serialize> Serialize for Box<T> {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.as_ref().serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(Box::new(T::deserialize(decoder, header)?))
    }
}

impl<K: Serialize + std::hash::Hash + std::cmp::Eq, V: Serialize> Serialize for HashMap<K, V> {
    type Header = (K::Header, V::Header);

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.len() as u32)?;
        for (key, value) in self {
            key.serialize(encoder)?;
            value.serialize(encoder)?;
        }
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let len = decoder.read_u32()? as usize;
        let mut map = HashMap::with_capacity(len);
        for _ in 0..len {
            let key = K::deserialize(decoder, &header.0)?;
            let value = V::deserialize(decoder, &header.1)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl<K: Serialize + core::hash::Hash + core::cmp::Eq + core::cmp::Ord, V: Serialize> Serialize
    for BTreeMap<K, V>
{
    type Header = (K::Header, V::Header);

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.len() as u32)?;
        for (key, value) in self {
            key.serialize(encoder)?;
            value.serialize(encoder)?;
        }
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let len = decoder.read_u32()? as usize;
        let mut map = BTreeMap::new();
        for _ in 0..len {
            let key = K::deserialize(decoder, &header.0)?;
            let value = V::deserialize(decoder, &header.1)?;
            map.insert(key, value);
        }
        Ok(map)
    }
}

impl Serialize for Vec2 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_f32(self.x)?;
        encoder.write_f32(self.y)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_f32()?;
        let y = decoder.read_f32()?;
        Ok(Vec2::new(x, y))
    }
}

impl Serialize for UVec2 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.x)?;
        encoder.write_u32(self.y)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_u32()?;
        let y = decoder.read_u32()?;
        Ok(UVec2::new(x, y))
    }
}

impl Serialize for IVec2 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.x as u32)?;
        encoder.write_u32(self.y as u32)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_u32()? as i32;
        let y = decoder.read_u32()? as i32;
        Ok(IVec2::new(x, y))
    }
}

impl Serialize for Vec3 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_f32(self.x)?;
        encoder.write_f32(self.y)?;
        encoder.write_f32(self.z)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_f32()?;
        let y = decoder.read_f32()?;
        let z = decoder.read_f32()?;
        Ok(Vec3::new(x, y, z))
    }
}

impl Serialize for IVec3 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.x as u32)?;
        encoder.write_u32(self.y as u32)?;
        encoder.write_u32(self.z as u32)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_u32()? as i32;
        let y = decoder.read_u32()? as i32;
        let z = decoder.read_u32()? as i32;
        Ok(IVec3::new(x, y, z))
    }
}

impl Serialize for Vec4 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_f32(self.x)?;
        encoder.write_f32(self.y)?;
        encoder.write_f32(self.z)?;
        encoder.write_f32(self.w)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_f32()?;
        let y = decoder.read_f32()?;
        let z = decoder.read_f32()?;
        let w = decoder.read_f32()?;
        Ok(Vec4::new(x, y, z, w))
    }
}

impl Serialize for IVec4 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.x as u32)?;
        encoder.write_u32(self.y as u32)?;
        encoder.write_u32(self.z as u32)?;
        encoder.write_u32(self.w as u32)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_u32()? as i32;
        let y = decoder.read_u32()? as i32;
        let z = decoder.read_u32()? as i32;
        let w = decoder.read_u32()? as i32;
        Ok(IVec4::new(x, y, z, w))
    }
}

impl Serialize for Quat {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_f32(self.x)?;
        encoder.write_f32(self.y)?;
        encoder.write_f32(self.z)?;
        encoder.write_f32(self.w)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = decoder.read_f32()?;
        let y = decoder.read_f32()?;
        let z = decoder.read_f32()?;
        let w = decoder.read_f32()?;
        Ok(Quat::from_xyzw(x, y, z, w))
    }
}

impl Serialize for Mat4 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        for x in self.to_cols_array() {
            encoder.write_f32(x)?;
        }
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let mut array = [0.0; 16];
        for x in &mut array {
            *x = decoder.read_f32()?;
        }
        Ok(Mat4::from_cols_array(&array))
    }
}
