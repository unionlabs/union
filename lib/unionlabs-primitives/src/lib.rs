#![deny(
    clippy::pedantic,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]

use core::ptr::addr_of;

use crate::encoding::HexPrefixed;

extern crate alloc;

pub mod encoding;

mod fixed_bytes;

mod bytes;

// TODO: Replace with something like <https://github.com/recmo/uint>
pub mod uint;

mod compat;

pub use crate::{
    bytes::Bytes,
    fixed_bytes::{FixedBytes, FixedBytesError},
    uint::U256,
};

pub type H32<E = HexPrefixed> = FixedBytes<4, E>;
pub type H64<E = HexPrefixed> = FixedBytes<8, E>;
pub type H72<E = HexPrefixed> = FixedBytes<9, E>;
pub type H160<E = HexPrefixed> = FixedBytes<20, E>;
pub type H256<E = HexPrefixed> = FixedBytes<32, E>;
pub type H384<E = HexPrefixed> = FixedBytes<48, E>;
pub type H512<E = HexPrefixed> = FixedBytes<64, E>;
pub type H768<E = HexPrefixed> = FixedBytes<96, E>;
pub type H2048<E = HexPrefixed> = FixedBytes<256, E>;

pub trait ByteArrayExt<const N: usize> {
    /// Slice into an array at `FROM..(FROM + LEN)`, returning an array of length `LEN`. This will fail to compile if the equivalent slicing would panic at runtime.
    ///
    /// ```compile_fail
    /// # use unionlabs_primitives::ByteArrayExt;
    /// let arr = [1, 2, 3, 4, 5];
    ///
    /// // attempt to read `arr[4..(4 + 2)]`
    /// arr.array_slice::<4, 2>();
    /// ```
    ///
    /// ```rust
    /// # use unionlabs_primitives::ByteArrayExt;
    /// # let arr = [1, 2, 3, 4, 5];
    /// // checked at compile time!
    /// assert_eq!(arr.array_slice::<0, 2>(), [1, 2]);
    /// ```
    fn array_slice<const OFFSET: usize, const LEN: usize>(&self) -> [u8; LEN];
}

impl<const N: usize> ByteArrayExt<N> for [u8; N] {
    fn array_slice<const OFFSET: usize, const LEN: usize>(&self) -> [u8; LEN] {
        const { assert!(OFFSET + LEN <= N) };

        unsafe { *addr_of!(self[OFFSET..(OFFSET + LEN)]).cast::<[u8; LEN]>() }
    }
}

#[test]
fn array_slice() {
    let arr = [1, 2, 3, 4, 5];

    assert_eq!(arr.array_slice::<0, 2>(), [1, 2]);
    assert_eq!(arr.array_slice::<1, 1>(), [2]);
    assert_eq!(arr.array_slice::<4, 1>(), [5]);
    assert_eq!(arr.array_slice::<0, 0>(), [0; 0]);
    assert_eq!(arr.array_slice::<5, 0>(), [0; 0]);
}
