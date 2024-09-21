use core::num::NonZeroU16;
pub(crate) type BitMaskWord = u16;
pub(crate) type NonZeroBitMaskWord = NonZeroU16;
pub(crate) const BITMASK_STRIDE: usize = 1;
pub(crate) const BITMASK_ITER_MASK: BitMaskWord = !0;

#[derive(Copy, Clone)]
pub(crate) struct BitMask(pub(crate) BitMaskWord);

#[allow(clippy::use_self)]
impl BitMask {
    fn remove_lowest_bit(self) -> Self {
        BitMask(self.0 & (self.0 - 1))
    }

    pub(crate) fn any_bit_set(self) -> bool {
        self.0 != 0
    }

    pub(crate) fn lowest_set_bit(self) -> Option<usize> {
        NonZeroBitMaskWord::new(self.0).map(Self::nonzero_trailing_zeros)
    }

    fn nonzero_trailing_zeros(nonzero: NonZeroBitMaskWord) -> usize {
        nonzero.trailing_zeros() as usize / BITMASK_STRIDE
    }
}

impl IntoIterator for BitMask {
    type Item = usize;
    type IntoIter = BitMaskIter;

    fn into_iter(self) -> BitMaskIter {
        // A BitMask only requires each element (group of bits) to be non-zero.
        // However for iteration we need each element to only contain 1 bit.
        BitMaskIter(BitMask(self.0 & BITMASK_ITER_MASK))
    }
}

// Iterator over the contents of a `BitMask`, returning the indices of set
// bits.
#[derive(Copy, Clone)]
pub(crate) struct BitMaskIter(pub(crate) BitMask);

impl Iterator for BitMaskIter {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let bit = self.0.lowest_set_bit()?;
        self.0 = self.0.remove_lowest_bit();
        Some(bit)
    }
}
