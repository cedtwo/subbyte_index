use crate::width::BitWidth;

/// Indicates element slot ordering for a byte. Used to specify the byte ordering within an array,
/// and the byte ordering for element input/output. See [`Lsb`] and [`Msb`].
pub trait BitOrd<W>: Clone + Copy {
    /// An aligned bitmask of the given [`PackWidth`].
    const MASK: u8;
    const INVERSE: u8 = !Self::MASK;

    /// Return a mask with the bits of the given slot index set.
    fn select_slot(slot_idx: usize) -> u8 {
        Self::bound_to_slot(Self::MASK, slot_idx)
    }

    /// Return a mask with the bits of the given slot index unset.
    fn deselect_slot(slot_idx: usize) -> u8 {
        Self::bound_to_slot(Self::INVERSE, slot_idx)
    }

    /// Shift the given bound aligned mask to the given slot index.
    fn bound_to_slot(mask: u8, slot_idx: usize) -> u8;

    /// Shift the given slot aligned `mask` to the byte bound.
    fn slot_to_bound(slot_mask: u8, mask: u8) -> u8;
}

/// Least Significant Bit [`BitOrd`].
#[derive(Clone, Copy)]
pub enum Lsb {}

impl<W: BitWidth> BitOrd<W> for Lsb {
    const MASK: u8 = u8::MAX >> (8 - W::WIDTH);

    #[inline]
    fn bound_to_slot(mask: u8, slot_idx: usize) -> u8 {
        mask.rotate_left((slot_idx * W::WIDTH) as u32)
    }

    #[inline]
    fn slot_to_bound(slot_mask: u8, mask: u8) -> u8 {
        mask >> slot_mask.trailing_zeros()
    }
}

/// Most Significant Bit [`BitOrd`].
#[derive(Clone, Copy)]
pub enum Msb {}

impl<W: BitWidth> BitOrd<W> for Msb {
    const MASK: u8 = u8::MAX << (8 - W::WIDTH);

    #[inline]
    fn bound_to_slot(mask: u8, slot_idx: usize) -> u8 {
        mask.rotate_right((slot_idx * W::WIDTH) as u32)
    }

    #[inline]
    fn slot_to_bound(slot_mask: u8, mask: u8) -> u8 {
        mask << slot_mask.leading_zeros()
    }
}
