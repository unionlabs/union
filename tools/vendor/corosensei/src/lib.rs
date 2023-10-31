//! ## Overview
//!
//! This crate provides a safe and efficient abstraction for context switching between different stacks, in the form of [coroutines]. A coroutine is a function that can be paused and resumed, yielding values to the caller. A coroutine can suspend itself from any point in its call stack. In addition to receiving yielded values from a coroutine, you can also pass data into the coroutine each time it is resumed.
//!
//! [coroutines]: https://en.wikipedia.org/wiki/Coroutine
//!
//! ## Example
//!
//! ```rust
//! use corosensei::{Coroutine, CoroutineResult};
//!
//! fn main() {
//!     println!("[main] creating coroutine");
//!
//!     let mut coroutine = Coroutine::new(|yielder, input| {
//!         println!("[coroutine] coroutine started with input {}", input);
//!         for i in 0..5 {
//!             println!("[coroutine] yielding {}", i);
//!             let input = yielder.suspend(i);
//!             println!("[coroutine] got {} from parent", input)
//!         }
//!         println!("[coroutine] exiting coroutine");
//!     });
//!
//!     let mut counter = 100;
//!     loop {
//!         println!("[main] resuming coroutine with argument {}", counter);
//!         match coroutine.resume(counter) {
//!             CoroutineResult::Yield(i) => println!("[main] got {:?} from coroutine", i),
//!             CoroutineResult::Return(()) => break,
//!         }
//!
//!         counter += 1;
//!     }
//!
//!     println!("[main] exiting");
//! }
//! ```
//!
//! #### Output
//!
//! ```text
//! [main] creating coroutine
//! [main] resuming coroutine with argument 100
//! [coroutine] coroutine started with input 100
//! [coroutine] yielding 0
//! [main] got 0 from coroutine
//! [main] resuming coroutine with argument 101
//! [coroutine] got 101 from parent
//! [coroutine] yielding 1
//! [main] got 1 from coroutine
//! [main] resuming coroutine with argument 102
//! [coroutine] got 102 from parent
//! [coroutine] yielding 2
//! [main] got 2 from coroutine
//! [main] resuming coroutine with argument 103
//! [coroutine] got 103 from parent
//! [coroutine] yielding 3
//! [main] got 3 from coroutine
//! [main] resuming coroutine with argument 104
//! [coroutine] got 104 from parent
//! [coroutine] yielding 4
//! [main] got 4 from coroutine
//! [main] resuming coroutine with argument 105
//! [coroutine] got 105 from parent
//! [coroutine] exiting coroutine
//! [main] exiting
//! ```
//!
//! ## Supported targets
//!
//! This crate currently supports the following targets:
//!
//! |         | ELF (Linux, BSD, bare metal, etc) | Darwin (macOS, iOS, etc) | Windows |
//! |---------|-----------------------------------|--------------------------|---------|
//! | x86_64  | ✅                                 | ✅                        | ✅       |
//! | x86     | ✅                                 | ✅                        | ⚠️*      |
//! | AArch64 | ✅                                 | ✅                        | ❌       |
//! | ARM     | ✅                                 | ❌                        | ❌       |
//! | RISC-V  | ✅                                 | ❌                        | ❌       |
//!
//! *\* Linked backtraces are not supported on x86 Windows.*
//!
//! Feel free to open an issue if your target is not supported yet.
//!
//! ## Features
//!
//! #### Panic propagation
//!
//! If a panic occurs in a coroutine then the panic will unwind through the coroutine stack and then continue to unwind out of the caller which last resumed it. Once this has happened, the coroutine is considered complete and can no longer be resumed.
//!
//! ```rust
//! use std::panic::{catch_unwind, AssertUnwindSafe};
//!
//! use corosensei::Coroutine;
//!
//! fn main() {
//!     println!("[main] creating coroutine");
//!
//!     let mut coroutine = Coroutine::new(|yielder, ()| {
//!         println!("[coroutine] yielding 42");
//!         yielder.suspend(42);
//!
//!         println!("[coroutine] panicking");
//!         panic!("foobar");
//!     });
//!
//!     println!("[main] resuming coroutine");
//!     let result = catch_unwind(AssertUnwindSafe(|| coroutine.resume(())));
//!     println!(
//!         "[main] got value {} from coroutine",
//!         result.unwrap().as_yield().unwrap()
//!     );
//!
//!     println!("[main] resuming coroutine");
//!     let result = catch_unwind(AssertUnwindSafe(|| coroutine.resume(())));
//!     println!(
//!         "[main] caught panic \"{}\" from coroutine",
//!         result.unwrap_err().downcast_ref::<&'static str>().unwrap()
//!     );
//!
//!     println!("[main] exiting");
//! }
//! ```
//!
//! ```text
//! [main] creating coroutine
//! [main] resuming coroutine
//! [coroutine] yielding 42
//! [main] got value 42 from coroutine
//! [main] resuming coroutine
//! [coroutine] panicking
//! thread 'main' panicked at 'foobar', examples/panic.rs:13:9
//! note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
//! [main] caught panic "foobar" from coroutine
//! [main] exiting
//! ```
//!
//! #### Linked backtraces
//!
//! Backtraces taken from within a coroutine will continue into the parent stack from the point where they were the coroutine was last resumed from. This is a significant help towards debugging issues with code in coroutines.
//!
//! Notice how the backtrace in this example shows that the coroutine was last resumed from `sub_function`:
//!
//! ```rust
//! use backtrace::Backtrace;
//! use corosensei::Coroutine;
//!
//! #[inline(never)]
//! fn sub_function(coroutine: &mut Coroutine<(), (), ()>) {
//!     coroutine.resume(());
//! }
//!
//! fn main() {
//!     let mut coroutine = Coroutine::new(move |_, ()| {
//!         let trace = Backtrace::new();
//!         println!("{:?}", trace);
//!     });
//!
//!     sub_function(&mut coroutine);
//! }
//! ```
//!
//! ```text
//!    0: backtrace::main::{{closure}}
//!              at examples/backtrace.rs:11:21
//!       corosensei::coroutine::ScopedCoroutine<Input,Yield,Return,Stack>::with_stack::coroutine_func::{{closure}}
//!              at src/coroutine.rs:174:62
//!       corosensei::unwind::catch_unwind_at_root
//!              at src/unwind.rs:43:16
//!    1: corosensei::coroutine::ScopedCoroutine<Input,Yield,Return,Stack>::with_stack::coroutine_func
//!              at src/coroutine.rs:174:30
//!    2: stack_init_trampoline
//!    3: corosensei::arch::x86_64::switch_and_link
//!              at src/arch/x86_64.rs:302:5
//!       corosensei::coroutine::ScopedCoroutine<Input,Yield,Return,Stack>::resume_inner
//!              at src/coroutine.rs:256:13
//!       corosensei::coroutine::ScopedCoroutine<Input,Yield,Return,Stack>::resume
//!              at src/coroutine.rs:235:19
//!       backtrace::sub_function
//!              at examples/backtrace.rs:6:5
//!    4: backtrace::main
//!              at examples/backtrace.rs:15:5
//! ```
//!
//! Linked backtraces use the platform's standard unwinding metadata and work with debuggers, profilers and many other tools.
//!
//! #### Cleanup on drop
//!
//! If a coroutine is dropped while it is suspended then the coroutine's stack will be safely unwound using the same mechanism as panics, which will drop any local variables on the stack. This is necessary to maintain a safe API: the coroutine's stack cannot be freed or reused while there are still live objects on it.
//!
//! If this cleanup is not desired then a coroutine can forcibly be marked as completed using the unsafe `force_reset` function.
//!
//! #### Trap handling
//!
//! For advanced use cases, this crate provides the ability to recover from traps (e.g. segmentation fault, stack overflow) that occur in a coroutine. This API is unsafe and should be used from within a signal/exception handler: it will help set up the return from the signal/exception handler so that the coroutine will exit immediately with the given return value and return control to its parent.
//!
//! This is particularly useful for running JIT-compiled code with a fixed stack limit: stack overflows can be caught by a signal/exception handler and easily recovered from.
//!
//! ## Cargo features
//!
//! This crate is compatible with `#![no_std]` when all Cargo features are disabled.
//!
//! The following Cargo feature flags are available on this crate:
//!
//! #### `default-stack` (Enabled by default)
//!
//! This feature provides a `DefaultStack` implementation which can be used by the `Coroutine` type. This stack is allocated with a guard page using OS APIs and requires `std`.
//!
//! If this feature is disabled then you must implement your own stack type which implements the `Stack` trait.
//!
//! #### `unwind` (Enabled by default)
//!
//! This feature adds support for:
//! - unwinding panics in a coroutine back out to its caller.
//! - forcibly unwinding a suspended coroutine via `force_unwind` or when the coroutine is dropped.
//!
//! Note that if a coroutine is dropped while suspended (i.e. it has been resumed at least once but has not returned yet) when this feature is disabled then the program will abort.
//!
//! Requires `std`.
//!
//! #### `asm-unwind`
//!
//! This feature uses the `asm_unwind` nightly feature to allow panics to unwind directly through the inline assembly used in this crate, which can improve performance since it doesn't need to be passed across stack boundaries as a `Result`.
//!
//! Implies `unwind`.

#![no_std]
#![cfg_attr(feature = "asm-unwind", feature(asm_unwind, c_unwind, asm_sym))]
#![warn(missing_docs)]

// Must come first because it defines macros used by other modules.
mod unwind;

mod arch;
mod coroutine;
pub mod stack;
pub mod trap;
mod util;

pub use coroutine::*;

#[cfg(test)]
mod tests;
