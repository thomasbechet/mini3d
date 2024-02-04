use alloc::vec::Vec;

pub(crate) type BitIndex = u16;

#[derive(Default, Clone, Copy)]
struct Block(u32);

impl Block {
    pub(crate) const BITS: u16 = 32;

    #[inline]
    fn empty() -> Self {
        Self(0)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    fn set(&mut self, bit: BitIndex, set: bool) {
        if set {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }

    #[inline]
    fn is_set(&self, bit: BitIndex) -> bool {
        self.0 & (1 << bit) != 0
    }

    #[inline]
    fn iter(&self) -> BlockIter {
        BlockIter {
            mask: self.0,
            index: 0,
        }
    }
}

pub(crate) struct BlockIter {
    mask: u32,
    index: u8,
}

impl BlockIter {
    fn empty() -> Self {
        Self { mask: 0, index: 0 }
    }
}

impl Default for BlockIter {
    fn default() -> Self {
        Self::empty()
    }
}

impl Iterator for BlockIter {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        while self.mask != 0 {
            let set = self.mask & 1;
            self.mask >>= 1;
            self.index += 1;
            if set != 0 {
                return Some(self.index - 1);
            }
        }
        None
    }
}

#[derive(Default)]
pub(crate) struct Bitset {
    l0: Block,
    l1: Vec<Block>,
    l2: Vec<Block>,
}

impl Bitset {
    pub(crate) const OFFSET_MASK: u16 = Block::BITS - 1;

    pub(crate) fn new() -> Self {
        Self::with_capacity(256)
    }

    pub(crate) fn with_capacity(capacity: u16) -> Self {
        assert!(capacity > 0);
        let l2_len = (capacity + Block::BITS - 1) / Block::BITS;
        let l1_len = (l2_len + Block::BITS - 1) / Block::BITS;
        Self {
            l0: Block::empty(),
            l1: Vec::with_capacity(l1_len as usize),
            l2: Vec::with_capacity(l2_len as usize),
        }
    }

    pub(crate) fn capacity(&self) -> u16 {
        self.l2.capacity() as u16 * Block::BITS
    }

    pub(crate) fn is_set(&self, bit: BitIndex) -> bool {
        let l2_index = bit / Block::BITS;
        let l2_offset = bit & Self::OFFSET_MASK;
        self.l2
            .get(l2_index as usize)
            .map(|block| block.is_set(l2_offset))
            .unwrap_or(false)
    }

    pub(crate) fn set(&mut self, bit: BitIndex, set: bool) {
        let l2_index = bit / Block::BITS;
        let l2_offset = bit & Self::OFFSET_MASK;
        if l2_index >= self.l2.len() as u16 {
            self.l2.resize((l2_index + 1) as usize, Block::empty());
        }
        self.l2[l2_index as usize].set(l2_offset, set);
        // Update l1
        let l1_bit = !self.l2[l2_index as usize].is_empty();
        let l1_index = l2_index / Block::BITS;
        let l1_offset = l2_index & Self::OFFSET_MASK;
        if l1_index >= self.l1.len() as u16 {
            self.l1.resize((l1_index + 1) as usize, Block::empty());
        }
        self.l1[l1_index as usize].set(l1_offset, l1_bit);
        // Update l0
        let l0_bit = !self.l1[l1_index as usize].is_empty();
        let l0_offset = l1_index & Self::OFFSET_MASK;
        self.l0.set(l0_offset, l0_bit);
    }

    pub(crate) fn iter(&self) -> BitsetIter {
        BitsetIter {
            bitset: self,
            iter0: self.l0.iter(),
            iter1: BlockIter::empty(),
            iter2: BlockIter::empty(),
            level: Level::L0,
            index: (0, 0, 0),
        }
    }
}

enum Level {
    L0,
    L1,
    L2,
}

enum QueryFilter {
    All,
    Any,
}

pub(crate) struct BitsetIter<'a> {
    bitset: &'a Bitset,
    iter0: BlockIter,
    iter1: BlockIter,
    iter2: BlockIter,
    level: Level,
    index: (u8, u8, u8),
}

impl<'a> Iterator for BitsetIter<'a> {
    type Item = BitIndex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.level {
                Level::L0 => {
                    if let Some(index) = self.iter0.next() {
                        self.index.0 = index;
                        self.iter1 = self.bitset.l1[index as usize].iter();
                        self.level = Level::L1;
                    } else {
                        return None;
                    }
                }
                Level::L1 => {
                    if let Some(index) = self.iter1.next() {
                        self.index.1 = index;
                        self.iter2 = self.bitset.l2[index as usize].iter();
                        self.level = Level::L2;
                    } else {
                        self.level = Level::L0;
                        continue;
                    }
                }
                Level::L2 => {
                    if let Some(index) = self.iter2.next() {
                        self.index.2 = index;
                        return Some(
                            self.index.0 as u16 * Block::BITS * Block::BITS
                                + self.index.1 as u16 * Block::BITS
                                + self.index.2 as u16,
                        );
                    } else {
                        self.level = Level::L1;
                        continue;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_basic() {
        let mut bitset = Bitset::new();
        assert!(bitset.capacity() == 256);
        for i in 33..256 {
            bitset.set(i, true);
        }
        for i in 0..32 {
            assert!(!bitset.is_set(i));
        }
        for i in 33..256 {
            assert!(bitset.is_set(i));
        }
    }

    #[test]
    fn test_iter() {
        let mut bitset = Bitset::new();
        bitset.set(0, true);
        bitset.set(1, false);
        bitset.set(2, false);
        bitset.set(3, true);
        bitset.set(4, true);
        bitset.set(5, true);
        let mut it = bitset.iter();
        assert!(it.next() == Some(0));
        assert!(it.next() == Some(3));
        assert!(it.next() == Some(4));
        assert!(it.next() == Some(5));
        assert!(it.next().is_none());
    }

    #[test]
    fn test_iter2() {
        let mut bitset = Bitset::new();
        bitset.set(33, true);
        bitset.set(259, true);
        let mut it = bitset.iter();
        assert!(it.next() == Some(33));
        assert!(it.next() == Some(259));
        assert!(it.next().is_none());
    }
}
