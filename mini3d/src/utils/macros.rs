#[macro_export]
macro_rules! define_provider_handle {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u64);
        impl $name {
            pub fn null() -> Self {
                Self(0)
            }
        }
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
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub(crate) $crate::resource::handle::ResourceHandle);

        impl $name {
            pub fn null() -> Self {
                Self($crate::resource::handle::ResourceHandle::null())
            }
            pub fn resolve(&mut self, resolver: &mut $crate::resource::handle::ReferenceResolver) {
                self.0.resolve(resolver);
            }
        }

        impl $crate::resource::handle::ToResourceHandle for $name {
            fn to_handle(&self) -> $crate::resource::handle::ResourceHandle {
                self.0
            }
            fn from_handle(handle: $crate::resource::handle::ResourceHandle) -> Self {
                Self(handle)
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
            ) -> Result<Self, $crate::serialize::DecoderError> {
                Ok(Self($crate::resource::handle::ResourceHandle::deserialize(
                    decoder,
                    &Default::default(),
                )?))
            }
        }
    };
}

#[macro_export]
macro_rules! slot_map_key {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name($crate::utils::slotmap::DefaultKey);

        impl $crate::utils::slotmap::Key for $name {
            type Version = $crate::utils::slotmap::DefaultKeyVersion;
            type Index = $crate::utils::slotmap::DefaultKeyIndex;

            fn new(index: Self::Index, version: Self::Version) -> Self {
                Self($crate::utils::slotmap::DefaultKey::new(index, version))
            }

            fn index(&self) -> Self::Index {
                self.0.index()
            }

            fn version(&self) -> Self::Version {
                self.0.version()
            }

            fn null() -> Self {
                Self($crate::utils::slotmap::DefaultKey::null())
            }

            fn is_null(&self) -> bool {
                self.0.is_null()
            }
        }
    };
}
