use core::{
    fmt::{Display, Formatter},
    ops::Deref,
};

use alloc::string::{String, ToString};
use mini3d_derive::Error;

use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

use super::uid::{ToUID, UID};

#[derive(Clone)]
pub struct AsciiArray<const SIZE: usize> {
    data: [u8; SIZE],
    len: usize,
}

#[derive(Error, PartialEq, Eq, Debug)]
pub enum AsciiArrayError {
    #[error("Invalid character")]
    InvalidCharacter,
    #[error("Out of bounds")]
    OutOfBounds,
}

impl<const SIZE: usize> AsciiArray<SIZE> {
    pub fn set(&mut self, value: &str) -> Result<(), AsciiArrayError> {
        if !value.is_ascii() {
            return Err(AsciiArrayError::InvalidCharacter);
        }
        if value.len() > self.data.len() {
            return Err(AsciiArrayError::OutOfBounds);
        }
        self.data[..value.len()].copy_from_slice(value.as_bytes());
        self.len = value.as_bytes().len();
        Ok(())
    }

    pub fn capacity(&self) -> usize {
        SIZE
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.data[..self.len]).unwrap()
    }

    pub fn push(&mut self, c: char) {
        if self.len < self.data.len() {
            self.data[self.len] = c as u8;
            self.len += 1;
        }
    }
}

impl<const SIZE: usize> Display for AsciiArray<SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl<const SIZE: usize> Default for AsciiArray<SIZE> {
    fn default() -> Self {
        Self {
            data: [0; SIZE],
            len: 0,
        }
    }
}

impl<const SIZE: usize> ToUID for AsciiArray<SIZE> {
    fn to_uid(&self) -> UID {
        self.as_str().to_uid()
    }
}

impl<const SIZE: usize> From<&str> for AsciiArray<SIZE> {
    fn from(value: &str) -> Self {
        let mut array = Self::default();
        array
            .set(value)
            .unwrap_or_else(|_| panic!("{}", AsciiArrayError::OutOfBounds.to_string()));
        array
    }
}

impl<'a, const SIZE: usize> From<&'a AsciiArray<SIZE>> for &'a str {
    fn from(value: &'a AsciiArray<SIZE>) -> Self {
        value.as_str()
    }
}

impl<const SIZE: usize> From<String> for AsciiArray<SIZE> {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl<const SIZE: usize> AsRef<str> for AsciiArray<SIZE> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<const SIZE: usize> Deref for AsciiArray<SIZE> {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const SIZE: usize> PartialEq for AsciiArray<SIZE> {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl<const SIZE: usize> PartialEq<&str> for AsciiArray<SIZE> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl<const SIZE: usize> Serialize for AsciiArray<SIZE> {
    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.as_str().serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let mut array = Self::default();
        array.len = decoder.read_u32()? as usize;
        array.data.copy_from_slice(
            decoder
                .read_bytes(array.len)
                .map_err(|_| DecoderError::CorruptedData)?,
        );
        Ok(array)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_ascii_array() {
        let mut array = AsciiArray::<5>::default();
        assert_eq!(array.len(), 0);
        assert_eq!(array.capacity(), 5);
        assert!(array.is_empty());
        assert_eq!(array.as_str(), "");
        assert_eq!(array.set("abcdef"), Err(AsciiArrayError::OutOfBounds));
        assert_eq!(array.set("abcde"), Ok(()));
        assert_eq!(array.len(), 5);
    }
}
