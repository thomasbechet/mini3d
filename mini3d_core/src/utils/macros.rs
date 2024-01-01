#[macro_export]
macro_rules! define_provider_handle {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
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
macro_rules! slot_map_key {
    ($name:ident) => {
        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name($crate::utils::slotmap::DefaultKey);

        impl $crate::utils::slotmap::Key for $name {
            fn new(index: usize) -> Self {
                Self($crate::utils::slotmap::DefaultKey::new(index))
            }

            fn null() -> Self {
                Self($crate::utils::slotmap::DefaultKey::null())
            }

            fn update(&mut self, index: usize) {
                self.0.update(index);
            }

            fn index(&self) -> Option<usize> {
                self.0.index()
            }
        }
    };
}
