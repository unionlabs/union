#![cfg_attr(docs_rs, feature(doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![deny(
    anonymous_parameters,
    clippy::all,
    clippy::missing_safety_doc,
    clippy::missing_safety_doc,
    clippy::undocumented_unsafe_blocks,
    illegal_floating_point_literal_pattern,
    late_bound_lifetime_arguments,
    patterns_in_fns_without_body,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_op_in_unsafe_fn,
    unused_extern_crates
)]
#![warn(
    clippy::dbg_macro,
    clippy::decimal_literal_representation,
    clippy::get_unwrap,
    clippy::nursery,
    clippy::pedantic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unwrap_used,
    clippy::use_debug,
    missing_copy_implementations,
    missing_debug_implementations,
    unused_qualifications,
    variant_size_differences
)]
#![allow(
    path_statements, // used for static assertions
    clippy::inline_always,
    clippy::missing_errors_doc,
    clippy::must_use_candidate,
    clippy::redundant_pub_crate,
)]
#![doc(test(attr(deny(warnings))))]

#[cfg(test)]
mod tests;
mod traits;

#[cfg(feature = "alloc")]
#[allow(unused_extern_crates)]
extern crate alloc;

use core::borrow::Borrow;
use core::cmp::Ordering;
use core::fmt;
use core::num::IntErrorKind;
use core::str::FromStr;
#[cfg(feature = "std")]
use std::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TryFromIntError;

impl fmt::Display for TryFromIntError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("out of range integral type conversion attempted")
    }
}
#[cfg(feature = "std")]
impl Error for TryFromIntError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIntError {
    kind: IntErrorKind,
}

impl ParseIntError {
    /// Outputs the detailed cause of parsing an integer failing.
    // This function is not const because the counterpart of stdlib isn't
    #[allow(clippy::missing_const_for_fn)]
    #[inline(always)]
    pub fn kind(&self) -> &IntErrorKind {
        &self.kind
    }
}

impl fmt::Display for ParseIntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            IntErrorKind::Empty => "cannot parse integer from empty string",
            IntErrorKind::InvalidDigit => "invalid digit found in string",
            IntErrorKind::PosOverflow => "number too large to fit in target type",
            IntErrorKind::NegOverflow => "number too small to fit in target type",
            IntErrorKind::Zero => "number would be zero for non-zero type",
            _ => "Unknown Int error kind",
        }
        .fmt(f)
    }
}

#[cfg(feature = "std")]
impl Error for ParseIntError {}

macro_rules! const_try_opt {
    ($e:expr) => {
        match $e {
            Some(value) => value,
            None => return None,
        }
    };
}

macro_rules! if_signed {
    (true $($x:tt)*) => { $($x)*};
    (false $($x:tt)*) => {};
}

macro_rules! article {
    (true) => {
        "An"
    };
    (false) => {
        "A"
    };
}

macro_rules! unsafe_unwrap_unchecked {
    ($e:expr) => {{
        let opt = $e;
        debug_assert!(opt.is_some());
        match $e {
            Some(value) => value,
            None => core::hint::unreachable_unchecked(),
        }
    }};
}

macro_rules! assume {
    ($e:expr) => {{
        let val = $e;
        debug_assert!(val);
        if !val {
            core::hint::unreachable_unchecked()
        }
    }};
}

macro_rules! impl_ranged {
    ($(
        $type:ident {
            mod_name: $mod_name:ident
            internal: $internal:ident
            signed: $is_signed:ident
            optional: $optional_type:ident
        }
    )*) => {$(
        pub use $mod_name::{$type, $optional_type};

        // Introduce the type in a module. This ensures that all accesses and mutations of the field
        // have the necessary checks.
        mod $mod_name {
            #[doc = concat!(
                article!($is_signed),
                " `",
                stringify!($internal),
                "` that is known to be in the range `MIN..=MAX`.",
            )]
            #[repr(transparent)]
            #[derive(Clone, Copy, Eq, Ord, Hash)]
            pub struct $type<const MIN: $internal, const MAX: $internal>(
                $internal,
            );

            #[doc = concat!(
                "A `",
                stringify!($type),
                "` that is optional. Equivalent to [`Option<",
                stringify!($type),
                ">`] with niche value optimization.",
            )]
            ///
            #[doc = concat!(
                "If `MIN` is [`",
                stringify!($internal),
                "::MIN`] _and_ `MAX` is [`",
                stringify!($internal)
                ,"::MAX`] then compilation will fail. This is because there is no way to represent \
                the niche value.",
            )]
            ///
            /// This type is useful when you need to store an optional ranged value in a struct, but
            /// do not want the overhead of an `Option` type. This reduces the size of the struct
            /// overall, and is particularly useful when you have a large number of optional fields.
            /// Note that most operations must still be performed on the [`Option`] type, which is
            #[doc = concat!("obtained with [`", stringify!($optional_type), "::get`].")]
            #[repr(transparent)]
            #[derive(Clone, Copy, Eq, Hash)]
            pub struct $optional_type<const MIN: $internal, const MAX: $internal>(
                $internal,
            );

            impl<const MIN: $internal, const MAX: $internal> $type<MIN, MAX> {
                /// Creates a ranged integer without checking the value.
                ///
                /// # Safety
                ///
                /// The value must be within the range `MIN..=MAX`.
                #[inline(always)]
                pub const unsafe fn new_unchecked(value: $internal) -> Self {
                    <Self as $crate::traits::RangeIsValid>::ASSERT;
                    // Safety: The caller must ensure that the value is in range.
                    unsafe { assume!(MIN <= value && value <= MAX) };
                    Self(value)
                }

                /// Creates a ranged integer if the given value is in the range
                /// `MIN..=MAX`.
                #[inline(always)]
                pub const fn new(value: $internal) -> Option<Self> {
                    <Self as $crate::traits::RangeIsValid>::ASSERT;
                    if value < MIN || value > MAX {
                        None
                    } else {
                        Some(Self(value))
                    }
                }

                /// Returns the value as a primitive type.
                #[inline(always)]
                pub const fn get(self) -> $internal {
                    <Self as $crate::traits::RangeIsValid>::ASSERT;
                    // Safety: A stored value is always in range.
                    unsafe { assume!(MIN <= self.0 && self.0 <= MAX) };
                    self.0
                }

                #[inline(always)]
                pub(crate) const fn get_ref(&self) -> &$internal {
                    <Self as $crate::traits::RangeIsValid>::ASSERT;
                    // Safety: A stored value is always in range.
                    unsafe { assume!(MIN <= self.0 && self.0 <= MAX) };
                    &self.0
                }
            }

            impl<const MIN: $internal, const MAX: $internal> $optional_type<MIN, MAX> {
                /// The value used as the niche. Must not be in the range `MIN..=MAX`.
                const NICHE: $internal = match (MIN, MAX) {
                    ($internal::MIN, $internal::MAX) => panic!("type has no niche"),
                    ($internal::MIN, _) => $internal::MAX,
                    (_, _) => $internal::MIN,
                };

                /// An optional ranged value that is not present.
                #[allow(non_upper_case_globals)]
                pub const None: Self = Self(Self::NICHE);

                /// Creates an optional ranged value that is present.
                #[allow(non_snake_case)]
                #[inline(always)]
                pub const fn Some(value: $type<MIN, MAX>) -> Self {
                    <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                    Self(value.get())
                }

                /// Returns the value as the standard library's [`Option`] type.
                #[inline(always)]
                pub const fn get(self) -> Option<$type<MIN, MAX>> {
                    <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                    if self.0 == Self::NICHE {
                        None
                    } else {
                        // Safety: A stored value that is not the niche is always in range.
                        Some(unsafe { $type::new_unchecked(self.0) })
                    }
                }

                /// Creates an optional ranged integer without checking the value.
                ///
                /// # Safety
                ///
                /// The value must be within the range `MIN..=MAX`. As the value used for niche
                /// value optimization is unspecified, the provided value must not be the niche
                /// value.
                #[inline(always)]
                pub const unsafe fn some_unchecked(value: $internal) -> Self {
                    <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                    // Safety: The caller must ensure that the value is in range.
                    unsafe { assume!(MIN <= value && value <= MAX) };
                    Self(value)
                }

                /// Obtain the inner value of the struct. This is useful for comparisons.
                #[inline(always)]
                pub(crate) const fn inner(self) -> $internal {
                    <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                    self.0
                }
            }
        }

        impl<const MIN: $internal, const MAX: $internal> $type<MIN, MAX> {
            /// The smallest value that can be represented by this type.
            // Safety: `MIN` is in range by definition.
            pub const MIN: Self = unsafe { Self::new_unchecked(MIN) };

            /// The largest value that can be represented by this type.
            // Safety: `MAX` is in range by definition.
            pub const MAX: Self = unsafe { Self::new_unchecked(MAX) };

            /// Expand the range that the value may be in. **Fails to compile** if the new range is
            /// not a superset of the current range.
            pub const fn expand<const NEW_MIN: $internal, const NEW_MAX: $internal>(
                self,
            ) -> $type<NEW_MIN, NEW_MAX> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                <$type<NEW_MIN, NEW_MAX> as $crate::traits::RangeIsValid>::ASSERT;
                <($type<MIN, MAX>, $type<NEW_MIN, NEW_MAX>) as $crate::traits::ExpandIsValid>
                    ::ASSERT;
                // Safety: The range is widened.
                unsafe { $type::new_unchecked(self.get()) }
            }

            /// Attempt to narrow the range that the value may be in. Returns `None` if the value
            /// is outside the new range. **Fails to compile** if the new range is not a subset of
            /// the current range.
            pub const fn narrow<
                const NEW_MIN: $internal,
                const NEW_MAX: $internal,
            >(self) -> Option<$type<NEW_MIN, NEW_MAX>> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                <$type<NEW_MIN, NEW_MAX> as $crate::traits::RangeIsValid>::ASSERT;
                <($type<MIN, MAX>, $type<NEW_MIN, NEW_MAX>) as $crate::traits::NarrowIsValid>
                    ::ASSERT;
                $type::<NEW_MIN, NEW_MAX>::new(self.get())
            }


            /// Creates a ranged integer with a statically known value. **Fails to compile** if the
            /// value is not in range.
            #[inline(always)]
            pub const fn new_static<const VALUE: $internal>() -> Self {
                <($type<MIN, VALUE>, $type<VALUE, MAX>) as $crate::traits::StaticIsValid>::ASSERT;
                // Safety: The value is in range.
                unsafe { Self::new_unchecked(VALUE) }
            }

            #[inline]
            const fn new_saturating(value: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The value is clamped to the range.
                unsafe {
                    Self::new_unchecked(if value < MIN {
                        MIN
                    } else if value > MAX {
                        MAX
                    } else {
                        value
                    })
                }
            }


            /// Converts a string slice in a given base to an integer.
            ///
            /// The string is expected to be an optional `+` or `-` sign followed by digits. Leading
            /// and trailing whitespace represent an error. Digits are a subset of these characters,
            /// depending on `radix`:
            ///
            /// - `0-9`
            /// - `a-z`
            /// - `A-Z`
            ///
            /// # Panics
            ///
            /// Panics if `radix` is not in the range `2..=36`.
            ///
            /// # Examples
            ///
            /// Basic usage:
            ///
            /// ```rust
            #[doc = concat!("# use deranged::", stringify!($type), ";")]
            #[doc = concat!(
                "assert_eq!(",
                stringify!($type),
                "::<5, 10>::from_str_radix(\"A\", 16), Ok(",
                stringify!($type),
                "::new_static::<10>()));",
            )]
            /// ```
            #[inline]
            pub fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                match $internal::from_str_radix(src, radix) {
                    Ok(value) if value > MAX => {
                        Err(ParseIntError { kind: IntErrorKind::PosOverflow })
                    }
                    Ok(value) if value < MIN => {
                        Err(ParseIntError { kind: IntErrorKind::NegOverflow })
                    }
                    // Safety: If the value was out of range, it would have been caught in a
                    // previous arm.
                    Ok(value) => Ok(unsafe { Self::new_unchecked(value) }),
                    Err(e) => Err(ParseIntError { kind: e.kind().clone() }),
                }
            }

            /// Checked integer addition. Computes `self + rhs`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_add(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_add(rhs)))
            }

            /// Unchecked integer addition. Computes `self + rhs`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self + rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_add(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_add(rhs)))
                }
            }

            /// Checked integer addition. Computes `self - rhs`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_sub(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_sub(rhs)))
            }

            /// Unchecked integer subtraction. Computes `self - rhs`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self - rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_sub(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_sub(rhs)))
                }
            }

            /// Checked integer addition. Computes `self * rhs`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_mul(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_mul(rhs)))
            }

            /// Unchecked integer multiplication. Computes `self * rhs`, assuming that the result is
            /// in range.
            ///
            /// # Safety
            ///
            /// The result of `self * rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_mul(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_mul(rhs)))
                }
            }

            /// Checked integer addition. Computes `self / rhs`, returning `None` if `rhs == 0` or
            /// if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_div(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_div(rhs)))
            }

            /// Unchecked integer division. Computes `self / rhs`, assuming that `rhs != 0` and that
            /// the result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self / rhs` must be in the range
            /// `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_div(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_div(rhs)))
                }
            }

            /// Checked Euclidean division. Computes `self.div_euclid(rhs)`, returning `None` if
            /// `rhs == 0` or if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_div_euclid(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_div_euclid(rhs)))
            }

            /// Unchecked Euclidean division. Computes `self.div_euclid(rhs)`, assuming that
            /// `rhs != 0` and that the result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self.div_euclid(rhs)` must be in the
            /// range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_div_euclid(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(
                        unsafe_unwrap_unchecked!(self.get().checked_div_euclid(rhs))
                    )
                }
            }

            /// Checked integer remainder. Computes `self % rhs`, returning `None` if `rhs == 0` or
            /// if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_rem(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_rem(rhs)))
            }

            /// Unchecked remainder. Computes `self % rhs`, assuming that `rhs != 0` and that the
            /// result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self % rhs` must be in the range
            /// `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_rem(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_rem(rhs)))
                }
            }

            /// Checked Euclidean remainder. Computes `self.rem_euclid(rhs)`, returning `None` if
            /// `rhs == 0` or if the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_rem_euclid(self, rhs: $internal) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_rem_euclid(rhs)))
            }

            /// Unchecked Euclidean remainder. Computes `self.rem_euclid(rhs)`, assuming that
            /// `rhs != 0` and that the result is in range.
            ///
            /// # Safety
            ///
            /// `self` must not be zero and the result of `self.rem_euclid(rhs)` must be in the
            /// range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_rem_euclid(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range and that `rhs` is not
                // zero.
                unsafe {
                    Self::new_unchecked(
                        unsafe_unwrap_unchecked!(self.get().checked_rem_euclid(rhs))
                    )
                }
            }

            /// Checked negation. Computes `-self`, returning `None` if the resulting value is out
            /// of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_neg(self) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_neg()))
            }

            /// Unchecked negation. Computes `-self`, assuming that `-self` is in range.
            ///
            /// # Safety
            ///
            /// The result of `-self` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_neg(self) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe { Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_neg())) }
            }

            /// Absolute value. Computes `self.neg()`, **failing to compile** if the result is not
            /// guaranteed to be in range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const fn neg(self) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                <Self as $crate::traits::NegIsSafe>::ASSERT;
                // Safety: The compiler asserts that the result is in range.
                unsafe { self.unchecked_neg() }
            }

            /// Checked shift left. Computes `self << rhs`, returning `None` if the resulting value
            /// is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_shl(self, rhs: u32) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_shl(rhs)))
            }

            /// Unchecked shift left. Computes `self << rhs`, assuming that the result is in range.
            ///
            /// # Safety
            ///
            /// The result of `self << rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_shl(self, rhs: u32) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_shl(rhs)))
                }
            }

            /// Checked shift right. Computes `self >> rhs`, returning `None` if
            /// the resulting value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_shr(self, rhs: u32) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_shr(rhs)))
            }

            /// Unchecked shift right. Computes `self >> rhs`, assuming that the result is in range.
            ///
            /// # Safety
            ///
            /// The result of `self >> rhs` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_shr(self, rhs: u32) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_shr(rhs)))
                }
            }

            if_signed!($is_signed
            /// Checked absolute value. Computes `self.abs()`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_abs(self) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_abs()))
            }

            /// Unchecked absolute value. Computes `self.abs()`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self.abs()` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_abs(self) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe { Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_abs())) }
            }

            /// Absolute value. Computes `self.abs()`, **failing to compile** if the result is not
            /// guaranteed to be in range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const fn abs(self) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                <Self as $crate::traits::AbsIsSafe>::ASSERT;
                // Safety: The compiler asserts that the result is in range.
                unsafe { self.unchecked_abs() }
            });

            /// Checked exponentiation. Computes `self.pow(exp)`, returning `None` if the resulting
            /// value is out of range.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn checked_pow(self, exp: u32) -> Option<Self> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(const_try_opt!(self.get().checked_pow(exp)))
            }

            /// Unchecked exponentiation. Computes `self.pow(exp)`, assuming that the result is in
            /// range.
            ///
            /// # Safety
            ///
            /// The result of `self.pow(exp)` must be in the range `MIN..=MAX`.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline(always)]
            pub const unsafe fn unchecked_pow(self, exp: u32) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The caller must ensure that the result is in range.
                unsafe {
                    Self::new_unchecked(unsafe_unwrap_unchecked!(self.get().checked_pow(exp)))
                }
            }

            /// Saturating integer addition. Computes `self + rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_add(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new_saturating(self.get().saturating_add(rhs))
            }

            /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_sub(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new_saturating(self.get().saturating_sub(rhs))
            }

            if_signed!($is_signed
            /// Saturating integer negation. Computes `self - rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_neg(self) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new_saturating(self.get().saturating_neg())
            });

            if_signed!($is_signed
            /// Saturating absolute value. Computes `self.abs()`, saturating at the numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_abs(self) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new_saturating(self.get().saturating_abs())
            });

            /// Saturating integer multiplication. Computes `self * rhs`, saturating at the numeric
            /// bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_mul(self, rhs: $internal) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new_saturating(self.get().saturating_mul(rhs))
            }

            /// Saturating integer exponentiation. Computes `self.pow(exp)`, saturating at the
            /// numeric bounds.
            #[must_use = "this returns the result of the operation, without modifying the original"]
            #[inline]
            pub const fn saturating_pow(self, exp: u32) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new_saturating(self.get().saturating_pow(exp))
            }
        }

        impl<const MIN: $internal, const MAX: $internal> $optional_type<MIN, MAX> {
            #[inline(always)]
            pub const fn get_primitive(self) -> Option<$internal> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                Some(const_try_opt!(self.get()).get())
            }

            /// Returns `true` if the value is the niche value.
            #[inline(always)]
            pub const fn is_none(self) -> bool {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                self.get().is_none()
            }

            /// Returns `true` if the value is not the niche value.
            #[inline(always)]
            pub const fn is_some(self) -> bool {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                self.get().is_some()
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Debug for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Debug for $optional_type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Display for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> Default for $optional_type<MIN, MAX> {
            #[inline(always)]
            fn default() -> Self {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                Self::None
            }
        }

        impl<const MIN: $internal, const MAX: $internal> AsRef<$internal> for $type<MIN, MAX> {
            #[inline(always)]
            fn as_ref(&self) -> &$internal {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                &self.get_ref()
            }
        }

        impl<const MIN: $internal, const MAX: $internal> Borrow<$internal> for $type<MIN, MAX> {
            #[inline(always)]
            fn borrow(&self) -> &$internal {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                &self.get_ref()
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialEq<$type<MIN_B, MAX_B>> for $type<MIN_A, MAX_A> {
            #[inline(always)]
            fn eq(&self, other: &$type<MIN_B, MAX_B>) -> bool {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                <$type<MIN_B, MAX_B> as $crate::traits::RangeIsValid>::ASSERT;
                self.get() == other.get()
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialEq<$optional_type<MIN_B, MAX_B>> for $optional_type<MIN_A, MAX_A> {
            #[inline(always)]
            fn eq(&self, other: &$optional_type<MIN_B, MAX_B>) -> bool {
                <$type<MIN_A, MAX_A> as $crate::traits::RangeIsValid>::ASSERT;
                <$type<MIN_B, MAX_B> as $crate::traits::RangeIsValid>::ASSERT;
                self.inner() == other.inner()
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialOrd<$type<MIN_B, MAX_B>> for $type<MIN_A, MAX_A> {
            #[inline(always)]
            fn partial_cmp(&self, other: &$type<MIN_B, MAX_B>) -> Option<Ordering> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                <$type<MIN_B, MAX_B> as $crate::traits::RangeIsValid>::ASSERT;
                self.get().partial_cmp(&other.get())
            }
        }

        impl<
            const MIN_A: $internal,
            const MAX_A: $internal,
            const MIN_B: $internal,
            const MAX_B: $internal,
        > PartialOrd<$optional_type<MIN_B, MAX_B>> for $optional_type<MIN_A, MAX_A> {
            #[inline]
            fn partial_cmp(&self, other: &$optional_type<MIN_B, MAX_B>) -> Option<Ordering> {
                <$type<MIN_A, MAX_A> as $crate::traits::RangeIsValid>::ASSERT;
                <$type<MIN_B, MAX_B> as $crate::traits::RangeIsValid>::ASSERT;
                if self.is_none() && other.is_none() {
                    Some(Ordering::Equal)
                } else if self.is_none() {
                    Some(Ordering::Less)
                } else if other.is_none() {
                    Some(Ordering::Greater)
                } else {
                    self.inner().partial_cmp(&other.inner())
                }
            }
        }

        impl<
            const MIN: $internal,
            const MAX: $internal,
        > Ord for $optional_type<MIN, MAX> {
            #[inline]
            fn cmp(&self, other: &Self) -> Ordering {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                if self.is_none() && other.is_none() {
                    Ordering::Equal
                } else if self.is_none() {
                    Ordering::Less
                } else if other.is_none() {
                    Ordering::Greater
                } else {
                    self.inner().cmp(&other.inner())
                }
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Binary for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::LowerHex for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::UpperHex for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::LowerExp for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::UpperExp for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> fmt::Octal for $type<MIN, MAX> {
            #[inline(always)]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().fmt(f)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> From<$type<MIN, MAX>> for $internal {
            #[inline(always)]
            fn from(value: $type<MIN, MAX>) -> Self {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                value.get()
            }
        }

        impl<
            const MIN: $internal,
            const MAX: $internal,
        > From<$type<MIN, MAX>> for $optional_type<MIN, MAX> {
            #[inline(always)]
            fn from(value: $type<MIN, MAX>) -> Self {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                Self::Some(value)
            }
        }

        impl<
            const MIN: $internal,
            const MAX: $internal,
        > From<Option<$type<MIN, MAX>>> for $optional_type<MIN, MAX> {
            #[inline(always)]
            fn from(value: Option<$type<MIN, MAX>>) -> Self {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                match value {
                    Some(value) => Self::Some(value),
                    None => Self::None,
                }
            }
        }

        impl<
            const MIN: $internal,
            const MAX: $internal,
        > From<$optional_type<MIN, MAX>> for Option<$type<MIN, MAX>> {
            #[inline(always)]
            fn from(value: $optional_type<MIN, MAX>) -> Self {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                value.get()
            }
        }

        impl<const MIN: $internal, const MAX: $internal> TryFrom<$internal> for $type<MIN, MAX> {
            type Error = TryFromIntError;

            #[inline]
            fn try_from(value: $internal) -> Result<Self, Self::Error> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::new(value).ok_or(TryFromIntError)
            }
        }

        impl<const MIN: $internal, const MAX: $internal> FromStr for $type<MIN, MAX> {
            type Err = ParseIntError;

            #[inline]
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                let value = s.parse::<$internal>().map_err(|e| ParseIntError {
                    kind: e.kind().clone()
                })?;
                if value < MIN {
                    Err(ParseIntError { kind: IntErrorKind::NegOverflow })
                } else if value > MAX {
                    Err(ParseIntError { kind: IntErrorKind::PosOverflow })
                } else {
                    // Safety: The value was previously checked for validity.
                    Ok(unsafe { Self::new_unchecked(value) })
                }
            }
        }

        #[cfg(feature = "serde")]
        impl<const MIN: $internal, const MAX: $internal> serde::Serialize for $type<MIN, MAX> {
            #[inline(always)]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                self.get().serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<
            const MIN: $internal,
            const MAX: $internal,
        > serde::Serialize for $optional_type<MIN, MAX> {
            #[inline(always)]
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                self.get().serialize(serializer)
            }
        }

        #[cfg(feature = "serde")]
        impl<
            'de,
            const MIN: $internal,
            const MAX: $internal,
        > serde::Deserialize<'de> for $type<MIN, MAX> {
            #[inline]
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                let internal = <$internal>::deserialize(deserializer)?;
                Self::new(internal).ok_or_else(|| <D::Error as serde::de::Error>::invalid_value(
                    serde::de::Unexpected::Other("integer"),
                    #[cfg(feature = "std")] {
                        &format!("an integer in the range {}..={}", MIN, MAX).as_ref()
                    },
                    #[cfg(not(feature = "std"))] {
                        &"an integer in the valid range"
                    }
                ))
            }
        }

        #[cfg(feature = "serde")]
        impl<
            'de,
            const MIN: $internal,
            const MAX: $internal,
        > serde::Deserialize<'de> for $optional_type<MIN, MAX> {
            #[inline]
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                Ok(Self::Some($type::<MIN, MAX>::deserialize(deserializer)?))
            }
        }

        #[cfg(feature = "rand")]
        impl<
            const MIN: $internal,
            const MAX: $internal,
        > rand::distributions::Distribution<$type<MIN, MAX>> for rand::distributions::Standard {
            #[inline]
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $type<MIN, MAX> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                $type::new(rng.gen_range(MIN..=MAX)).expect("rand failed to generate a valid value")
            }
        }

        #[cfg(feature = "rand")]
        impl<
            const MIN: $internal,
            const MAX: $internal,
        > rand::distributions::Distribution<$optional_type<MIN, MAX>>
        for rand::distributions::Standard {
            #[inline]
            fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> $optional_type<MIN, MAX> {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                rng.gen::<Option<$type<MIN, MAX>>>().into()
            }
        }

        #[cfg(feature = "num")]
        impl<const MIN: $internal, const MAX: $internal> num_traits::Bounded for $type<MIN, MAX> {
            #[inline(always)]
            fn min_value() -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::MIN
            }

            #[inline(always)]
            fn max_value() -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                Self::MAX
            }
        }

        #[cfg(feature = "quickcheck")]
        impl<const MIN: $internal, const MAX: $internal> quickcheck::Arbitrary for $type<MIN, MAX> {
            #[inline]
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                <Self as $crate::traits::RangeIsValid>::ASSERT;
                // Safety: The `rem_euclid` call and addition ensure that the value is in range.
                unsafe {
                    Self::new_unchecked($internal::arbitrary(g).rem_euclid(MAX - MIN + 1) + MIN)
                }
            }

            #[inline]
            fn shrink(&self) -> ::alloc::boxed::Box<dyn Iterator<Item = Self>> {
                ::alloc::boxed::Box::new(
                    self.get()
                        .shrink()
                        .filter_map(Self::new)
                )
            }
        }

        #[cfg(feature = "quickcheck")]
        impl<
            const MIN: $internal,
            const MAX: $internal,
        > quickcheck::Arbitrary for $optional_type<MIN, MAX> {
            #[inline]
            fn arbitrary(g: &mut quickcheck::Gen) -> Self {
                <$type<MIN, MAX> as $crate::traits::RangeIsValid>::ASSERT;
                Option::<$type<MIN, MAX>>::arbitrary(g).into()
            }

            #[inline]
            fn shrink(&self) -> ::alloc::boxed::Box<dyn Iterator<Item = Self>> {
                ::alloc::boxed::Box::new(self.get().shrink().map(Self::from))
            }
        }
    )*};
}

impl_ranged! {
    RangedU8 {
        mod_name: ranged_u8
        internal: u8
        signed: false
        optional: OptionRangedU8
    }
    RangedU16 {
        mod_name: ranged_u16
        internal: u16
        signed: false
        optional: OptionRangedU16
    }
    RangedU32 {
        mod_name: ranged_u32
        internal: u32
        signed: false
        optional: OptionRangedU32
    }
    RangedU64 {
        mod_name: ranged_u64
        internal: u64
        signed: false
        optional: OptionRangedU64
    }
    RangedU128 {
        mod_name: ranged_u128
        internal: u128
        signed: false
        optional: OptionRangedU128
    }
    RangedUsize {
        mod_name: ranged_usize
        internal: usize
        signed: false
        optional: OptionRangedUsize
    }
    RangedI8 {
        mod_name: ranged_i8
        internal: i8
        signed: true
        optional: OptionRangedI8
    }
    RangedI16 {
        mod_name: ranged_i16
        internal: i16
        signed: true
        optional: OptionRangedI16
    }
    RangedI32 {
        mod_name: ranged_i32
        internal: i32
        signed: true
        optional: OptionRangedI32
    }
    RangedI64 {
        mod_name: ranged_i64
        internal: i64
        signed: true
        optional: OptionRangedI64
    }
    RangedI128 {
        mod_name: ranged_i128
        internal: i128
        signed: true
        optional: OptionRangedI128
    }
    RangedIsize {
        mod_name: ranged_isize
        internal: isize
        signed: true
        optional: OptionRangedIsize
    }
}
