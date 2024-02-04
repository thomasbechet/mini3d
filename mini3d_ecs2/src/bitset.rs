use alloc::vec::Vec;

pub(crate) type BitIndex = u16;

#[derive(Default, Clone, Copy)]
struct Block(u32);

impl Block {
    pub(crate) const BITS: u16 = 32;

    #[inline]
    fn mask(mask: u32) -> Self {
        Self(mask)
    }

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
}

#[derive(Default)]
pub(crate) struct Bitset {
    blocks: Vec<Block>,
}

impl Bitset {
    pub(crate) const MAX_BITS: usize = 16 * 32 * 32;
    pub(crate) const OFFSET_MASK: u16 = Block::BITS - 1;
    pub(crate) const L0_BLOCK_COUNT: usize = 1;
    pub(crate) const L0L1_BLOCK_COUNT: usize = Self::L0_BLOCK_COUNT + 16;

    pub(crate) const fn l1_block_index(bit: BitIndex) -> usize {
        (bit / (Block::BITS * Block::BITS)) as usize + 1
    }

    pub(crate) const fn l2_block_index(bit: BitIndex) -> usize {
        (bit / Block::BITS) as usize + Self::L0L1_BLOCK_COUNT
    }

    pub fn new() -> Self {
        Self::with_capacity(256)
    }

    pub fn capacity(&self) -> usize {
        (self.blocks.len() - Self::L0L1_BLOCK_COUNT) * Block::BITS as usize
    }

    pub(crate) fn with_capacity(capacity: u16) -> Self {
        assert!(capacity > 0);
        let l2_len = (capacity + Block::BITS - 1) / Block::BITS;
        Self {
            blocks: alloc::vec![Block::empty(); Self::L0L1_BLOCK_COUNT + l2_len as usize],
        }
    }

    pub(crate) fn is_set(&self, bit: BitIndex) -> bool {
        let l2_index = Self::l2_block_index(bit);
        let l2_offset = bit & Self::OFFSET_MASK;
        self.blocks
            .get(l2_index)
            .map(|block| block.is_set(l2_offset))
            .unwrap_or(false)
    }

    pub(crate) fn set(&mut self, bit: BitIndex, set: bool) {
        assert!(bit < Self::MAX_BITS as u16);

        // Compute block indices
        let l2_index = Self::l2_block_index(bit);
        let l2_offset = bit & Self::OFFSET_MASK;
        let l1_index = Self::l1_block_index(bit);
        let l1_offset = (l2_index - Self::L0L1_BLOCK_COUNT) as u16 & Self::OFFSET_MASK;
        let l0_offset = (l1_index - Self::L0_BLOCK_COUNT) as u16 & Self::OFFSET_MASK;

        // Resize blocks
        if l2_index >= self.blocks.len() {
            self.blocks.resize(l2_index + 1, Block::empty());
        }

        // Update l2
        self.blocks[l2_index].set(l2_offset, set);
        // Update l1
        let l1_bit = !self.blocks[l2_index].is_empty();
        self.blocks[l1_index].set(l1_offset, l1_bit);
        // Update l0
        let l0_bit = !self.blocks[l1_index].is_empty();
        self.blocks[0].set(l0_offset, l0_bit);
    }

    pub(crate) fn iter(&self) -> BitsetIter {
        BitsetIter {
            blocks: &self.blocks,
            iter: BlockIter::new(self.blocks[0].0),
        }
    }
}

enum Level {
    L0,
    L1,
    L2,
}

enum IterAnswer {
    Request(usize),
    Some(BitIndex),
    End,
}

struct BlockIter {
    masks: (u32, u32, u32),
    indices: (u8, u8, u8),
    level: Level,
}

impl BlockIter {
    fn new(mask0: u32) -> Self {
        Self {
            masks: (mask0, 0, 0),
            indices: (0, 0, 0),
            level: Level::L0,
        }
    }

    fn set_block(&mut self, block: Block) {
        match self.level {
            Level::L0 => {
                self.masks.0 = block.0;
                self.indices.0 = 0;
            }
            Level::L1 => {
                self.masks.1 = block.0;
                self.indices.1 = 0;
            }
            Level::L2 => {
                self.masks.2 = block.0;
                self.indices.2 = 0;
            }
        }
    }

    fn next(&mut self) -> IterAnswer {
        loop {
            match self.level {
                Level::L0 => {
                    while self.masks.0 != 0 {
                        let set = self.masks.0 & 1;
                        let index = self.indices.0;
                        self.masks.0 >>= 1;
                        self.indices.0 += 1;
                        if set != 0 {
                            self.level = Level::L1;
                            return IterAnswer::Request(Bitset::L0_BLOCK_COUNT + index as usize);
                        }
                    }
                    return IterAnswer::End;
                }
                Level::L1 => {
                    while self.masks.1 != 0 {
                        let set = self.masks.1 & 1;
                        let index = self.indices.1;
                        self.masks.1 >>= 1;
                        self.indices.1 += 1;
                        if set != 0 {
                            self.level = Level::L2;
                            return IterAnswer::Request(Bitset::L0L1_BLOCK_COUNT + index as usize);
                        }
                    }
                    self.level = Level::L0;
                }
                Level::L2 => {
                    while self.masks.2 != 0 {
                        let set = self.masks.2 & 1;
                        self.masks.2 >>= 1;
                        self.indices.2 += 1;
                        if set != 0 {
                            return IterAnswer::Some(
                                (self.indices.0 - 1) as u16 * 32 * 32
                                    + (self.indices.1 - 1) as u16 * 32
                                    + (self.indices.2 - 1) as u16,
                            );
                        }
                    }
                    self.level = Level::L1;
                }
            }
        }
    }
}

pub(crate) struct BitsetIter<'a> {
    blocks: &'a [Block],
    iter: BlockIter,
}

impl<'a> Iterator for BitsetIter<'a> {
    type Item = BitIndex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                IterAnswer::Request(index) => {
                    self.iter.set_block(self.blocks[index]);
                }
                IterAnswer::Some(index) => {
                    return Some(index);
                }
                IterAnswer::End => {
                    return None;
                }
            }
        }
    }
}

pub(crate) struct BitsetQuery<'a> {
    all: &'a [Bitset],
    any: &'a [Bitset],
    not: &'a [Bitset],
    iter: BlockIter,
}

impl<'a> BitsetQuery<'a> {
    pub(crate) fn new(all: &'a [Bitset], any: &'a [Bitset], not: &'a [Bitset]) -> BitsetQuery<'a> {
        BitsetQuery {
            all,
            any,
            not,
            iter: BlockIter::new(all[0].blocks[0].0),
        }
    }
}

impl<'a> Iterator for BitsetQuery<'a> {
    type Item = BitIndex;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                IterAnswer::Request(index) => {
                    let mut mask = self.all[0].blocks[index].0;
                    for b in self.all.iter().skip(1) {
                        mask &= b.blocks[index].0;
                    }
                    for b in self.any.iter() {
                        mask |= b.blocks[index].0;
                    }
                    self.iter.set_block(Block::mask(mask));
                }
                IterAnswer::Some(index) => {
                    for not in self.not {
                        if not.is_set(index) {
                            continue;
                        }
                    }
                    return Some(index);
                }
                IterAnswer::End => {
                    return None;
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

    #[test]
    fn test_query_all() {
        let mut b0 = Bitset::new();
        let mut b1 = Bitset::new();
        let mut b2 = Bitset::new();
        b0.set(0, true);
        b0.set(1, false);
        b0.set(2, true);
        b1.set(0, true);
        b1.set(1, true);
        b1.set(2, true);
        b2.set(0, true);
        b2.set(1, false);
        b2.set(2, true);
        let all = &[b0, b1, b2];
        let mut it = BitsetQuery::new(all, &[], &[]);
        assert!(it.next() == Some(0));
        assert!(it.next() == Some(2));
        assert!(it.next().is_none());
    }
}
