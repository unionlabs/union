#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![cfg_attr(
    feature = "nightly",
    feature(
        generic_const_exprs,
        const_trait_impl,
        const_option_ext
    )
)]
#![cfg_attr(
    test,
    feature(
        bigint_helper_methods,
        int_roundings,
        //float_minimum_maximum,
        wrapping_next_power_of_two,
        //float_next_up_down,
    )
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "arbitrary"), no_std)]
// TODO: MAKE SURE NO_STD IS ENABLED WHEN PUBLISHING NEW VERSION

#[macro_use]
extern crate alloc;

mod bint;
mod buint;

pub mod cast;
mod digit;
mod doc;
pub mod errors;
pub mod helpers;
mod int;
mod nightly;
pub mod prelude;

#[cfg(feature = "rand")]
pub mod random;

pub mod types;

/*#[cfg(feature = "nightly")]
mod float;

#[cfg(feature = "nightly")]
pub use float::Float;*/

#[cfg(test)]
mod test;

#[cfg(test)]
use test::types::*;

/*#[cfg(feature = "usize_exptype")]
type ExpType = usize;
#[cfg(not(feature = "usize_exptype"))]*/
type ExpType = u32;

macro_rules! macro_impl {
    ($name: ident) => {
        use crate::bigints::*;

        crate::main_impl!($name);
    };
}

pub(crate) use macro_impl;

macro_rules! main_impl {
    ($name: ident) => {
        $name!(BUint, BInt, u64);
        $name!(BUintD32, BIntD32, u32);
        $name!(BUintD16, BIntD16, u16);
        $name!(BUintD8, BIntD8, u8);
    };
}

use crate::buint::cast::buint_as_different_digit_bigint;
use crate::bint::cast::bint_as_different_digit_bigint;

buint_as_different_digit_bigint!(BUint, BInt, u64; (BUintD32, u32), (BUintD16, u16), (BUintD8, u8));
buint_as_different_digit_bigint!(BUintD32, BIntD32, u32; (BUint, u64), (BUintD16, u16), (BUintD8, u8));
buint_as_different_digit_bigint!(BUintD16, BIntD16, u16; (BUint, u64), (BUintD32, u32), (BUintD8, u8));
buint_as_different_digit_bigint!(BUintD8, BIntD8, u8; (BUint, u64), (BUintD32, u32), (BUintD16, u16));

bint_as_different_digit_bigint!(BUint, BInt, u64; (BIntD32, u32), (BIntD16, u16), (BIntD8, u8));
bint_as_different_digit_bigint!(BUintD32, BIntD32, u32; (BInt, u64), (BIntD16, u16), (BIntD8, u8));
bint_as_different_digit_bigint!(BUintD16, BIntD16, u16; (BInt, u64), (BIntD32, u32), (BIntD8, u8));
bint_as_different_digit_bigint!(BUintD8, BIntD8, u8; (BInt, u64), (BIntD32, u32), (BIntD16, u16));

pub(crate) use main_impl;

mod bigints {
    pub use crate::bint::{BInt, BIntD16, BIntD32, BIntD8};
    pub use crate::buint::{BUint, BUintD16, BUintD32, BUintD8};
}

pub use bigints::*;

#[cfg(feature = "numtraits")]
#[cfg(test)]
quickcheck::quickcheck! {
    fn test_f32_parse(f: f32) -> quickcheck::TestResult {
        if !f.is_finite() {
            return quickcheck::TestResult::discard();
        }
        let s = f.to_string();
        quickcheck::TestResult::from_bool(<f32 as num_traits::Num>::from_str_radix(&s, 10).unwrap() == <f32 as core::str::FromStr>::from_str(&s).unwrap())
    }
}