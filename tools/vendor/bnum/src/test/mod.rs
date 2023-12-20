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

pub mod cast_types {
    use crate::{BUint, BUintD32, BUintD16, BUintD8, BInt, BIntD32, BIntD16, BIntD8};

    pub use crate::big_types::u8::{UTEST as UTESTD8, ITEST as ITESTD8};
    pub use crate::big_types::u16::{UTEST as UTESTD16, ITEST as ITESTD16};
    pub use crate::big_types::u32::{UTEST as UTESTD32, ITEST as ITESTD32};
    pub use crate::big_types::u64::{UTEST as UTESTD64, ITEST as ITESTD64};

    pub type TestUint1 = BUint<3>;
    pub type TestUint2 = BUintD32<5>;
    pub type TestUint3 = BUintD16<6>;
    pub type TestUint4 = BUintD8<11>;
    pub type TestUint5 = BUintD16<3>;
    pub type TestUint6 = BUintD8<7>;
    pub type TestUint7 = BUintD8<3>;
    pub type TestUint8 = BUintD16<1>;
    pub type TestUint9 = BUintD8<15>;
    pub type TestUint10 = BUintD8<17>;

    pub type TestInt1 = BInt<3>;
    pub type TestInt2 = BIntD32<5>;
    pub type TestInt3 = BIntD16<6>;
    pub type TestInt4 = BIntD8<11>;
    pub type TestInt5 = BIntD16<3>;
    pub type TestInt6 = BIntD8<7>;
    pub type TestInt7 = BIntD8<3>;
    pub type TestInt8 = BIntD16<1>;
    pub type TestInt9 = BIntD8<15>;
    pub type TestInt10 = BIntD8<17>;
}