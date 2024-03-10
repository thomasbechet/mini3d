#![no_std]

extern crate alloc;

use core::num::NonZeroU32;

use alloc::{boxed::Box, collections::VecDeque, string::String, vec::Vec};
use mini3d_derive::Error;

pub mod version;

pub use version::*;

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
    #[error("Invalid size: found {found}, expected {expected}")]
    InvalidSize { found: usize, expected: usize },
}

pub trait Encoder {
    fn write_byte(&mut self, value: u8) -> Result<(), EncoderError>;
    fn write_bytes(&mut self, value: &[u8]) -> Result<(), EncoderError>;
    fn write_u16(&mut self, value: u16) -> Result<(), EncoderError>;
    fn write_u32(&mut self, value: u32) -> Result<(), EncoderError>;
    fn write_u64(&mut self, value: u64) -> Result<(), EncoderError>;
}

pub trait Decoder {
    fn read_byte(&mut self) -> Result<u8, DecoderError>;
    fn read_bytes(&mut self, count: usize) -> Result<&[u8], DecoderError>;
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

impl Serialize for u16 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u16(*self)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        decoder.read_u16()
    }
}

impl Serialize for i16 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u16(*self as u16)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(decoder.read_u16()? as i16)
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

impl Serialize for NonZeroU32 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.get())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        NonZeroU32::new(decoder.read_u32()?).ok_or(DecoderError::CorruptedData)
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

impl Serialize for i64 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u64(*self as u64)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        Ok(decoder.read_u64()? as i64)
    }
}

impl Serialize for u64 {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u64(*self)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        _header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        decoder.read_u64()
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

impl<T: Serialize + Default + Copy, const N: usize> Serialize for [T; N] {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(N as u32)?;
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
        if len != N {
            return Err(DecoderError::InvalidSize {
                found: len,
                expected: N,
            });
        }
        let mut array = [T::default(); N];
        for i in 0..N {
            array[i] = T::deserialize(decoder, header)?;
        }
        Ok(array)
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
