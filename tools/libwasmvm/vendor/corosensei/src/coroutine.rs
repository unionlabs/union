use core::cell::Cell;
use core::hint::unreachable_unchecked;
use core::marker::PhantomData;
use core::mem::{self, ManuallyDrop};
use core::ptr;

use crate::arch::{self, STACK_ALIGNMENT};
#[cfg(windows)]
use crate::stack::StackTebFields;
use crate::stack::{self, DefaultStack, StackPointer};
use crate::trap::CoroutineTrapHandler;
use crate::unwind::{self, initial_func_abi, CaughtPanic, ForcedUnwindErr};
use crate::util::{self, EncodedValue};

/// Value returned from resuming a coroutine.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CoroutineResult<Yield, Return> {
    /// Value returned by a coroutine suspending itself with a `Yielder`.
    Yield(Yield),

    /// Value returned by a coroutine returning from its main function.
    Return(Return),
}

impl<Yield, Return> CoroutineResult<Yield, Return> {
    /// Returns the `Yield` value as an `Option<Yield>`.
    pub fn as_yield(self) -> Option<Yield> {
        match self {
            CoroutineResult::Yield(val) => Some(val),
            CoroutineResult::Return(_) => None,
        }
    }

    /// Returns the `Return` value as an `Option<Return>`.
    pub fn as_return(self) -> Option<Return> {
        match self {
            CoroutineResult::Yield(_) => None,
            CoroutineResult::Return(val) => Some(val),
        }
    }
}

/// Alias for a [`ScopedCoroutine`] with a `'static` lifetime.
///
/// This means that the function executing in the coroutine does not borrow
/// anything from its caller.
pub type Coroutine<Input, Yield, Return, Stack = DefaultStack> =
    ScopedCoroutine<'static, Input, Yield, Return, Stack>;

/// A coroutine wraps a closure and allows suspending its execution more than
/// once, returning a value each time.
///
/// # Lifetime
///
/// The `'a` lifetime here refers to the lifetime of the initial function in a
/// coroutine and ensures that the coroutine doesn't exceed the lifetime of the
/// initial function.
///
/// # Dropping a coroutine
///
/// When a coroutine is dropped, its stack must be unwound so that all object on
/// it are properly dropped. This is done by calling `force_unwind` to unwind
/// the stack. If `force_unwind` fails then the program is aborted.
///
/// See the [`Coroutine::force_unwind`] function for more details.
///
/// # `Send`
///
/// In the general case, a coroutine can only be sent to another if all of the
/// data on its stack is `Send`. There is no way to guarantee this using Rust
/// language features so `Coroutine` does not implement the `Send` trait.
///
/// However if all of the code executed by a coroutine is under your control and
/// you can ensure that all types on the stack when a coroutine is suspended
/// are `Send` then it is safe to manually implement `Send` for a coroutine.
pub struct ScopedCoroutine<'a, Input, Yield, Return, Stack: stack::Stack> {
    // Stack that the coroutine is executing on.
    stack: Stack,

    // Current stack pointer at which the coroutine state is held. This is
    // None when the coroutine has completed execution.
    stack_ptr: Option<StackPointer>,

    // Initial stack pointer value. This is used to detect whether a coroutine
    // has ever been resumed since it was created.
    //
    // This works because it is impossible for a coroutine to revert back to its
    // initial stack pointer: suspending a coroutine requires pushing several
    // values to the stack.
    initial_stack_ptr: StackPointer,

    // Function to call to drop the initial state of a coroutine if it has
    // never been resumed.
    drop_fn: unsafe fn(ptr: *mut u8),

    // We want to be covariant over 'a, Yield and Return, and contravariant
    // over Input.
    //
    // Effectively this means that we can pass a
    //   ScopedCoroutine<'static, &'a (), &'static (), &'static ()>
    // to a function that expects a
    //   ScopedCoroutine<'b, &'static (), &'c (), &'d ()>
    marker: PhantomData<&'a fn(Input) -> CoroutineResult<Yield, Return>>,

    // Coroutine must be !Send.
    /// ```compile_fail
    /// fn send<T: Send>() {}
    /// send::<corosensei::Coroutine<(), ()>>();
    /// ```
    marker2: PhantomData<*mut ()>,
}

// Coroutines can be Sync if the stack is Sync.
unsafe impl<Input, Yield, Return, Stack: stack::Stack + Sync> Sync
    for ScopedCoroutine<'_, Input, Yield, Return, Stack>
{
}

#[cfg(feature = "default-stack")]
impl<'a, Input, Yield, Return> ScopedCoroutine<'a, Input, Yield, Return, DefaultStack> {
    /// Creates a new coroutine which will execute `func` on a new stack.
    ///
    /// This function returns a `Coroutine` which, when resumed, will execute
    /// `func` to completion. When desired the `func` can suspend itself via
    /// `Yielder::suspend`.
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(&Yielder<Input, Yield>, Input) -> Return,
        F: 'a,
    {
        Self::with_stack(Default::default(), f)
    }
}

impl<'a, Input, Yield, Return, Stack: stack::Stack>
    ScopedCoroutine<'a, Input, Yield, Return, Stack>
{
    /// Creates a new coroutine which will execute `func` on the given stack.
    ///
    /// This function returns a coroutine which, when resumed, will execute
    /// `func` to completion. When desired the `func` can suspend itself via
    /// [`Yielder::suspend`].
    pub fn with_stack<F>(stack: Stack, f: F) -> Self
    where
        F: FnOnce(&Yielder<Input, Yield>, Input) -> Return,
        F: 'a,
    {
        // The ABI of the initial function is either "C" or "C-unwind" depending
        // on whether the "asm-unwind" feature is enabled.
        initial_func_abi! {
            unsafe fn coroutine_func<Input, Yield, Return, F>(
                input: EncodedValue,
                parent_link: &mut StackPointer,
                func: *mut F,
            ) -> !
            where
                F: FnOnce(&Yielder<Input, Yield>, Input) -> Return,
            {
                // The yielder is a #[repr(transparent)] wrapper around the
                // parent link on the stack.
                let yielder = &*(parent_link as *mut StackPointer as *const Yielder<Input, Yield>);

                // Read the function from the stack.
                debug_assert_eq!(func as usize % mem::align_of::<F>(), 0);
                let f = func.read();

                // This is the input from the first call to resume(). It is not
                // possible for a forced unwind to reach this point because we
                // check if a coroutine has been resumed at least once before
                // generating a forced unwind.
                let input : Result<Input, ForcedUnwindErr> = util::decode_val(input);
                let input = match input {
                    Ok(input) => input,
                    Err(_) => unreachable_unchecked(),
                };

                // Run the body of the generator, catching any panics.
                let result = unwind::catch_unwind_at_root(|| f(yielder, input));

                // Return any caught panics to the parent context.
                let mut result = ManuallyDrop::new(result);
                arch::switch_and_reset(util::encode_val(&mut result), yielder.stack_ptr.as_ptr());
            }
        }

        // Drop function to free the initial state of the coroutine.
        unsafe fn drop_fn<T>(ptr: *mut u8) {
            ptr::drop_in_place(ptr as *mut T);
        }

        unsafe {
            // Set up the stack so that the coroutine starts executing
            // coroutine_func. Write the given function object to the stack so
            // its address is passed to coroutine_func on the first resume.
            let stack_ptr = arch::init_stack(&stack, coroutine_func::<Input, Yield, Return, F>, f);

            Self {
                stack,
                stack_ptr: Some(stack_ptr),
                initial_stack_ptr: stack_ptr,
                drop_fn: drop_fn::<F>,
                marker: PhantomData,
                marker2: PhantomData,
            }
        }
    }

    /// Resumes execution of this coroutine.
    ///
    /// This function will transfer execution to the coroutine and resume from
    /// where it last left off.
    ///
    /// If the coroutine calls [`Yielder::suspend`] then this function returns
    /// [`CoroutineResult::Yield`] with the value passed to `suspend`.
    ///
    /// If the coroutine returns then this function returns
    /// [`CoroutineResult::Return`] with the return value of the coroutine.
    ///
    /// # Panics
    ///
    /// Panics if the coroutine has already finished executing.
    ///
    /// If the coroutine itself panics during execution then the panic will be
    /// propagated to this caller.
    pub fn resume(&mut self, val: Input) -> CoroutineResult<Yield, Return> {
        unsafe {
            let stack_ptr = self
                .stack_ptr
                .expect("attempt to resume a completed coroutine");

            // If the coroutine terminated then a caught panic may have been
            // returned, in which case we must resume unwinding.
            match self.resume_inner(stack_ptr, Ok(val)) {
                CoroutineResult::Yield(val) => CoroutineResult::Yield(val),
                CoroutineResult::Return(result) => {
                    CoroutineResult::Return(unwind::maybe_resume_unwind(result))
                }
            }
        }
    }

    /// Common code for resuming execution of a coroutine.
    unsafe fn resume_inner(
        &mut self,
        stack_ptr: StackPointer,
        input: Result<Input, ForcedUnwindErr>,
    ) -> CoroutineResult<Yield, Result<Return, CaughtPanic>> {
        // Pre-emptively set the stack pointer to None in case
        // switch_and_link unwinds.
        self.stack_ptr = None;

        let mut input = ManuallyDrop::new(input);
        let (result, stack_ptr) =
            arch::switch_and_link(util::encode_val(&mut input), stack_ptr, self.stack.base());
        self.stack_ptr = stack_ptr;

        // Decode the returned value depending on whether the coroutine
        // terminated.
        if stack_ptr.is_some() {
            CoroutineResult::Yield(util::decode_val(result))
        } else {
            CoroutineResult::Return(util::decode_val(result))
        }
    }

    /// Returns whether this coroutine has been resumed at least once.
    pub fn started(&self) -> bool {
        self.stack_ptr != Some(self.initial_stack_ptr)
    }

    /// Returns whether this coroutine has finished executing.
    ///
    /// A coroutine that has returned from its initial function can no longer
    /// be resumed.
    pub fn done(&self) -> bool {
        self.stack_ptr.is_none()
    }

    /// Forcibly marks the coroutine as having completed, even if it is
    /// currently suspended in the middle of a function.
    ///
    /// # Safety
    ///
    /// This is equivalent to a `longjmp` all the way back to the initial
    /// function of the coroutine, so the same rules apply.
    ///
    /// This can only be done safely if there are no objects currently on the
    /// coroutine's stack that need to execute `Drop` code.
    pub unsafe fn force_reset(&mut self) {
        self.stack_ptr = None;
    }

    /// Unwinds the coroutine stack, dropping any live objects that are
    /// currently on the stack. This is automatically called when the coroutine
    /// is dropped.
    ///
    /// If the coroutine has already completed then this function is a no-op.
    ///
    /// If the coroutine is currently suspended on a `Yielder::suspend` call
    /// then unwinding it requires the `unwind` feature to be enabled and
    /// for the crate to be compiled with `-C panic=unwind`.
    ///
    /// # Panics
    ///
    /// This function panics if the coroutine could not be fully unwound. This
    /// can happen for one of two reasons:
    /// - The `ForcedUnwind` panic that is used internally was caught and not
    ///   rethrown.
    /// - This crate was compiled without the `unwind` feature and the
    ///   coroutine is currently suspended in the yielder (`started && !done`).
    pub fn force_unwind(&mut self) {
        // If the coroutine has already terminated then there is nothing to do.
        if let Some(stack_ptr) = self.stack_ptr {
            self.force_unwind_slow(stack_ptr);
        }
    }

    /// Slow path of `force_unwind` when the coroutine is known to not have
    /// terminated yet.
    #[cold]
    fn force_unwind_slow(&mut self, stack_ptr: StackPointer) {
        // If the coroutine has not started yet then we just need to drop the
        // initial object.
        if !self.started() {
            unsafe {
                arch::drop_initial_obj(self.stack.base(), stack_ptr, self.drop_fn);
            }
            self.stack_ptr = None;
            return;
        }

        // If the coroutine is suspended then we need the standard library so
        // that we can unwind the stack. This also requires that the code be
        // compiled with -C panic=unwind.
        #[cfg(feature = "unwind")]
        {
            extern crate std;

            let forced_unwind = unwind::ForcedUnwind(stack_ptr);
            let result = unwind::catch_forced_unwind(|| {
                #[cfg(not(feature = "asm-unwind"))]
                let result = unsafe { self.resume_inner(stack_ptr, Err(forced_unwind)) };
                #[cfg(feature = "asm-unwind")]
                let result = unsafe { self.resume_with_exception(stack_ptr, forced_unwind) };
                match result {
                    CoroutineResult::Yield(_) | CoroutineResult::Return(Ok(_)) => Ok(()),
                    CoroutineResult::Return(Err(e)) => Err(e),
                }
            });

            match result {
                Ok(_) => panic!("the ForcedUnwind panic was caught and not rethrown"),
                Err(e) => {
                    if let Some(forced_unwind) = e.downcast_ref::<unwind::ForcedUnwind>() {
                        if forced_unwind.0 == stack_ptr {
                            return;
                        }
                    }

                    std::panic::resume_unwind(e);
                }
            }
        }

        #[cfg(not(feature = "unwind"))]
        panic!("can't unwind a suspended coroutine without the \"unwind\" feature");
    }

    /// Variant of `resume_inner` that throws an exception in the context of
    /// the coroutine instead of passing a value.
    ///
    /// Used by `force_unwind`.
    #[cfg(feature = "asm-unwind")]
    unsafe fn resume_with_exception(
        &mut self,
        stack_ptr: StackPointer,
        forced_unwind: unwind::ForcedUnwind,
    ) -> CoroutineResult<Yield, Result<Return, CaughtPanic>> {
        // Pre-emptively set the stack pointer to None in case
        // switch_and_throw unwinds.
        self.stack_ptr = None;

        let (result, stack_ptr) =
            arch::switch_and_throw(forced_unwind, stack_ptr, self.stack.base());
        self.stack_ptr = stack_ptr;

        // Decode the returned value depending on whether the coroutine
        // terminated.
        if stack_ptr.is_some() {
            CoroutineResult::Yield(util::decode_val(result))
        } else {
            CoroutineResult::Return(util::decode_val(result))
        }
    }

    /// Extracts the stack from a coroutine that has finished executing.
    ///
    /// This allows the stack to be re-used for another coroutine.
    #[allow(unused_mut)]
    pub fn into_stack(mut self) -> Stack {
        assert!(
            self.done(),
            "cannot extract stack from an incomplete coroutine"
        );

        #[cfg(windows)]
        unsafe {
            arch::update_stack_teb_fields(&mut self.stack);
        }

        unsafe {
            let stack = ptr::read(&self.stack);
            mem::forget(self);
            stack
        }
    }

    /// Returns a [`CoroutineTrapHandler`] which can be used to handle traps that
    /// occur inside the coroutine. Examples of traps that can be handled are
    /// invalid memory accesses and stack overflows.
    ///
    /// The returned [`CoroutineTrapHandler`] can be used in a trap handler to
    /// force the trapping coroutine to return with a specific value, after
    /// which is it considered to have completed and can no longer be resumed.
    ///
    /// Needless to say, this is extremely unsafe and must be used with extreme
    /// care. See [`CoroutineTrapHandler::setup_trap_handler`] for the exact
    /// safety requirements.
    pub fn trap_handler(&self) -> CoroutineTrapHandler<Return> {
        CoroutineTrapHandler {
            stack_base: self.stack.base(),
            stack_limit: self.stack.limit(),
            marker: PhantomData,
        }
    }
}

impl<'a, Input, Yield, Return, Stack: stack::Stack> Drop
    for ScopedCoroutine<'a, Input, Yield, Return, Stack>
{
    fn drop(&mut self) {
        let guard = scopeguard::guard((), |()| {
            // We can't catch panics in #![no_std], force an abort using
            // a double-panic.
            panic!("cannot propagte coroutine panic with #![no_std]");
        });
        self.force_unwind();
        mem::forget(guard);

        #[cfg(windows)]
        unsafe {
            arch::update_stack_teb_fields(&mut self.stack);
        }
    }
}

/// `Yielder` is an interface provided to a coroutine which allows it to suspend
/// itself and pass values in and out of the coroutine.
///
/// `Yielder` can be freely copied to create multiple references to the same
/// underlying object. However a `Yielder` cannot be moved to another thread.
#[repr(transparent)]
pub struct Yielder<Input, Yield> {
    // Internally the Yielder is just the parent link on the stack which is
    // updated every time resume() is called.
    stack_ptr: Cell<StackPointer>,
    marker: PhantomData<fn(Yield) -> Input>,
}

impl<Input, Yield> Yielder<Input, Yield> {
    /// Suspends the execution of a currently running coroutine.
    ///
    /// This function will switch control back to the original caller of
    /// [`Coroutine::resume`]. This function will then return once the
    /// [`Coroutine::resume`] function is called again.
    pub fn suspend(&self, val: Yield) -> Input {
        unsafe {
            let mut val = ManuallyDrop::new(val);
            let result = arch::switch_yield(util::encode_val(&mut val), self.stack_ptr.as_ptr());
            unwind::maybe_force_unwind(util::decode_val(result))
        }
    }

    /// Executes some code on the stack of the parent context (the one who
    /// last resumed the current coroutine).
    ///
    /// This is particularly useful when executing on a coroutine with limited
    /// stack space: stack-heavy operations can be performed in a way that
    /// avoids stack overflows on the coroutine stack.
    ///
    /// # Panics
    ///
    /// Any panics in the provided closure are automatically propagated back up
    /// to the caller of this function.
    pub fn on_parent_stack<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
        // The F: Send bound here is somewhat subtle but important. It exists to
        // prevent references to the Yielder from being passed into the parent
        // thread.
        F: Send,
    {
        // Get the top of the parent stack.
        let stack_ptr = unsafe {
            StackPointer::new_unchecked(self.stack_ptr.get().get() - arch::PARENT_LINK_OFFSET)
        };

        // Create a virtual stack that starts below the parent stack.
        let stack = unsafe { ParentStack::new(stack_ptr) };

        on_stack(stack, f)
    }
}

/// Executes some code on the given stack.
///
/// This is useful when running with limited stack space: stack-intensive
/// computation can be executed on a separate stack with more space.
///
/// # Panics
///
/// Any panics in the provided closure are automatically propagated back up to
/// the caller of this function.
pub fn on_stack<F, R>(stack: impl stack::Stack, f: F) -> R
where
    F: FnOnce() -> R,
{
    // Union to hold both the function and its result.
    union FuncOrResult<F, R> {
        func: ManuallyDrop<F>,
        result: ManuallyDrop<Result<R, CaughtPanic>>,
    }

    initial_func_abi! {
        unsafe fn wrapper<F, R>(ptr: *mut u8)
        where
            F: FnOnce() -> R,
        {
            // Read the function out of the union.
            let data = &mut *(ptr as *mut FuncOrResult<F, R>);
            let func = ManuallyDrop::take(&mut data.func);

            // Call it.
            let result = unwind::catch_unwind_at_root(func);

            // And write the result back to the union.
            data.result = ManuallyDrop::new(result);
        }
    }

    unsafe {
        let mut data = FuncOrResult {
            func: ManuallyDrop::new(f),
        };

        // Call the wrapper function on the new stack.
        arch::on_stack(&mut data as *mut _ as *mut u8, stack, wrapper::<F, R>);

        // Re-throw any panics if one was caught.
        unwind::maybe_resume_unwind(ManuallyDrop::take(&mut data.result))
    }
}

/// Custom stack implementation used by `on_parent_stack`. This is a private
/// type because it is generally unsafe to use:
struct ParentStack {
    /// Base address of the stack, below any existing data on the parent stack.
    stack_base: StackPointer,

    /// Stack pointer value of the parent stack. This is not the same as
    /// `stack_base` since the latter has been aligned to `STACK_ALIGNMENT`.
    ///
    /// This is needed on Windows to access the saved TEB fields on the parent
    /// stack.
    #[cfg(windows)]
    stack_ptr: StackPointer,
}

impl ParentStack {
    #[inline]
    unsafe fn new(stack_ptr: StackPointer) -> Self {
        let stack_base = StackPointer::new_unchecked(stack_ptr.get() & !(STACK_ALIGNMENT - 1));
        Self {
            stack_base,
            #[cfg(windows)]
            stack_ptr,
        }
    }
}

unsafe impl stack::Stack for ParentStack {
    #[inline]
    fn base(&self) -> StackPointer {
        self.stack_base
    }

    // We can get away with a dummy implementation here because we never expose
    // the coroutine type to the user. This is only used for creating a
    // CoroutineTrapHandler.
    #[inline]
    fn limit(&self) -> StackPointer {
        self.stack_base
    }

    #[inline]
    #[cfg(windows)]
    fn teb_fields(&self) -> StackTebFields {
        unsafe { arch::read_parent_stack_teb_fields(self.stack_ptr) }
    }

    #[inline]
    #[cfg(windows)]
    fn update_teb_fields(&mut self, stack_limit: usize, guaranteed_stack_bytes: usize) {
        unsafe {
            arch::update_parent_stack_teb_fields(
                self.stack_ptr,
                stack_limit,
                guaranteed_stack_bytes,
            );
        }
    }
}
