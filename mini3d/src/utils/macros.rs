#[macro_export]
macro_rules! define_provider_handle {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);
        impl From<u64> for $name {
            fn from(value: u64) -> Self {
                Self(value)
            }
        }
        impl From<$name> for u64 {
            fn from(handle: $name) -> Self {
                handle.0
            }
        }
        impl From<$crate::utils::uid::UID> for $name {
            fn from(uid: $crate::utils::uid::UID) -> Self {
                Self(uid.into())
            }
        }
        impl From<$name> for $crate::utils::uid::UID {
            fn from(handle: $name) -> Self {
                $crate::utils::uid::UID::from(handle.0)
            }
        }
    };
}

#[macro_export]
macro_rules! define_resource_handle {
    ($name:ident) => {
        pub struct $name(pub(crate) $crate::resource::handle::ResourceHandle);

        impl $crate::resource::handle::ToResourceHandle for $name {
            fn to_handle(&self) -> $crate::resource::handle::ResourceHandle {
                self.0
            }
        }

        impl From<$crate::resource::handle::ResourceHandle> for $name {
            fn from(handle: $crate::resource::handle::ResourceHandle) -> Self {
                Self(handle)
            }
        }

        impl $crate::serialize::Serialize for $name {
            type Header = ();
            fn serialize(
                &self,
                encoder: &mut impl $crate::serialize::Encoder,
            ) -> Result<(), $crate::serialize::EncoderError> {
                self.0.serialize(encoder)
            }
            fn deserialize(
                decoder: &mut impl $crate::serialize::Decoder,
                header: &Self::Header,
            ) -> Result<(), $crate::serialize::DecoderError> {
                Self($crate::resource::handle::ResourceHandle::deserialize(
                    decoder, header,
                ))
            }
        }
    };
}
