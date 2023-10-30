//! Details about stacks used by coroutines.
//!
//! In most cases, just using the default stack implementation is sufficient.
//! However it is possible to use a custom `Stack` implementation if more
//! control is desired.

use core::num::NonZeroUsize;

pub mod valgrind;

cfg_if::cfg_if! {
    if #[cfg(all(feature = "default-stack", unix))] {
        mod unix;
        pub use self::unix::DefaultStack;
    } else if #[cfg(all(feature = "default-stack", windows))] {
        mod windows;
        pub use self::windows::DefaultStack;
    } else {
        /// Dummy stack for platforms that do not provide a default stack.
        ///
        /// This is only here for use as a default generic parameter.
        pub struct DefaultStack;
    }
}

/// Type to represent a stack address.
pub type StackPointer = NonZeroUsize;

/// Required stack alignment at function call boundaries.
pub const STACK_ALIGNMENT: usize = crate::arch::STACK_ALIGNMENT;

/// Minimum size of a stack, excluding guard pages.
pub const MIN_STACK_SIZE: usize = 4096;

/// A trait for objects that hold ownership of a stack.
///
/// # Safety
///
/// To preserve memory safety, a stack must have a guard page which catches
/// stack overflows. It must also contain at least [`MIN_STACK_SIZE`] bytes of
/// usable memory.
///
/// # Windows
///
/// On Windows, this trait has some extra methods which are needed because the
/// OS is much more involved with stack management than other platforms.
pub unsafe trait Stack {
    /// Returns the base address of the stack. This is the highest address since
    /// stacks grow downwards on most modern architectures.
    ///
    /// Must be aligned to [`STACK_ALIGNMENT`].
    fn base(&self) -> StackPointer;

    /// Returns the maximum limit address of the stack. This is the lowest
    /// address since stacks grow downwards on most modern architectures.
    ///
    /// This limit must include any guard pages in the stack.
    ///
    /// Must be aligned to [`STACK_ALIGNMENT`].
    fn limit(&self) -> StackPointer;

    /// On Windows, certain fields must be updated in the Thread Environment
    /// Block when switching to another stack. This function returns the values
    /// that must be assigned for this stack.
    #[cfg(windows)]
    fn teb_fields(&self) -> StackTebFields;

    /// Updates the stack's copy of TEB fields which may have changed while
    /// executing code on the stack.
    #[cfg(windows)]
    fn update_teb_fields(&mut self, stack_limit: usize, guaranteed_stack_bytes: usize);
}

/// Fields in the Thread Environment Block (TEB) which must be updated when
/// switching to a different stack. These are the same fields that are updated
/// by the `SwitchToFiber` function in the Windows API.
#[cfg(windows)]
#[derive(Clone, Copy, Debug)]
#[allow(non_snake_case)]
#[allow(missing_docs)]
pub struct StackTebFields {
    pub StackBase: usize,
    pub StackLimit: usize,
    pub DeallocationStack: usize,
    pub GuaranteedStackBytes: usize,
}

/// A mutable reference to a stack can be used as a stack. The lifetime of the
/// resulting coroutine will be bound to that of the reference.
unsafe impl<'a, S: Stack> Stack for &'a mut S {
    #[inline]
    fn base(&self) -> StackPointer {
        (**self).base()
    }

    #[inline]
    fn limit(&self) -> StackPointer {
        (**self).limit()
    }

    #[inline]
    #[cfg(windows)]
    fn teb_fields(&self) -> StackTebFields {
        (**self).teb_fields()
    }

    #[inline]
    #[cfg(windows)]
    fn update_teb_fields(&mut self, stack_limit: usize, guaranteed_stack_bytes: usize) {
        (**self).update_teb_fields(stack_limit, guaranteed_stack_bytes)
    }
}

#[test]
fn assert_send_sync() {
    fn send<T: Send>() {}
    fn sync<T: Sync>() {}
    send::<DefaultStack>();
    sync::<DefaultStack>();
}
