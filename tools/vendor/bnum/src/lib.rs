#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![cfg_attr(
    feature = "nightly",
    feature(
        generic_const_exprs,
        const_trait_impl,
        const_mut_refs,
        const_maybe_uninit_as_mut_ptr,
        const_swap,
        const_option_ext
    )
)]
#![cfg_attr(
    test,
    feature(
        bigint_helper_methods,
        int_roundings,
        float_minimum_maximum,
        wrapping_next_power_of_two,
        float_next_up_down,
    )
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(not(feature = "arbitrary"), no_std)]

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

#[cfg(feature = "usize_exptype")]
type ExpType = usize;
#[cfg(not(feature = "usize_exptype"))]
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

pub(crate) use main_impl;

mod bigints {
    pub use crate::bint::{BInt, BIntD16, BIntD32, BIntD8};
    pub use crate::buint::{BUint, BUintD16, BUintD32, BUintD8};
}

pub use bigints::*;
