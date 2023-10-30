// Adapted from https://github.com/Alexhuszagh/rust-lexical.

//! Precalculated large powers for limbs.

#[cfg(limb_width_32)]
pub(crate) use super::large_powers32::*;

#[cfg(limb_width_64)]
pub(crate) use super::large_powers64::*;
