//! Generate the user-facing flags type.
//!
//! The code here belongs to the end-user, so new trait implementations and methods can't be
//! added without potentially breaking users.

/// Declare the user-facing bitflags struct.
///
/// This type is guaranteed to be a newtype with a `bitflags`-facing type as its single field.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __declare_public_bitflags {
    (
        $(#[$outer:meta])*
        $vis:vis struct $PublicBitFlags:ident
    ) => {
        $(#[$outer])*
        $vis struct $PublicBitFlags(<$PublicBitFlags as $crate::__private::PublicFlags>::Internal);
    };
}

/// Implement functions on the public (user-facing) bitflags type.
///
/// We need to be careful about adding new methods and trait implementations here because they
/// could conflict with items added by the end-user.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_public_bitflags_forward {
    (
        $PublicBitFlags:ident: $T:ty, $InternalBitFlags:ident
    ) => {
        __impl_bitflags! {
            $PublicBitFlags: $T {
                fn empty() {
                    Self($InternalBitFlags::empty())
                }

                fn all() {
                    Self($InternalBitFlags::all())
                }

                fn bits(f) {
                    f.0.bits()
                }

                fn from_bits(bits) {
                    match $InternalBitFlags::from_bits(bits) {
                        $crate::__private::core::option::Option::Some(bits) => $crate::__private::core::option::Option::Some(Self(bits)),
                        $crate::__private::core::option::Option::None => $crate::__private::core::option::Option::None,
                    }
                }

                fn from_bits_truncate(bits) {
                    Self($InternalBitFlags::from_bits_truncate(bits))
                }

                fn from_bits_retain(bits) {
                    Self($InternalBitFlags::from_bits_retain(bits))
                }

                fn from_name(name) {
                    match $InternalBitFlags::from_name(name) {
                        $crate::__private::core::option::Option::Some(bits) => $crate::__private::core::option::Option::Some(Self(bits)),
                        $crate::__private::core::option::Option::None => $crate::__private::core::option::Option::None,
                    }
                }

                fn is_empty(f) {
                    f.0.is_empty()
                }

                fn is_all(f) {
                    f.0.is_all()
                }

                fn intersects(f, other) {
                    f.0.intersects(other.0)
                }

                fn contains(f, other) {
                    f.0.contains(other.0)
                }

                fn insert(f, other) {
                    f.0.insert(other.0)
                }

                fn remove(f, other) {
                    f.0.remove(other.0)
                }

                fn toggle(f, other) {
                    f.0.toggle(other.0)
                }

                fn set(f, other, value) {
                    f.0.set(other.0, value)
                }

                fn intersection(f, other) {
                    Self(f.0.intersection(other.0))
                }

                fn union(f, other) {
                    Self(f.0.union(other.0))
                }

                fn difference(f, other) {
                    Self(f.0.difference(other.0))
                }

                fn symmetric_difference(f, other) {
                    Self(f.0.symmetric_difference(other.0))
                }

                fn complement(f) {
                    Self(f.0.complement())
                }
            }
        }
    };
}

/// Implement functions on the public (user-facing) bitflags type.
///
/// We need to be careful about adding new methods and trait implementations here because they
/// could conflict with items added by the end-user.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_public_bitflags {
    (
        $BitFlags:ident: $T:ty, $PublicBitFlags:ident {
            $(
                $(#[$attr:ident $($args:tt)*])*
                $Flag:ident;
            )*
        }
    ) => {
        __impl_bitflags! {
            $BitFlags: $T {
                fn empty() {
                    Self(<$T as $crate::Bits>::EMPTY)
                }

                fn all() {
                    Self::from_bits_truncate(<$T as $crate::Bits>::ALL)
                }

                fn bits(f) {
                    f.0
                }

                fn from_bits(bits) {
                    let truncated = Self::from_bits_truncate(bits).0;

                    if truncated == bits {
                        $crate::__private::core::option::Option::Some(Self(bits))
                    } else {
                        $crate::__private::core::option::Option::None
                    }
                }

                fn from_bits_truncate(bits) {
                    if bits == <$T as $crate::Bits>::EMPTY {
                        return Self(bits)
                    }

                    let mut truncated = <$T as $crate::Bits>::EMPTY;

                    $(
                        __bitflags_expr_safe_attrs!(
                            $(#[$attr $($args)*])*
                            {
                                if bits & $PublicBitFlags::$Flag.bits() == $PublicBitFlags::$Flag.bits() {
                                    truncated = truncated | $PublicBitFlags::$Flag.bits()
                                }
                            }
                        );
                    )*

                    Self(truncated)
                }

                fn from_bits_retain(bits) {
                    Self(bits)
                }

                fn from_name(name) {
                    $(
                        __bitflags_expr_safe_attrs!(
                            $(#[$attr $($args)*])*
                            {
                                if name == $crate::__private::core::stringify!($Flag) {
                                    return $crate::__private::core::option::Option::Some(Self($PublicBitFlags::$Flag.bits()));
                                }
                            }
                        );
                    )*

                    let _ = name;
                    $crate::__private::core::option::Option::None
                }

                fn is_empty(f) {
                    f.bits() == <$T as $crate::Bits>::EMPTY
                }

                fn is_all(f) {
                    // NOTE: We check against `Self::all` here, not `Self::Bits::ALL`
                    // because the set of all flags may not use all bits
                    Self::all().bits() | f.bits() == f.bits()
                }

                fn intersects(f, other) {
                    f.bits() & other.bits() != <$T as $crate::Bits>::EMPTY
                }

                fn contains(f, other) {
                    f.bits() & other.bits() == other.bits()
                }

                fn insert(f, other) {
                    *f = Self::from_bits_retain(f.bits() | other.bits());
                }

                fn remove(f, other) {
                    *f = Self::from_bits_retain(f.bits() & !other.bits());
                }

                fn toggle(f, other) {
                    *f = Self::from_bits_retain(f.bits() ^ other.bits());
                }

                fn set(f, other, value) {
                    if value {
                        f.insert(other);
                    } else {
                        f.remove(other);
                    }
                }

                fn intersection(f, other) {
                    Self::from_bits_retain(f.bits() & other.bits())
                }

                fn union(f, other) {
                    Self::from_bits_retain(f.bits() | other.bits())
                }

                fn difference(f, other) {
                    Self::from_bits_retain(f.bits() & !other.bits())
                }

                fn symmetric_difference(f, other) {
                    Self::from_bits_retain(f.bits() ^ other.bits())
                }

                fn complement(f) {
                    Self::from_bits_truncate(!f.bits())
                }
            }
        }
    };
}

/// Implement iterators on the public (user-facing) bitflags type.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_public_bitflags_iter {
    ($BitFlags:ident: $T:ty, $PublicBitFlags:ident) => {
        impl $BitFlags {
            /// Iterate over enabled flag values.
            #[inline]
            pub const fn iter(&self) -> $crate::iter::Iter<$PublicBitFlags> {
                $crate::iter::Iter::__private_const_new(
                    <$PublicBitFlags as $crate::Flags>::FLAGS,
                    $PublicBitFlags::from_bits_retain(self.bits()),
                    $PublicBitFlags::from_bits_retain(self.bits()),
                )
            }

            /// Iterate over enabled flag values with their stringified names.
            #[inline]
            pub const fn iter_names(&self) -> $crate::iter::IterNames<$PublicBitFlags> {
                $crate::iter::IterNames::__private_const_new(
                    <$PublicBitFlags as $crate::Flags>::FLAGS,
                    $PublicBitFlags::from_bits_retain(self.bits()),
                    $PublicBitFlags::from_bits_retain(self.bits()),
                )
            }
        }

        impl $crate::__private::core::iter::IntoIterator for $BitFlags {
            type Item = $PublicBitFlags;
            type IntoIter = $crate::iter::Iter<$PublicBitFlags>;

            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
    };
}

/// Implement traits on the public (user-facing) bitflags type.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_public_bitflags_ops {
    ($PublicBitFlags:ident) => {
        impl $crate::__private::core::fmt::Binary for $PublicBitFlags {
            fn fmt(
                &self,
                f: &mut $crate::__private::core::fmt::Formatter,
            ) -> $crate::__private::core::fmt::Result {
                $crate::__private::core::fmt::Binary::fmt(&self.0, f)
            }
        }

        impl $crate::__private::core::fmt::Octal for $PublicBitFlags {
            fn fmt(
                &self,
                f: &mut $crate::__private::core::fmt::Formatter,
            ) -> $crate::__private::core::fmt::Result {
                $crate::__private::core::fmt::Octal::fmt(&self.0, f)
            }
        }

        impl $crate::__private::core::fmt::LowerHex for $PublicBitFlags {
            fn fmt(
                &self,
                f: &mut $crate::__private::core::fmt::Formatter,
            ) -> $crate::__private::core::fmt::Result {
                $crate::__private::core::fmt::LowerHex::fmt(&self.0, f)
            }
        }

        impl $crate::__private::core::fmt::UpperHex for $PublicBitFlags {
            fn fmt(
                &self,
                f: &mut $crate::__private::core::fmt::Formatter,
            ) -> $crate::__private::core::fmt::Result {
                $crate::__private::core::fmt::UpperHex::fmt(&self.0, f)
            }
        }

        impl $crate::__private::core::ops::BitOr for $PublicBitFlags {
            type Output = Self;

            /// Returns the union of the two sets of flags.
            #[inline]
            fn bitor(self, other: $PublicBitFlags) -> Self {
                self.union(other)
            }
        }

        impl $crate::__private::core::ops::BitOrAssign for $PublicBitFlags {
            /// Adds the set of flags.
            #[inline]
            fn bitor_assign(&mut self, other: Self) {
                *self = Self::from_bits_retain(self.bits()).union(other);
            }
        }

        impl $crate::__private::core::ops::BitXor for $PublicBitFlags {
            type Output = Self;

            /// Returns the left flags, but with all the right flags toggled.
            #[inline]
            fn bitxor(self, other: Self) -> Self {
                self.symmetric_difference(other)
            }
        }

        impl $crate::__private::core::ops::BitXorAssign for $PublicBitFlags {
            /// Toggles the set of flags.
            #[inline]
            fn bitxor_assign(&mut self, other: Self) {
                *self = Self::from_bits_retain(self.bits()).symmetric_difference(other);
            }
        }

        impl $crate::__private::core::ops::BitAnd for $PublicBitFlags {
            type Output = Self;

            /// Returns the intersection between the two sets of flags.
            #[inline]
            fn bitand(self, other: Self) -> Self {
                self.intersection(other)
            }
        }

        impl $crate::__private::core::ops::BitAndAssign for $PublicBitFlags {
            /// Disables all flags disabled in the set.
            #[inline]
            fn bitand_assign(&mut self, other: Self) {
                *self = Self::from_bits_retain(self.bits()).intersection(other);
            }
        }

        impl $crate::__private::core::ops::Sub for $PublicBitFlags {
            type Output = Self;

            /// Returns the set difference of the two sets of flags.
            #[inline]
            fn sub(self, other: Self) -> Self {
                self.difference(other)
            }
        }

        impl $crate::__private::core::ops::SubAssign for $PublicBitFlags {
            /// Disables all flags enabled in the set.
            #[inline]
            fn sub_assign(&mut self, other: Self) {
                *self = Self::from_bits_retain(self.bits()).difference(other);
            }
        }

        impl $crate::__private::core::ops::Not for $PublicBitFlags {
            type Output = Self;

            /// Returns the complement of this set of flags.
            #[inline]
            fn not(self) -> Self {
                self.complement()
            }
        }

        impl $crate::__private::core::iter::Extend<$PublicBitFlags> for $PublicBitFlags {
            fn extend<T: $crate::__private::core::iter::IntoIterator<Item = Self>>(
                &mut self,
                iterator: T,
            ) {
                for item in iterator {
                    self.insert(item)
                }
            }
        }

        impl $crate::__private::core::iter::FromIterator<$PublicBitFlags> for $PublicBitFlags {
            fn from_iter<T: $crate::__private::core::iter::IntoIterator<Item = Self>>(
                iterator: T,
            ) -> Self {
                use $crate::__private::core::iter::Extend;

                let mut result = Self::empty();
                result.extend(iterator);
                result
            }
        }
    };
}

/// Implement constants on the public (user-facing) bitflags type.
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! __impl_public_bitflags_consts {
    (
        $PublicBitFlags:ident: $T:ty {
            $(
                $(#[$attr:ident $($args:tt)*])*
                $Flag:ident = $value:expr;
            )*
        }
    ) => {
        impl $PublicBitFlags {
            $(
                $(#[$attr $($args)*])*
                #[allow(
                    deprecated,
                    non_upper_case_globals,
                )]
                pub const $Flag: Self = Self::from_bits_retain($value);
            )*
        }

        impl $crate::Flags for $PublicBitFlags {
            const FLAGS: &'static [$crate::Flag<$PublicBitFlags>] = &[
                $(
                    __bitflags_expr_safe_attrs!(
                        $(#[$attr $($args)*])*
                        {
                            #[allow(
                                deprecated,
                                non_upper_case_globals,
                            )]
                            $crate::Flag::new($crate::__private::core::stringify!($Flag), $PublicBitFlags::$Flag)
                        }
                    ),
                )*
            ];

            type Bits = $T;

            fn bits(&self) -> $T {
                $PublicBitFlags::bits(self)
            }

            fn from_bits_retain(bits: $T) -> $PublicBitFlags {
                $PublicBitFlags::from_bits_retain(bits)
            }
        }
    };
}
