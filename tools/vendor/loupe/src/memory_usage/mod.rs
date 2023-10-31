mod alloc;
mod any;
mod r#box;
mod cell;
mod collection;
mod marker;
mod option;
mod path;
mod primitive;
mod ptr;
mod remote;
mod result;
mod slice;
mod string;
mod sync;

pub use alloc::*;
pub use any::*;
pub use cell::*;
pub use collection::*;
pub use marker::*;
pub use option::*;
pub use path::*;
pub use primitive::*;
pub use ptr::*;
pub use r#box::*;
pub use remote::*;
pub use result::*;
pub use slice::*;
pub use string::*;
pub use sync::*;

/// Size of a pointer for the compilation target.
pub const POINTER_BYTE_SIZE: usize = if cfg!(target_pointer_width = "16") {
    2
} else if cfg!(target_pointer_width = "32") {
    4
} else {
    8
};

/// Represent a bucket that can track memory addresses that have
/// already been visited by `MemoryUsage`.
pub trait MemoryUsageTracker {
    /// When first called on a given address returns true, false otherwise.
    fn track(&mut self, address: *const ()) -> bool;
}

impl MemoryUsageTracker for std::collections::BTreeSet<*const ()> {
    fn track(&mut self, address: *const ()) -> bool {
        self.insert(address)
    }
}

impl MemoryUsageTracker for std::collections::HashSet<*const ()> {
    fn track(&mut self, address: *const ()) -> bool {
        self.insert(address)
    }
}

/// Traverse a value and collect its memory usage.
pub trait MemoryUsage {
    /// Returns the size of the referenced value in bytes.
    ///
    /// Recursively visits the value and any children returning the sum of their
    /// sizes. The size always includes any tail padding if applicable.
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize;
}

/// Alias to `assert_eq!(loupe::MemoryUsage::size_of_val(&$value), $expected)`.
#[macro_export]
macro_rules! assert_size_of_val_eq {
    ($value:expr, $expected:expr $(,)*) => {
        assert_size_of_val_eq!($value, $expected, &mut std::collections::BTreeSet::new());
    };

    ($value:expr, $expected:expr, $tracker:expr $(,)*) => {
        assert_eq!(
            $crate::MemoryUsage::size_of_val(&$value, $tracker),
            $expected
        );
    };
}

// TODO:
//
// * Cell
// * Pin (is a Pin always referenceable?)
// * Rc
// * Ref
// * RefCell
// * RefMut
// * PhantomPinned
