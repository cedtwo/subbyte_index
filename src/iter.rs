use std::marker::PhantomData;

use crate::ord::BitOrd;
use crate::width::BitWidth;

/// Immutable subbyte range iterator.
pub struct SubByteIter<'a, W, AOrd, OOrd> {
    /// The byte slice range.
    slice: &'a [u8],
    /// The current byte (The first element of the byte slice range).
    byte: u8,
    /// The byte slot index for the current byte.
    slot_idx: usize,
    /// The number of elements to return.
    len: usize,
    /// The [`PackWidth`].
    _width: PhantomData<W>,
    /// The [`BitOrd`] alignment of the array and resulting type (`B0` and `B1` respectively).
    _bitord: PhantomData<(AOrd, OOrd)>,
}

impl<'a, W: BitWidth, B0, B1> SubByteIter<'a, W, B0, B1> {
    pub fn new(slice: &'a [u8], start: usize, end: usize) -> Self {
        let slice = &slice[start / W::N_SLOTS..(end - 1) / W::N_SLOTS + 1];
        let byte = slice[0];
        let slot_idx = start % W::N_SLOTS;

        Self {
            slice,
            byte,
            slot_idx,
            len: end - start,
            _width: PhantomData,
            _bitord: PhantomData,
        }
    }
}

impl<'a, W: BitWidth, B0: BitOrd<W>, B1: BitOrd<W>> Iterator for SubByteIter<'a, W, B0, B1> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            if self.slot_idx >= W::N_SLOTS {
                self.slot_idx = self.slot_idx % W::N_SLOTS;
                self.slice = &self.slice[1..];
                self.byte = self.slice[0];
            }

            let slot_mask = B0::select_slot(self.slot_idx);
            let mask = B1::slot_to_bound(slot_mask, self.byte & slot_mask);

            self.len -= 1;
            self.slot_idx += 1;

            return Some(mask);
        }

        None
    }
}
