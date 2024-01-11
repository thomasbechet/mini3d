use alloc::vec::Vec;

#[derive(Default, Clone, Copy)]
struct Block(u32);

impl Block {
    pub(crate) const BITS: u32 = 32;

    #[inline]
    fn empty() -> Self {
        Self(0)
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    #[inline]
    fn set(&mut self, bit: u32, set: bool) {
        if set {
            self.0 |= 1 << bit;
        } else {
            self.0 &= !(1 << bit);
        }
    }

    #[inline]
    fn is_set(&self, bit: u32) -> bool {
        self.0 & (1 << bit) != 0
    }
}

pub(crate) struct Bitset {
    l0: Block,
    l1: Vec<Block>,
    l2: Vec<Block>,
}

impl Bitset {
    pub(crate) const OFFSET_MASK: u32 = Block::BITS - 1;

    pub(crate) fn new() -> Self {
        Self::with_capacity(256)
    }

    pub(crate) fn with_capacity(capacity: u32) -> Self {
        assert!(capacity > 0);
        let l2_len = (capacity + Block::BITS - 1) / Block::BITS;
        let l1_len = (l2_len + Block::BITS - 1) / Block::BITS;
        Self {
            l0: Block::empty(),
            l1: Vec::with_capacity(l1_len as usize),
            l2: Vec::with_capacity(l2_len as usize),
        }
    }

    pub(crate) fn capacity(&self) -> u32 {
        self.l2.capacity() as u32 * Block::BITS
    }

    pub(crate) fn is_set(&self, bit: u32) -> bool {
        let l2_index = bit / Block::BITS;
        let l2_offset = bit & Self::OFFSET_MASK;
        self.l2
            .get(l2_index as usize)
            .map(|block| block.is_set(l2_offset))
            .unwrap_or(false)
    }

    fn set(&mut self, bit: u32, set: bool) {
        let l2_index = bit / Block::BITS;
        let l2_offset = bit & Self::OFFSET_MASK;
        if l2_index >= self.l2.len() as u32 {
            self.l2.resize((l2_index + 1) as usize, Block::empty());
        }
        self.l2[l2_index as usize].set(l2_offset, set);
        // Update l1
        let l1_bit = !self.l2[l2_index as usize].is_empty();
        let l1_index = l2_index / Block::BITS;
        let l1_offset = l2_index & Self::OFFSET_MASK;
        if l1_index >= self.l1.len() as u32 {
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
            l0mask: 1,
            l1: 0,
            l1mask: 0,
            l2: 0,
            l2mask: 0,
            index: 0,
        }
    }
}

pub(crate) struct BitsetIter<'a> {
    bitset: &'a Bitset,
    l0mask: u32,
    l1: u32,
    l1mask: u32,
    l2: u32,
    l2mask: u32,
    index: u32,
}

impl<'a> Iterator for BitsetIter<'a> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            // Check l2 mask
            if self.l2mask != 0 && (self.bitset.l2[self.l2 as usize].0 & self.l2mask) != 0 {
                let index = self.index;
                self.l2mask <<= 1;
                self.index += 1;
                return Some(index);
            }

            // Check l1 mask
            if self.l1mask != 0 {
                if (self.bitset.l1[self.l1 as usize].0 & self.l1mask) == 0 {
                    self.l1mask <<= 1;
                    self.l2 += 1;
                    self.index += Block::BITS;
                    if self.l2 >= self.bitset.l2.len() as u32 {
                        return None;
                    }
                } else {
                    self.l2mask = 1;
                }
                continue;
            }

            // Check l0 mask
            if self.l0mask != 0 {
                if (self.bitset.l0.0 & self.l0mask) == 0 {
                    self.l0mask <<= 1;
                    self.l1 += 1;
                    self.index += Block::BITS * Block::BITS;
                    if self.l1 >= self.bitset.l1.len() as u32 {
                        return None;
                    }
                } else {
                    self.l1mask = 1;
                }
                continue;
            }

            return None;
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
}
