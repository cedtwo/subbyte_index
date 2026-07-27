#![doc = include_str!("../README.md")]
mod ord;
mod width;

mod index;
mod index_mut;
mod iter;

pub use index::SubByteIndex;
pub use index_mut::SubByteIndexMut;
pub use iter::SubByteIter;
pub use ord::{BitOrd, Lsb, Msb};
pub use width::{BitWidth, W2, W4};
