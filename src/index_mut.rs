use std::ops::{Range, RangeFrom, RangeFull, RangeTo};

use crate::ord::BitOrd;
use crate::width::BitWidth;

pub trait SubByteIndexMut<Index, Input> {
    /// Set a subset of bits at the given index.
    ///
    /// This operation requires specifying a bit width `W` and the *array* and *input* bit ordering
    /// (`AOrd` and `IOrd` respectively) as generic arguments. Similar to [`Index`](std::ops::Index)
    /// operations on slices, `SubByteIndexMut` supports indexing by both `usize` and range types
    /// (`Range`, `RangeFrom`, `RangeInclusive`, `RangeTo`, `RangeToInclusive` and `RangeFull`.
    ///
    /// Input expects `u8` byte mask(s) of the given `IOrd` alignment. Bits outside of this range
    /// are truncated. For a `usize` index, a single `u8` is expected as input, while for range
    /// indices, a type implementing `IntoIterator<Item = u8>` of a length `>=` the index range is
    /// expected.
    ///
    /// # Example
    /// ```
    /// # use subbyte_index::{SubByteIndex, SubByteIndexMut};
    /// # use subbyte_index::{Msb, Lsb, W2, W4};
    ///
    /// // An 8 byte vector, represented as four bit subsets in the following operations.
    /// let mut array_w4 = vec![0; 8];
    ///
    /// // Set each 4-bit element to its index.
    /// array_w4.as_mut_slice().subbyte_index_mut::<W4, Lsb, Lsb>(.., (0..));
    /// assert_eq!(array_w4.as_slice().subbyte_index::<W4, Lsb, Lsb>(..).collect::<Vec<_>>(), Vec::from_iter(0..16));
    ///
    /// // Set indices `5..8` to zero.
    /// array_w4.as_mut_slice().subbyte_index_mut::<W4, Lsb, Lsb>(5..8, [0].into_iter().cycle());
    /// assert_eq!(
    ///     array_w4
    ///         .subbyte_index::<W4, Lsb, Lsb>(..)
    ///         .collect::<Vec<u8>>(),
    ///     [0, 1, 2, 3, 4, 0, 0, 0, 8, 9, 10, 11, 12, 13, 14, 15]
    /// );
    /// assert_eq!(array_w4.len(), 8);
    /// ```
    fn subbyte_index_mut<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>>(
        &mut self,
        index: Index,
        input: Input,
    );
}

impl SubByteIndexMut<usize, u8> for [u8] {
    fn subbyte_index_mut<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>>(
        &mut self,
        index: usize,
        input: u8,
    ) {
        let slot_idx = index % W::N_SLOTS;

        self[index / W::N_SLOTS] = self[index / W::N_SLOTS] & AOrd::deselect_slot(slot_idx)
            | AOrd::bound_to_slot(
                AOrd::slot_to_bound(IOrd::MASK, input & IOrd::MASK),
                slot_idx,
            );
    }
}

impl<I: IntoIterator<Item = u8>> SubByteIndexMut<Range<usize>, I> for [u8] {
    fn subbyte_index_mut<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>>(
        &mut self,
        index: Range<usize>,
        input: I,
    ) {
        iter_map::<W, AOrd, IOrd, I>(self, index.start, index.end, input);
    }
}

impl<I: IntoIterator<Item = u8>> SubByteIndexMut<RangeFrom<usize>, I> for [u8] {
    fn subbyte_index_mut<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>>(
        &mut self,
        index: RangeFrom<usize>,
        input: I,
    ) {
        iter_map::<W, AOrd, IOrd, I>(self, index.start, self.len() * W::N_SLOTS, input);
    }
}

impl<I: IntoIterator<Item = u8>> SubByteIndexMut<RangeTo<usize>, I> for [u8] {
    fn subbyte_index_mut<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>>(
        &mut self,
        index: RangeTo<usize>,
        input: I,
    ) {
        iter_map::<W, AOrd, IOrd, I>(self, 0, index.end, input);
    }
}

impl<I: IntoIterator<Item = u8>> SubByteIndexMut<RangeFull, I> for [u8] {
    fn subbyte_index_mut<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>>(
        &mut self,
        _: RangeFull,
        input: I,
    ) {
        iter_map::<W, AOrd, IOrd, I>(self, 0, self.len() * W::N_SLOTS, input);
    }
}

fn iter_map<W, AOrd, IOrd, T>(slice: &mut [u8], start: usize, end: usize, input: T)
where
    W: BitWidth,
    AOrd: BitOrd<W>,
    IOrd: BitOrd<W>,
    T: IntoIterator<Item = u8>,
{
    const INPUT_RANGE_ERR: &'static str = "Expected an input range >= the given slice range";
    let mut iter = input.into_iter();

    let mut slice = &mut slice[start / W::N_SLOTS..(end - 1) / W::N_SLOTS + 1];
    let mut byte = slice[0];
    let mut slot_idx = start % W::N_SLOTS;
    let mut len = end - start;

    while len > 0 {
        if slot_idx >= W::N_SLOTS {
            slice[0] = byte;

            slot_idx = slot_idx % W::N_SLOTS;
            slice = &mut slice[1..];
            byte = slice[0];
        }

        byte = byte & AOrd::deselect_slot(slot_idx)
            | AOrd::bound_to_slot(
                AOrd::slot_to_bound(IOrd::MASK, iter.next().expect(INPUT_RANGE_ERR) & IOrd::MASK),
                slot_idx,
            );

        len -= 1;
        slot_idx += 1;
    }

    if !slice.is_empty() {
        slice[0] = byte;
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn set_usize_lsb_lsb() {
        let mut arr = [0b0010_0001, 0b0100_0011];
        arr.subbyte_index_mut::<W4, Lsb, Lsb>(1, 0b1111);
        arr.subbyte_index_mut::<W4, Lsb, Lsb>(2, 0b1111);

        assert_eq!(arr, [0b1111_0001, 0b0100_1111]);
    }

    #[test]
    fn set_usize_lsb_msb() {
        let mut arr = [0b0010_0001, 0b0100_0011];
        arr.subbyte_index_mut::<W4, Lsb, Msb>(1, 0b1111_0000);
        arr.subbyte_index_mut::<W4, Lsb, Msb>(2, 0b1111_0000);

        assert_eq!(arr, [0b1111_0001, 0b0100_1111]);
    }

    #[test]
    fn set_usize_msb_lsb() {
        let mut arr = [0b0010_0001, 0b0100_0011];
        arr.subbyte_index_mut::<W4, Msb, Lsb>(1, 0b1111);
        arr.subbyte_index_mut::<W4, Msb, Lsb>(2, 0b1111);

        assert_eq!(arr, [0b0010_1111, 0b1111_0011]);
    }

    #[test]
    fn set_usize_msb_msb() {
        let mut arr = [0b0010_0001, 0b0100_0011];
        arr.subbyte_index_mut::<W4, Msb, Msb>(1, 0b1111_0000);
        arr.subbyte_index_mut::<W4, Msb, Msb>(2, 0b1111_0000);

        assert_eq!(arr, [0b0010_1111, 0b1111_0011]);
    }

    #[test]
    fn set_range_lsb_lsb() {
        let mut arr = [0b1111_1111, 0b1111_1111];
        arr.subbyte_index_mut::<W4, Lsb, Lsb>(1..3, 0..);

        assert_eq!(arr, [0b0000_1111, 0b1111_0001]);
    }

    #[test]
    fn set_range_msb_lsb() {
        let mut arr = [0b1111_1111, 0b1111_1111];
        arr.subbyte_index_mut::<W4, Msb, Lsb>(1..3, 0..);

        assert_eq!(arr, [0b1111_0000, 0b0001_1111]);
    }

    #[test]
    fn set_range_lsb_msb() {
        let mut arr = [0b1111_1111, 0b1111_1111];
        arr.subbyte_index_mut::<W4, Lsb, Msb>(1..3, [0b0000_0000, 0b0001_0000, 0b0010_0000]);

        assert_eq!(arr, [0b0000_1111, 0b1111_0001]);
    }

    #[test]
    fn set_range_msb_msb() {
        let mut arr = [0b1111_1111, 0b1111_1111];
        arr.subbyte_index_mut::<W4, Msb, Msb>(1..3, [0b0000_0000, 0b0001_0000, 0b0010_0000]);

        assert_eq!(arr, [0b1111_0000, 0b0001_1111]);
    }

    #[test]
    fn range_parity() {
        /// Clones, mutates then returns the given `array`.
        fn clone_set_array<W: BitWidth, AOrd: BitOrd<W>, IOrd: BitOrd<W>, Index, Input>(
            array: &[u8; 2],
            index: Index,
            input: Input,
        ) -> [u8; 2]
        where
            [u8]: SubByteIndexMut<Index, Input>,
        {
            let mut arr = array.clone();
            arr.subbyte_index_mut::<W, AOrd, IOrd>(index, input);
            arr
        }

        let arr = [0b1111_1111, 0b1111_1111];
        let lsb_out = [0b0001_0000, 0b0011_0010];

        #[rustfmt::skip]
        assert_eq!(clone_set_array::<W4, Lsb, Lsb, _, _>(&arr, 0..4, 0..4), lsb_out);
        #[rustfmt::skip]
        assert_eq!(clone_set_array::<W4, Lsb, Lsb, _, _>(&arr, 0.., 0..4), lsb_out);
        #[rustfmt::skip]
        assert_eq!(clone_set_array::<W4, Lsb, Lsb, _, _>(&arr, ..4, 0..4), lsb_out);
        #[rustfmt::skip]
        assert_eq!(clone_set_array::<W4, Lsb, Lsb, _, _>(&arr, .., 0..4), lsb_out);
    }
}
