use super::{key::ResourceSlotKey, ResourceEntryKey};

pub(crate) struct Wrapper {
    pub(crate) entry_key: ResourceEntryKey,
    pub(crate) slot_key: ResourceSlotKey,
}

pub(crate) struct ResourceKeysIterator<'a> {
    pub(crate) iter: std::slice::Iter<'a, Wrapper>,
}

impl<'a> ResourceKeysIterator<'a> {
    pub(crate) fn empty() -> Self {
        Self { iter: &[].iter() }
    }
}

impl<'a> Iterator for ResourceKeysIterator<'a> {
    type Item = (ResourceEntryKey, ResourceSlotKey);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|wrapper| (*wrapper.entry_key, *wrapper.slot_key))
            .copied()
    }
}
