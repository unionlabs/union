use core::{
    fmt,
    ops::{BitAnd, BitOr, BitXor, Not},
};

use crate::{
    iter,
    parser::{ParseError, ParseHex, WriteHex},
};

/// Metadata for an individual flag.
pub struct Flag<B> {
    name: &'static str,
    value: B,
}

impl<B> Flag<B> {
    /// Create a new flag with the given name and value.
    pub const fn new(name: &'static str, value: B) -> Self {
        Flag { name, value }
    }

    /// Get the name of this flag.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// Get the value of this flag.
    pub const fn value(&self) -> &B {
        &self.value
    }
}

/// A set of flags.
///
/// This trait is automatically implemented for flags types defined using the `bitflags!` macro.
/// It can also be implemented manually for custom flags types.
pub trait Flags: Sized + 'static {
    /// The set of available flags and their names.
    const FLAGS: &'static [Flag<Self>];

    /// The underlying storage type.
    type Bits: Bits;

    /// Returns an empty set of flags.
    fn empty() -> Self {
        Self::from_bits_retain(Self::Bits::EMPTY)
    }

    /// Returns the set containing all flags.
    fn all() -> Self {
        Self::from_bits_truncate(Self::Bits::ALL)
    }

    /// Returns the raw value of the flags currently stored.
    fn bits(&self) -> Self::Bits;

    /// Convert from underlying bit representation, unless that
    /// representation contains bits that do not correspond to a flag.
    ///
    /// Note that each [multi-bit flag] is treated as a unit for this comparison.
    ///
    /// [multi-bit flag]: index.html#multi-bit-flags
    fn from_bits(bits: Self::Bits) -> Option<Self> {
        let truncated = Self::from_bits_truncate(bits);

        if truncated.bits() == bits {
            Some(truncated)
        } else {
            None
        }
    }

    /// Convert from underlying bit representation, dropping any bits
    /// that do not correspond to flags.
    ///
    /// Note that each [multi-bit flag] is treated as a unit for this comparison.
    ///
    /// [multi-bit flag]: index.html#multi-bit-flags
    fn from_bits_truncate(bits: Self::Bits) -> Self {
        if bits == Self::Bits::EMPTY {
            return Self::empty();
        }

        let mut truncated = Self::Bits::EMPTY;

        for flag in Self::FLAGS.iter() {
            let flag = flag.value();

            if bits & flag.bits() == flag.bits() {
                truncated = truncated | flag.bits();
            }
        }

        Self::from_bits_retain(truncated)
    }

    /// Convert from underlying bit representation, preserving all
    /// bits (even those not corresponding to a defined flag).
    fn from_bits_retain(bits: Self::Bits) -> Self;

    /// Get the flag for a particular name.
    fn from_name(name: &str) -> Option<Self> {
        for flag in Self::FLAGS {
            if flag.name() == name {
                return Some(Self::from_bits_retain(flag.value().bits()));
            }
        }

        None
    }

    /// Iterate over enabled flag values.
    fn iter(&self) -> iter::Iter<Self> {
        iter::Iter::new(self)
    }

    /// Iterate over the raw names and bits for enabled flag values.
    fn iter_names(&self) -> iter::IterNames<Self> {
        iter::IterNames::new(self)
    }

    /// Returns `true` if no flags are currently stored.
    fn is_empty(&self) -> bool {
        self.bits() == Self::Bits::EMPTY
    }

    /// Returns `true` if all flags are currently set.
    fn is_all(&self) -> bool {
        // NOTE: We check against `Self::all` here, not `Self::Bits::ALL`
        // because the set of all flags may not use all bits
        Self::all().bits() | self.bits() == self.bits()
    }

    /// Returns `true` if there are flags common to both `self` and `other`.
    fn intersects(&self, other: Self) -> bool
    where
        Self: Sized,
    {
        self.bits() & other.bits() != Self::Bits::EMPTY
    }

    /// Returns `true` if all of the flags in `other` are contained within `self`.
    fn contains(&self, other: Self) -> bool
    where
        Self: Sized,
    {
        self.bits() & other.bits() == other.bits()
    }

    /// Inserts the specified flags in-place.
    ///
    /// This method is equivalent to `union`.
    fn insert(&mut self, other: Self)
    where
        Self: Sized,
    {
        *self = Self::from_bits_retain(self.bits() | other.bits());
    }

    /// Removes the specified flags in-place.
    ///
    /// This method is equivalent to `difference`.
    fn remove(&mut self, other: Self)
    where
        Self: Sized,
    {
        *self = Self::from_bits_retain(self.bits() & !other.bits());
    }

    /// Toggles the specified flags in-place.
    ///
    /// This method is equivalent to `symmetric_difference`.
    fn toggle(&mut self, other: Self)
    where
        Self: Sized,
    {
        *self = Self::from_bits_retain(self.bits() ^ other.bits());
    }

    /// Inserts or removes the specified flags depending on the passed value.
    fn set(&mut self, other: Self, value: bool)
    where
        Self: Sized,
    {
        if value {
            self.insert(other);
        } else {
            self.remove(other);
        }
    }

    /// Returns the intersection between the flags in `self` and `other`.
    #[must_use]
    fn intersection(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits() & other.bits())
    }

    /// Returns the union of between the flags in `self` and `other`.
    #[must_use]
    fn union(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits() | other.bits())
    }

    /// Returns the difference between the flags in `self` and `other`.
    #[must_use]
    fn difference(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits() & !other.bits())
    }

    /// Returns the symmetric difference between the flags
    /// in `self` and `other`.
    #[must_use]
    fn symmetric_difference(self, other: Self) -> Self {
        Self::from_bits_retain(self.bits() ^ other.bits())
    }

    /// Returns the complement of this set of flags.
    #[must_use]
    fn complement(self) -> Self {
        Self::from_bits_truncate(!self.bits())
    }
}

/// Underlying storage for a flags type.
pub trait Bits:
    Clone
    + Copy
    + PartialEq
    + BitAnd<Output = Self>
    + BitOr<Output = Self>
    + BitXor<Output = Self>
    + Not<Output = Self>
    + Sized
    + 'static
{
    /// The value of `Self` where no bits are set.
    const EMPTY: Self;

    /// The value of `Self` where all bits are set.
    const ALL: Self;
}

// Not re-exported: prevent custom `Bits` impls being used in the `bitflags!` macro,
// or they may fail to compile based on crate features
pub trait Primitive {}

macro_rules! impl_bits {
    ($($u:ty, $i:ty,)*) => {
        $(
            impl Bits for $u {
                const EMPTY: $u = 0;
                const ALL: $u = <$u>::MAX;
            }

            impl Bits for $i {
                const EMPTY: $i = 0;
                const ALL: $i = <$u>::MAX as $i;
            }

            impl ParseHex for $u {
                fn parse_hex(input: &str) -> Result<Self, ParseError> {
                    <$u>::from_str_radix(input, 16).map_err(|_| ParseError::invalid_hex_flag(input))
                }
            }

            impl ParseHex for $i {
                fn parse_hex(input: &str) -> Result<Self, ParseError> {
                    <$i>::from_str_radix(input, 16).map_err(|_| ParseError::invalid_hex_flag(input))
                }
            }

            impl WriteHex for $u {
                fn write_hex<W: fmt::Write>(&self, mut writer: W) -> fmt::Result {
                    write!(writer, "{:x}", self)
                }
            }

            impl WriteHex for $i {
                fn write_hex<W: fmt::Write>(&self, mut writer: W) -> fmt::Result {
                    write!(writer, "{:x}", self)
                }
            }

            impl Primitive for $i {}
            impl Primitive for $u {}
        )*
    }
}

impl_bits! {
    u8, i8,
    u16, i16,
    u32, i32,
    u64, i64,
    u128, i128,
    usize, isize,
}

/// A trait for referencing the `bitflags`-owned internal type
/// without exposing it publicly.
pub trait PublicFlags {
    /// The type of the underlying storage.
    type Primitive: Primitive;

    /// The type of the internal field on the generated flags type.
    type Internal;
}

#[deprecated(note = "use the `Flags` trait instead")]
pub trait BitFlags: ImplementedByBitFlagsMacro + Flags {
    /// An iterator over enabled flags in an instance of the type.
    type Iter: Iterator<Item = Self>;

    /// An iterator over the raw names and bits for enabled flags in an instance of the type.
    type IterNames: Iterator<Item = (&'static str, Self)>;
}

#[allow(deprecated)]
impl<B: Flags> BitFlags for B {
    type Iter = iter::Iter<Self>;
    type IterNames = iter::IterNames<Self>;
}

impl<B: Flags> ImplementedByBitFlagsMacro for B {}

/// A marker trait that signals that an implementation of `BitFlags` came from the `bitflags!` macro.
///
/// There's nothing stopping an end-user from implementing this trait, but we don't guarantee their
/// manual implementations won't break between non-breaking releases.
#[doc(hidden)]
pub trait ImplementedByBitFlagsMacro {}

pub(crate) mod __private {
    pub use super::{ImplementedByBitFlagsMacro, PublicFlags};
}
