//! Utilities for working with [`ScopedCoroutine::trap_handler`].

use core::marker::PhantomData;
use core::mem::ManuallyDrop;

pub use crate::arch::TrapHandlerRegs;
use crate::stack::StackPointer;
use crate::unwind::initial_func_abi;
#[cfg(doc)]
use crate::ScopedCoroutine; // For rustdoc to resolve doc-links
use crate::{arch, util};

/// Helper type to force a trapping coroutine to return from a trap handler.
///
/// This type does not use a lifetime so that it can be placed in static
/// thread-local storage for access by a trap handler. However it is UB to call
/// `setup_trap_return` past the lifetime of the originating coroutine.
///
/// See [`ScopedCoroutine::trap_handler`] for more details.
#[derive(Clone, Copy)]
pub struct CoroutineTrapHandler<Return> {
    pub(crate) stack_base: StackPointer,
    pub(crate) stack_limit: StackPointer,
    pub(crate) marker: PhantomData<fn(Return)>,
}

impl<Return> CoroutineTrapHandler<Return> {
    /// Checks whether the stack pointer at the point where a trap occurred is
    /// within the coroutine that this `CoroutineTrapHandler` was produced from.
    /// This check includes any guard pages on the stack and will therefore
    /// still return true in the case of a stack overflow.
    ///
    /// The result of this function is only meaningful if the coroutine has not
    /// been dropped yet.
    pub fn stack_ptr_in_bounds(&self, stack_ptr: usize) -> bool {
        stack_ptr >= self.stack_limit.get() && stack_ptr < self.stack_base.get()
    }

    /// Sets up the stack and register state so that once the trap handler
    /// returns, the interrupted coroutine will execute the given function and
    /// then exit the coroutine with the result as a `CoroutineResult::Return`.
    ///
    /// This function works by setting up the coroutine stack for a call to a
    /// special handler function. The returned `TrapHandlerRegs` indicate which
    /// registers the trap handler must update in the interrupted register
    /// context so that control is properly passed to this handler function.
    ///
    /// The stack is reset by this function: all data currently on the coroutine
    /// stack is discarded and the full stack is available for use by the given
    /// function.
    ///
    /// # Panics
    ///
    /// It is safe for the provided function to panic, this will simply cause
    /// the panic to unwind back to the parent of the coroutine. However no
    /// further traps may happen after this.
    ///
    /// # Safety
    ///
    /// Using this feature safely has very strict requirements. The coroutine
    /// must not write to any memory outside of itself except through volatile
    /// operations or assembly. Effectively, the coroutine must yield to its
    /// parent for any operation which requires interactions with its
    /// environment.
    ///
    /// Additionally, the exception context must have its stack pointer within
    /// the interrupted coroutine, as checked by `stack_ptr_in_bounds`. Since
    /// this function modifies the coroutine stack, the trap handler *must*
    /// update the interrupted register context as specified in the
    /// `TrapHandlerRegs`.
    pub unsafe fn setup_trap_handler<F>(&self, f: F) -> TrapHandlerRegs
    where
        F: FnOnce() -> Return,
        F: 'static,
    {
        initial_func_abi! {
            // This function is executed on the coroutine stack once execution
            // is resumed from the trap handler.
            unsafe fn trap_handler<F: FnOnce() -> Return, Return>(
                f: *mut F,
                parent_link: *mut StackPointer
            ) -> ! {
                // After returning from the exception handler we may have an
                // invalid SEH exception chain. We need to reset it to the
                // exception record at the root of the stack.
                #[cfg(all(windows, target_arch = "x86"))]
                arch::reset_seh_handler(parent_link);

                // This must be called after a stack overflow exception, but it
                // doesn't hurt to call it for other exception types as well.
                #[cfg(windows)]
                reset_guard_page();

                let result = crate::unwind::catch_unwind_at_root(f.read());

                let mut result = ManuallyDrop::new(result);
                arch::switch_and_reset(util::encode_val(&mut result), parent_link);
            }
        }

        // Move the value into the coroutine stack so that it is still available
        // after the trap handler returns. Then defer to the arch-specific code
        // to set up the register state for a call.
        arch::setup_trap_trampoline(self.stack_base, f, trap_handler::<F, Return>)
    }
}

/// On Windows, the guard pages at the end of a stack must be reset after a
/// stack overflow exception has occurred. This function must be called before
/// executing new code otherwise future stack overflow may not be caught.
///
/// For more information, see the [`_resetstkoflw`] function in the Windows
/// documentation.
///
/// [`_resetstkoflw`]: https://docs.microsoft.com/en-us/cpp/c-runtime-library/reference/resetstkoflw
#[cfg(windows)]
fn reset_guard_page() {
    extern "C" {
        fn _resetstkoflw() -> i32;
    }

    if unsafe { _resetstkoflw() } == 0 {
        panic!("failed to restore stack guard page");
    }
}
