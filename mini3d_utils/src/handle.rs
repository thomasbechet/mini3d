use core::{fmt::Display, marker::PhantomData};

use mini3d_derive::Serialize;
use mini3d_serialize::Serialize;

// Raw handle representation
// Null handle representation is possible but it should not be used
// as sentinel value.
#[derive(Serialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct RawHandle(u32);

impl From<u32> for RawHandle {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

impl From<RawHandle> for u32 {
    fn from(handle: RawHandle) -> u32 {
        handle.0
    }
}

impl RawHandle {
    pub fn null() -> Self {
        Self(0)
    }
}

impl Default for RawHandle {
    fn default() -> Self {
        Self::null()
    }
}

impl Display for RawHandle {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
       write!(f, "{}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Handle<T>(RawHandle, PhantomData<T>);

impl<T> Default for Handle<T> {
    fn default() -> Self {
        Self(RawHandle::null(), PhantomData)
    }
}

impl<T> Serialize for Handle<T> {
    type Header = <RawHandle as Serialize>::Header;

    fn serialize(
        &self,
        encoder: &mut impl mini3d_serialize::Encoder,
    ) -> Result<(), mini3d_serialize::EncoderError> {
        self.0.serialize(encoder)
    }

    fn deserialize(
        decoder: &mut impl mini3d_serialize::Decoder,
        header: &Self::Header,
    ) -> Result<Self, mini3d_serialize::DecoderError> {
        Ok(Self(
            <RawHandle as Serialize>::deserialize(decoder, header)?,
            Default::default(),
        ))
    }
}
