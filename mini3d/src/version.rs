use crate::serialize::{Serialize, Encoder, Decoder, DecoderError, EncoderError};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version(u32);

impl Serialize for Version {

    type Header = ();

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.0)
    }

    fn deserialize(decoder: &mut impl Decoder, _header: &Self::Header) -> Result<Self, DecoderError> {
        decoder.read_u32().map(Self)
    }
}

impl Version {

    pub fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self((major as u32) << 16 | (minor as u32) << 8 | patch as u32)
    }

    pub fn major(&self) -> u8 {
        (self.0 >> 16) as u8
    }

    pub fn minor(&self) -> u8 {
        (self.0 >> 8) as u8
    }

    pub fn patch(&self) -> u8 {
        self.0 as u8
    }

    pub fn core() -> Self {
        Self::new(0, 0, 0)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::core()
    }
}

impl From<(u8, u8, u8)> for Version {
    fn from((major, minor, patch): (u8, u8, u8)) -> Self {
        Self::new(major, minor, patch)
    }
}

impl From<u32> for Version {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl From<Version> for u32 {
    fn from(value: Version) -> Self {
        value.0
    }
}