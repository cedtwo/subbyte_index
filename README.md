# Sub-Byte Index
Bitfield array/vector indexing, mutation and alignment.

## Functionality

`subbyte_index` enables the access and mutation of fixed size bitfields within `[u8]` slices. It
attempts to emulate `std::ops::Index` operations where possible, supporting `usize` and
`Range<usize>` indexes as arguments. The notable features are as follows:
- Support for **byte-aligned** bitwidths (`2` and `4` width bitfields). See [Limitations](#limitations) below.
- Support for arrays of *Lsb* or *Msb* aligned bytes.
- Support for outputting to *Lsb* or *Msb* (indexing operations).
- Support for aligning input to *Lsb* or *Msb* (mutating operations).
- Implemented on primitive byte slices.

## Examples

Sub-byte element access is handled by the [`SubByteIndex`] trait. Operations expect a [`BitWidth`],
and the array and output [`BitOrd`] respectively. Indices support `usize` and `usize` ranges. See
[`SubByteIndex::subbyte_index`].

```rust
# use subbyte_index::SubByteIndex;
# use subbyte_index::{Msb, Lsb, W2, W4};

// A two byte array, represented as two bit subsets in the following operations.
let array_w2 = [0b_00_01_10_11, 0b_11_10_01_00];
// Msb indices:     0  1  2  3      4  5  6  7
// Lsb indices:     3  2  1  0      7  6  5  4

// Msb indexing, output to the Msb.
assert_eq!(array_w2.subbyte_index::<W2, Msb, Msb>(1), 0b_01_000000);
// Lsb indexing, output to the Lsb.
assert_eq!(array_w2.subbyte_index::<W2, Lsb, Lsb>(6), 0b_10);

// A two byte array, represented as four bit subsets in the following operations.
let array_w4 = [0b_1000_1100, 0b_1110_1111];
// Msb indices:       0    1        2    3
// Lsb indices:       1    0        3    2

// Msb indexing, output to the Lsb.
assert_eq!(array_w4.subbyte_index::<W4, Msb, Lsb>(..).collect::<Vec<u8>>(),
     [0b1000, 0b1100, 0b1110, 0b1111]);
// Lsb indexing, output to the Msb.
assert_eq!(array_w4.subbyte_index::<W4, Lsb, Msb>(1..3).collect::<Vec<u8>>(),
     [0b1000_0000, 0b1111_0000]);
```

Sub-byte element mutation is handled by the [`SubByteIndexMut`] trait. Similar to [`SubByteIndex`],
it accepts a `usize` or `usize` range index. Where a `usize` index is passed, a a `u8` byte mask
input is expected. For `usize` ranges, a `u8` byte mask `IntoIterator` implementing type is
expected. Note that the number of elements to input must be greater or equal to the length of the
index range.

```rust
# use subbyte_index::{SubByteIndex, SubByteIndexMut};
# use subbyte_index::{Msb, Lsb, W2, W4};

// An 8 byte vector, represented as four bit subsets in the following operations.
let mut array_w4 = vec![0; 8];

// Set each 4-bit element to its index.
array_w4.as_mut_slice().subbyte_index_mut::<W4, Lsb, Lsb>(.., (0..));
assert_eq!(array_w4.as_slice().subbyte_index::<W4, Lsb, Lsb>(..).collect::<Vec<_>>(), Vec::from_iter(0..16));

// Set element `1`.
array_w4.as_mut_slice().subbyte_index_mut::<W4, Lsb, Lsb>(1, 15);
// Set elements `8..12`.
array_w4.as_mut_slice().subbyte_index_mut::<W4, Lsb, Lsb>(8..12, [15, 1, 2, 15]);
assert_eq!(
    array_w4
        .subbyte_index::<W4, Lsb, Lsb>(..)
        .collect::<Vec<u8>>(),
    [0, 15, 2, 3, 4, 5, 6, 7, 15, 1, 2, 15, 12, 13, 14, 15]
);

// Assert the vector only contains 8 bytes.
assert_eq!(array_w4.len(), 8);
```

Array creation is handled much in the same way as mutation, by specifying an index range and
providing an iterator of elements. As mentioned earlier, the number of elements input must be
greater than or equal to the index range. Therefore when populating an array, providing an exact
index range (rather than [`std::ops::RangeFull`]) is often preferred.

```rust
# use subbyte_index::{SubByteIndex, SubByteIndexMut};
# use subbyte_index::{BitWidth, Lsb, W4};
/// Create an array of 4 bit elements.
type W = W4;
const LEN: usize = 5;
const BYTE_LEN: usize = LEN.div_ceil(W::N_SLOTS) as usize;

// Create an array that can fit 5 elements of 4 bits each.
let mut array = [0; BYTE_LEN];
array.subbyte_index_mut::<W, Lsb, Lsb>(..LEN, [0, 1, 2, 3, 4]);
assert_eq!(array, [0b_0001_0000, 0b_0011_0010, 0b_0000_0100]);
// Lsb indices:          1    0        3    2      OOB    4

// Without providing an upper bound, the above method would expect 6 input elements.
assert_eq!(BYTE_LEN, 3);
assert_eq!(BYTE_LEN * W::N_SLOTS, 6);
```

## Limitations

This crate, by design, offers the maximum flexibility possible for `[u8]` slice operations and
alignment. As such, generic arguments are verbose, and index ranges are relative to byte size (in
contrast to the actual number of elements represented). For many use cases, abstracting away the
generic arguments and persisting the sub-byte upper bound is preferable. Apart from the above, the
following limitations are expected.

### Bit Width

This library only supports widths that are a factor of a byte. This ensures all operations are
*safe* (no element exceeds a byte bound) and performant (partial sequences need not be extracted
from multiple bytes). A bit width of `1` is not supported as other crates
(eg. [bitvec](https://crates.io/crates/bitvec)) already provide a mature and performant implementation.

### The `[u8]` Slice implementation

This crate is only implemented on `[u8]` slices. Support for other integer types is not supported as
casting to a `[u8]` slice is trivial with crates such as [bytemuck](https://crates.io/crates/bytemuck).

### Performance

There are various use-cases where sub-byte arrays can benefit. There are also equally, if not more
cases where the packing/unpacking operation overhead makes sub-byte arrays detrimental for a
use-case. Benchmark to ensure that your use-case benefits from a sub-byte array.

Note that the initial sub-byte indexing implementation is generic for bit-widths and bit-ordering.
The number of operations for all implementations can be reduced by implementing each case
individually. This is not currently prioritized but feel free to open an issue, or even contribute
if you feel performance is not up to expectation.
