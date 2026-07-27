use std::ops::{Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};

use crate::SubByteIter;
use crate::ord::BitOrd;
use crate::width::BitWidth;

/// # SubByteIndex
///
/// Immutable bit subset indexing.
///
/// `SubByteIndex` handles the extraction and alignment of a subset of bits. See
/// [`SubByteIndex::subbyte_index`] for usage and examples.
pub trait SubByteIndex<Index> {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>
    where
        Self: 'a;

    /// Get a subset of bits at the given index.
    ///
    /// This operation requires specifying a bit width `W` and the *array* and *output* bit ordering
    /// (`AOrd` and `OOrd` respectively) as generic arguments. Similar to [`Index`](std::ops::Index)
    /// operations on slices, `SubByteIndex` supports indexing by both `usize` and range types
    /// (`Range`, `RangeFrom`, `RangeInclusive`, `RangeTo`, `RangeToInclusive` and `RangeFull`. This
    /// operation considers the entirety of a byte slice to be in-bounds and panics only on indices
    /// outside of the slice.
    ///
    /// # Example
    /// ```
    /// # use subbyte_index::SubByteIndex;
    /// # use subbyte_index::{Msb, Lsb, W2, W4};
    ///
    // A two byte array, represented as two bit subsets in the following operations.
    /// let array_w2 = [0b_00_01_10_11, 0b_11_10_01_00];
    /// // Msb indices:     0  1  2  3      4  5  6  7
    /// // Lsb indices:     3  2  1  0      7  6  5  4
    ///
    /// // Msb indexing, output to the Msb.
    /// assert_eq!(array_w2.subbyte_index::<W2, Msb, Msb>(0), 0b_00_000000);
    /// // Lsb indexing, output to the Lsb.
    /// assert_eq!(array_w2.subbyte_index::<W2, Lsb, Lsb>(3..=6).collect::<Vec<u8>>(),
    ///     [0b_00, 0b_00, 0b_01, 0b_10]);
    /// ```
    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: Index,
    ) -> Self::Output<'_, W, AOrd, OOrd>;
}

impl SubByteIndex<usize> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = u8;

    #[inline]
    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: usize,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        let slot_mask = AOrd::select_slot(index % W::N_SLOTS);
        OOrd::slot_to_bound(slot_mask, self[index / W::N_SLOTS] & slot_mask)
    }
}

impl SubByteIndex<Range<usize>> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = SubByteIter<'a, W, AOrd, OOrd>;

    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: Range<usize>,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        SubByteIter::new(self, index.start, index.end)
    }
}

impl SubByteIndex<RangeInclusive<usize>> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = SubByteIter<'a, W, AOrd, OOrd>;

    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: RangeInclusive<usize>,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        SubByteIter::new(self, *index.start(), index.end() + 1)
    }
}

impl SubByteIndex<RangeFrom<usize>> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = SubByteIter<'a, W, AOrd, OOrd>;

    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: RangeFrom<usize>,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        SubByteIter::new(self, index.start, self.len() * W::N_SLOTS)
    }
}

impl SubByteIndex<RangeTo<usize>> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = SubByteIter<'a, W, AOrd, OOrd>;

    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: RangeTo<usize>,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        SubByteIter::new(self, 0, index.end)
    }
}

impl SubByteIndex<RangeToInclusive<usize>> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = SubByteIter<'a, W, AOrd, OOrd>;

    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        index: RangeToInclusive<usize>,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        SubByteIter::new(self, 0, index.end + 1)
    }
}

impl SubByteIndex<RangeFull> for [u8] {
    type Output<'a, W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>> = SubByteIter<'a, W, AOrd, OOrd>;

    fn subbyte_index<W: BitWidth, AOrd: BitOrd<W>, OOrd: BitOrd<W>>(
        &self,
        _: RangeFull,
    ) -> Self::Output<'_, W, AOrd, OOrd> {
        SubByteIter::new(self, 0, self.len() * W::N_SLOTS)
    }
}

#[cfg(test)]
mod tests {

    mod index {

        use crate::*;

        #[test]
        fn test_pack2_lsb_lsb() {
            let a0 = [0b0000_0011, 0b0000_0000];
            let a6 = [0b0000_0000, 0b0011_0000];
            let a7 = [0b0000_0000, 0b1100_0000];

            assert_eq!(a0.subbyte_index::<W2, Lsb, Lsb>(0), 0b0000_0011);
            assert_eq!(a6.subbyte_index::<W2, Lsb, Lsb>(6), 0b0000_0011);
            assert_eq!(a7.subbyte_index::<W2, Lsb, Lsb>(7), 0b0000_0011);
        }

        #[test]
        fn test_pack2_msb_lsb() {
            let a0 = [0b1100_0000, 0b0000_0000];
            let a6 = [0b0000_0000, 0b0000_1100];
            let a7 = [0b0000_0000, 0b0000_0011];

            assert_eq!(a0.subbyte_index::<W2, Msb, Lsb>(0), 0b0000_0011);
            assert_eq!(a6.subbyte_index::<W2, Msb, Lsb>(6), 0b0000_0011);
            assert_eq!(a7.subbyte_index::<W2, Msb, Lsb>(7), 0b0000_0011);
        }

        #[test]
        fn test_pack2_lsb_msb() {
            let a0 = [0b0000_0011, 0b0000_0000];
            let a6 = [0b0000_0000, 0b0011_0000];
            let a7 = [0b0000_0000, 0b1100_0000];

            assert_eq!(a0.subbyte_index::<W2, Lsb, Msb>(0), 0b1100_0000);
            assert_eq!(a6.subbyte_index::<W2, Lsb, Msb>(6), 0b1100_0000);
            assert_eq!(a7.subbyte_index::<W2, Lsb, Msb>(7), 0b1100_0000);
        }

        #[test]
        fn test_pack2_msb_msb() {
            let a0 = [0b1100_0000, 0b0000_0000];
            let a6 = [0b0000_0000, 0b0000_1100];
            let a7 = [0b0000_0000, 0b0000_0011];

            assert_eq!(a0.subbyte_index::<W2, Msb, Msb>(0), 0b1100_0000);
            assert_eq!(a6.subbyte_index::<W2, Msb, Msb>(6), 0b1100_0000);
            assert_eq!(a7.subbyte_index::<W2, Msb, Msb>(7), 0b1100_0000);
        }

        #[test]
        fn test_pack4_lsb_lsb() {
            let a0 = [0b0000_1101, 0b0000_0000];
            let a2 = [0b0000_0000, 0b0000_1101];
            let a3 = [0b0000_0000, 0b1101_0000];

            assert_eq!(a0.subbyte_index::<W4, Lsb, Lsb>(0), 0b0000_1101);
            assert_eq!(a2.subbyte_index::<W4, Lsb, Lsb>(2), 0b0000_1101);
            assert_eq!(a3.subbyte_index::<W4, Lsb, Lsb>(3), 0b0000_1101);
        }

        #[test]
        fn test_pack4_msb_lsb() {
            let a0 = [0b1101_0000, 0b0000_0000];
            let a2 = [0b0000_0000, 0b1101_0000];
            let a3 = [0b0000_0000, 0b0000_1101];

            assert_eq!(a0.subbyte_index::<W4, Msb, Lsb>(0), 0b0000_1101);
            assert_eq!(a2.subbyte_index::<W4, Msb, Lsb>(2), 0b0000_1101);
            assert_eq!(a3.subbyte_index::<W4, Msb, Lsb>(3), 0b0000_1101);
        }

        #[test]
        fn test_pack4_lsb_msb() {
            let a0 = [0b0000_1101, 0b0000_0000];
            let a2 = [0b0000_0000, 0b0000_1101];
            let a3 = [0b0000_0000, 0b1101_0000];

            assert_eq!(a0.subbyte_index::<W4, Lsb, Msb>(0), 0b1101_0000);
            assert_eq!(a2.subbyte_index::<W4, Lsb, Msb>(2), 0b1101_0000);
            assert_eq!(a3.subbyte_index::<W4, Lsb, Msb>(3), 0b1101_0000);
        }

        #[test]
        fn test_pack4_msb_msb() {
            let a0 = [0b1101_0000, 0b0000_0000];
            let a2 = [0b0000_0000, 0b1101_0000];
            let a3 = [0b0000_0000, 0b0000_1101];

            assert_eq!(a0.subbyte_index::<W4, Msb, Msb>(0), 0b1101_0000);
            assert_eq!(a2.subbyte_index::<W4, Msb, Msb>(2), 0b1101_0000);
            assert_eq!(a3.subbyte_index::<W4, Msb, Msb>(3), 0b1101_0000);
        }
    }

    mod range {

        use crate::*;

        #[test]
        fn test_pack2_lsb_lsb_iter() {
            let a = [0b0011_1001, 0b0001_1011];

            assert_eq!(
                a.subbyte_index::<W2, Lsb, Lsb>(0..3).collect::<Vec<_>>(),
                [0b01, 0b10, 0b11]
            );
            assert_eq!(
                a.subbyte_index::<W2, Lsb, Lsb>(3..8).collect::<Vec<_>>(),
                [0b00, 0b11, 0b10, 0b01, 0b00]
            );
        }

        #[test]
        fn test_pack2_msb_lsb_iter() {
            let a = [0b0110_1100, 0b1110_0100];

            assert_eq!(
                a.subbyte_index::<W2, Msb, Lsb>(0..3).collect::<Vec<_>>(),
                [0b01, 0b10, 0b11]
            );
            assert_eq!(
                a.subbyte_index::<W2, Msb, Lsb>(3..8).collect::<Vec<_>>(),
                [0b00, 0b11, 0b10, 0b01, 0b00]
            );
        }

        #[test]
        fn test_pack2_lsb_msb_iter() {
            let a = [0b0011_1001, 0b0001_1011];

            assert_eq!(
                a.subbyte_index::<W2, Lsb, Msb>(0..3).collect::<Vec<_>>(),
                [0b0100_0000, 0b1000_0000, 0b1100_0000]
            );
            #[rustfmt::skip]
            assert_eq!(
                a.subbyte_index::<W2, Lsb, Msb>(3..8).collect::<Vec<_>>(),
                [0b0000_0000, 0b1100_0000, 0b1000_0000, 0b0100_0000, 0b0000_0000]
            );
        }

        #[test]
        fn test_pack2_msb_msb_iter() {
            let a = [0b0110_1100, 0b1110_0100];

            assert_eq!(
                a.subbyte_index::<W2, Msb, Msb>(0..3).collect::<Vec<_>>(),
                [0b0100_0000, 0b1000_0000, 0b1100_0000]
            );
            #[rustfmt::skip]
            assert_eq!(
                a.subbyte_index::<W2, Msb, Msb>(3..8).collect::<Vec<_>>(),
                [0b0000_0000, 0b1100_0000, 0b1000_0000, 0b0100_0000, 0b0000_0000]
            );
        }

        #[test]
        fn test_pack4_lsb_lsb_iter() {
            let a = [0b0010_0001, 0b0100_0011, 0b0000_0101];

            assert_eq!(
                a.subbyte_index::<W4, Lsb, Lsb>(0..4).collect::<Vec<_>>(),
                [0b0001, 0b0010, 0b0011, 0b0100]
            );
            assert_eq!(
                a.subbyte_index::<W4, Lsb, Lsb>(3..6).collect::<Vec<_>>(),
                [0b0100, 0b0101, 0b0000]
            );
        }

        #[test]
        fn test_pack4_msb_lsb_iter() {
            let a = [0b0001_0010, 0b0011_0100, 0b0101_0000];

            assert_eq!(
                a.subbyte_index::<W4, Msb, Lsb>(0..4).collect::<Vec<_>>(),
                [0b0001, 0b0010, 0b0011, 0b0100]
            );
            assert_eq!(
                a.subbyte_index::<W4, Msb, Lsb>(3..6).collect::<Vec<_>>(),
                [0b0100, 0b0101, 0b0000]
            );
        }

        #[test]
        fn test_pack4_lsb_msb_iter() {
            let a = [0b0010_0001, 0b0100_0011, 0b0000_0101];

            assert_eq!(
                a.subbyte_index::<W4, Lsb, Msb>(0..4).collect::<Vec<_>>(),
                [0b0001_0000, 0b0010_0000, 0b0011_0000, 0b0100_0000]
            );
            assert_eq!(
                a.subbyte_index::<W4, Lsb, Msb>(3..6).collect::<Vec<_>>(),
                [0b0100_0000, 0b0101_0000, 0b0000_0000]
            );
        }

        #[test]
        fn test_pack4_msb_msb_iter() {
            let a = [0b0001_0010, 0b0011_0100, 0b0101_0000];

            assert_eq!(
                a.subbyte_index::<W4, Msb, Msb>(0..4).collect::<Vec<_>>(),
                [0b0001_0000, 0b0010_0000, 0b0011_0000, 0b0100_0000]
            );
            assert_eq!(
                a.subbyte_index::<W4, Msb, Msb>(3..6).collect::<Vec<_>>(),
                [0b0100_0000, 0b0101_0000, 0b0000_0000]
            );
        }

        #[test]
        fn range_parity() {
            let a = [0b0010_0001, 0b0100_0011];
            let lsb_out = [0b0001, 0b0010, 0b0011, 0b0100];

            #[rustfmt::skip]
            assert_eq!(a.subbyte_index::<W4, Lsb, Lsb>(0..4).collect::<Vec<_>>(), lsb_out);
            #[rustfmt::skip]
            assert_eq!(a.subbyte_index::<W4, Lsb, Lsb>(0..).collect::<Vec<_>>(), lsb_out);
            #[rustfmt::skip]
            assert_eq!(a.subbyte_index::<W4, Lsb, Lsb>(..4).collect::<Vec<_>>(), lsb_out);
            #[rustfmt::skip]
            assert_eq!(a.subbyte_index::<W4, Lsb, Lsb>(..).collect::<Vec<_>>(), lsb_out);
        }
    }
}
