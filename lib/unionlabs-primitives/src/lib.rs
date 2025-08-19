#![deny(
    clippy::pedantic,
    clippy::std_instead_of_core,
    clippy::std_instead_of_alloc,
    clippy::alloc_instead_of_core
)]

use core::ptr::{addr_of, addr_of_mut};

use crate::encoding::HexPrefixed;

extern crate alloc;

pub mod encoding;

mod fixed_bytes;

mod bytes;

mod bech32;

// TODO: Replace with something like <https://github.com/recmo/uint>
pub mod uint;

mod compat;

pub use crate::{
    bech32::{Bech32, Bech32DecodeError},
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

    /// Slice into an array at `FROM..(FROM + LEN)`, returning a mutable reference to an array of length `LEN`. This will fail to compile if the equivalent slicing would panic at runtime.
    ///
    /// ```compile_fail
    /// # use unionlabs_primitives::ByteArrayExt;
    /// let arr = [1, 2, 3, 4, 5];
    ///
    /// // attempt to read `arr[4..(4 + 2)]`
    /// arr.array_slice_mut::<4, 2>();
    /// ```
    ///
    /// ```rust
    /// # use unionlabs_primitives::ByteArrayExt;
    /// # let mut arr = [1, 2, 3, 4, 5];
    /// // checked at compile time!
    /// let new_arr: &mut [u8; 2] = arr.array_slice_mut::<0, 2>();
    /// *new_arr = [42_u8, 42];
    /// assert_eq!(arr, [42, 42, 3, 4, 5]);
    /// ```
    fn array_slice_mut<const OFFSET: usize, const LEN: usize>(&mut self) -> &mut [u8; LEN];
}

impl<const N: usize> ByteArrayExt<N> for [u8; N] {
    fn array_slice<const OFFSET: usize, const LEN: usize>(&self) -> [u8; LEN] {
        const { assert!(OFFSET + LEN <= N) };

        unsafe { *addr_of!(self[OFFSET..(OFFSET + LEN)]).cast::<[u8; LEN]>() }
    }

    fn array_slice_mut<const OFFSET: usize, const LEN: usize>(&mut self) -> &mut [u8; LEN] {
        const { assert!(OFFSET + LEN <= N) };

        unsafe { &mut *addr_of_mut!(self[OFFSET..(OFFSET + LEN)]).cast::<[u8; LEN]>() }
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

#[test]
fn array_slice_mut() {
    let mut arr = [1, 2, 3, 4, 5];

    assert_eq!(*arr.array_slice_mut::<0, 2>(), [1, 2]);
    assert_eq!(*arr.array_slice_mut::<1, 1>(), [2]);
    assert_eq!(*arr.array_slice_mut::<4, 1>(), [5]);
    assert_eq!(*arr.array_slice_mut::<0, 0>(), [0; 0]);
    assert_eq!(*arr.array_slice_mut::<5, 0>(), [0; 0]);

    arr.array_slice_mut::<0, 2>()[0] = 255;
    assert_eq!(arr, [255, 2, 3, 4, 5]);
    arr.array_slice_mut::<1, 1>()[0] = 255;
    assert_eq!(arr, [255, 255, 3, 4, 5]);
    arr.array_slice_mut::<4, 1>()[0] = 255;
    assert_eq!(arr, [255, 255, 3, 4, 255]);
}
