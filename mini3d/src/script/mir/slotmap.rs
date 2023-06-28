pub(crate) struct SlotId<V> {
    value: u16,
    _marker: core::marker::PhantomData<V>,
}

impl<V> Clone for SlotId<V> {
    fn clone(&self) -> Self {
        Self {
            value: self.value,
            _marker: self._marker,
        }
    }
}

impl<V> Copy for SlotId<V> {}

impl<V> SlotId<V> {
    fn new(value: u16) -> Self {
        Self {
            value: value + 1,
            _marker: core::marker::PhantomData,
        }
    }

    fn index(&self) -> usize {
        assert!(self.value > 0);
        (self.value - 1) as usize
    }

    pub(crate) fn null() -> Self {
        Self {
            value: 0,
            _marker: core::marker::PhantomData,
        }
    }

    pub(crate) fn is_null(&self) -> bool {
        self.value == 0
    }
}

impl<V> From<u16> for SlotId<V> {
    fn from(value: u16) -> Self {
        Self::new(value)
    }
}

impl<V> From<SlotId<V>> for u16 {
    fn from(value: SlotId<V>) -> Self {
        value.value
    }
}

impl<V> Default for SlotId<V> {
    fn default() -> Self {
        Self::null()
    }
}

struct SlotEntry<V> {
    value: V,
}

pub(crate) struct SlotMap<V> {
    entries: Vec<SlotEntry<V>>,
}

impl<V> Default for SlotMap<V> {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl<V> SlotMap<V> {
    pub(crate) fn add(&mut self, value: V) -> SlotId<V> {
        let index = self.entries.len();
        self.entries.push(SlotEntry { value });
        SlotId::new(index as u16)
    }

    pub(crate) fn get(&self, id: SlotId<V>) -> &V {
        &self.entries[id.index()].value
    }

    pub(crate) fn get_mut(&mut self, id: SlotId<V>) -> &mut V {
        &mut self.entries[id.index()].value
    }
}
