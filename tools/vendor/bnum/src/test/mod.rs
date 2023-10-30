pub mod convert;
pub use convert::TestConvert;

mod macros;

#[allow(unused_imports)]
pub use macros::*;

pub mod types;

#[derive(Clone, Copy)]
pub struct U8ArrayWrapper<const N: usize>(pub [u8; N]);

impl<const N: usize> From<U8ArrayWrapper<N>> for [u8; N] {
    fn from(a: U8ArrayWrapper<N>) -> Self {
        a.0
    }
}

use quickcheck::{Arbitrary, Gen};

impl<const N: usize> Arbitrary for U8ArrayWrapper<N> {
    fn arbitrary(g: &mut Gen) -> Self {
        let mut arr = [0u8; N];
        for x in arr.iter_mut() {
            *x = u8::arbitrary(g);
        }
        Self(arr)
    }
}

use core::fmt::{self, Debug, Formatter};

impl<const N: usize> Debug for U8ArrayWrapper<N> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}
