use std::fmt::Debug;

/// Indicates how many bits an array element occupies. Affects indexing as well as filtering out all
/// other bits. See [`W2`] and [`W4`].
pub trait BitWidth: Debug + Clone + Copy {
    /// The number of bits in the bitpack.
    const WIDTH: usize;
    /// A bitmask [`Self::WIDTH`] bits set at the LSB.
    const LSB_MASK: u8 = u8::MAX >> (8 - Self::WIDTH);
    /// A bitmask [`Self::WIDTH`] bits set at the MSB.
    const MSB_MASK: u8 = u8::MAX << (8 - Self::WIDTH);
    /// The number of slots in a byte.
    const N_SLOTS: usize = 8 / Self::WIDTH;
}

/// Indicates a bit-width of `2`.
#[derive(Debug, Clone, Copy)]
pub enum W2 {}

impl BitWidth for W2 {
    const WIDTH: usize = 2;
}

/// Indicates a bit-width of `4`.
#[derive(Debug, Clone, Copy)]
pub enum W4 {}

impl BitWidth for W4 {
    const WIDTH: usize = 4;
}
