// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

/// A type wrapping a small integer index should implement `EntityRef` so it can be used as the key
/// of an `SecondaryMap` or `SparseMap`.
pub trait EntityRef: Copy + Eq {
    /// Create a new entity reference from a small integer.
    /// This should crash if the requested index is not representable.
    fn new(_: usize) -> Self;

    /// Get the index that was used to create this entity reference.
    fn index(self) -> usize;
}

/// Macro which provides the common implementation of a 32-bit entity reference.
#[macro_export]
macro_rules! entity_impl {
    // Basic traits.
    ($entity:ident) => {
        impl $crate::entity::EntityRef for $entity {
            fn new(index: usize) -> Self {
                debug_assert!(index < ($crate::lib::std::u32::MAX as usize));
                $entity(index as u32)
            }

            fn index(self) -> usize {
                self.0 as usize
            }
        }

        impl $crate::entity::packed_option::ReservedValue for $entity {
            fn reserved_value() -> $entity {
                $entity($crate::lib::std::u32::MAX)
            }

            fn is_reserved_value(&self) -> bool {
                self.0 == $crate::lib::std::u32::MAX
            }
        }

        impl $entity {
            /// Create a new instance from a `u32`.
            #[allow(dead_code)]
            pub fn from_u32(x: u32) -> Self {
                debug_assert!(x < $crate::lib::std::u32::MAX);
                $entity(x)
            }

            /// Return the underlying index value as a `u32`.
            #[allow(dead_code)]
            pub fn as_u32(self) -> u32 {
                self.0
            }
        }
    };

    // Include basic `Display` impl using the given display prefix.
    // Display a `Block` reference as "block12".
    ($entity:ident, $display_prefix:expr) => {
        entity_impl!($entity);

        impl $crate::lib::std::fmt::Display for $entity {
            fn fmt(
                &self,
                f: &mut $crate::lib::std::fmt::Formatter,
            ) -> $crate::lib::std::fmt::Result {
                write!(f, concat!($display_prefix, "{}"), self.0)
            }
        }

        impl $crate::lib::std::fmt::Debug for $entity {
            fn fmt(
                &self,
                f: &mut $crate::lib::std::fmt::Formatter,
            ) -> $crate::lib::std::fmt::Result {
                (self as &dyn $crate::lib::std::fmt::Display).fmt(f)
            }
        }
    };
}

pub mod packed_option;

mod boxed_slice;
mod iter;
mod keys;
mod primary_map;
mod secondary_map;

pub use crate::entity_impl;
pub use boxed_slice::BoxedSlice;
pub use iter::{Iter, IterMut};
pub use keys::Keys;
pub use primary_map::PrimaryMap;
pub use secondary_map::SecondaryMap;
