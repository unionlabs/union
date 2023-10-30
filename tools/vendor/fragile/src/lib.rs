//! This library provides wrapper types that permit sending non `Send` types to
//! other threads and use runtime checks to ensure safety.
//!
//! It provides three types: [`Fragile`] and [`Sticky`] which are similar in nature
//! but have different behaviors with regards to how destructors are executed and
//! the extra [`SemiSticky`] type which uses [`Sticky`] if the value has a
//! destructor and [`Fragile`] if it does not.
//!
//! All three types wrap a value and provide a `Send` bound.  Neither of the types permit
//! access to the enclosed value unless the thread that wrapped the value is attempting
//! to access it.  The difference between the types starts playing a role once
//! destructors are involved.
//!
//! A [`Fragile`] will actually send the `T` from thread to thread but will only
//! permit the original thread to invoke the destructor.  If the value gets dropped
//! in a different thread, the destructor will panic.
//!
//! A [`Sticky`] on the other hand does not actually send the `T` around but keeps
//! it stored in the original thread's thread local storage.  If it gets dropped
//! in the originating thread it gets cleaned up immediately, otherwise it leaks
//! until the thread shuts down naturally.  [`Sticky`] because it borrows into the
//! TLS also requires you to "prove" that you are not doing any funny business with
//! the borrowed value that lives for longer than the current stack frame which
//! results in a slightly more complex API.
//!
//! There is a third typed called [`SemiSticky`] which shares the API with [`Sticky`]
//! but internally uses a boxed [`Fragile`] if the type does not actually need a dtor
//! in which case [`Fragile`] is preferred.
//!
//! # Fragile Usage
//!
//! [`Fragile`] is the easiest type to use.  It works almost like a cell.
//!
//! ```
//! use std::thread;
//! use fragile::Fragile;
//!
//! // creating and using a fragile object in the same thread works
//! let val = Fragile::new(true);
//! assert_eq!(*val.get(), true);
//! assert!(val.try_get().is_ok());
//!
//! // once send to another thread it stops working
//! thread::spawn(move || {
//!     assert!(val.try_get().is_err());
//! }).join()
//!     .unwrap();
//! ```
//!
//! # Sticky Usage
//!
//! [`Sticky`] is similar to [`Fragile`] but because it places the value in the
//! thread local storage it comes with some extra restrictions to make it sound.
//! The advantage is it can be dropped from any thread but it comes with extra
//! restrictions.  In particular it requires that values placed in it are `'static`
//! and that [`StackToken`]s are used to restrict lifetimes.
//!
//! ```
//! use std::thread;
//! use fragile::Sticky;
//!
//! // creating and using a fragile object in the same thread works
//! fragile::stack_token!(tok);
//! let val = Sticky::new(true);
//! assert_eq!(*val.get(tok), true);
//! assert!(val.try_get(tok).is_ok());
//!
//! // once send to another thread it stops working
//! thread::spawn(move || {
//!     fragile::stack_token!(tok);
//!     assert!(val.try_get(tok).is_err());
//! }).join()
//!     .unwrap();
//! ```
//!
//! # Why?
//!
//! Most of the time trying to use this crate is going to indicate some code smell.  But
//! there are situations where this is useful.  For instance you might have a bunch of
//! non `Send` types but want to work with a `Send` error type.  In that case the non
//! sendable extra information can be contained within the error and in cases where the
//! error did not cross a thread boundary yet extra information can be obtained.
//!
//! # Drop / Cleanup Behavior
//!
//! All types will try to eagerly drop a value if they are dropped on the right thread.
//! [`Sticky`] and [`SemiSticky`] will however temporarily leak memory until a thread
//! shuts down if the value is dropped on the wrong thread.  The benefit however is that
//! if you have that type of situation, and you can live with the consequences, the
//! type is not panicking.  A [`Fragile`] dropped in the wrong thread will not just panic,
//! it will effectively also tear down the process because panicking in destructors is
//! non recoverable.
//!
//! # Features
//!
//! By default the crate has no dependencies.  Optionally the `slab` feature can
//! be enabled which optimizes the internal storage of the [`Sticky`] type to
//! make it use a [`slab`](https://docs.rs/slab/latest/slab/) instead.
mod errors;
mod fragile;
mod registry;
mod semisticky;
mod sticky;
mod thread_id;

use std::marker::PhantomData;

pub use crate::errors::InvalidThreadAccess;
pub use crate::fragile::Fragile;
pub use crate::semisticky::SemiSticky;
pub use crate::sticky::Sticky;

/// A token that is placed to the stack to constrain lifetimes.
///
/// For more information about how these work see the documentation of
/// [`stack_token!`] which is the only way to create this token.
pub struct StackToken(PhantomData<*const ()>);

impl StackToken {
    /// Stack tokens must only be created on the stack.
    #[doc(hidden)]
    pub unsafe fn __private_new() -> StackToken {
        // we place a const pointer in there to get a type
        // that is neither Send nor Sync.
        StackToken(PhantomData)
    }
}

/// Crates a token on the stack with a certain name for semi-sticky.
///
/// The argument to the macro is the target name of a local variable
/// which holds a reference to a stack token.  Because this is the
/// only way to create such a token, it acts as a proof to [`Sticky`]
/// or [`SemiSticky`] that can be used to constrain the lifetime of the
/// return values to the stack frame.
///
/// This is necessary as otherwise a [`Sticky`] placed in a [`Box`] and
/// leaked with [`Box::leak`] (which creates a static lifetime) would
/// otherwise create a reference with `'static` lifetime.  This is incorrect
/// as the actual lifetime is constrained to the lifetime of the thread.
/// For more information see [`issue 26`](https://github.com/mitsuhiko/fragile/issues/26).
///
/// ```rust
/// let sticky = fragile::Sticky::new(true);
///
/// // this places a token on the stack.
/// fragile::stack_token!(my_token);
///
/// // the token needs to be passed to `get` and others.
/// let _ = sticky.get(my_token);
/// ```
#[macro_export]
macro_rules! stack_token {
    ($name:ident) => {
        let $name = &unsafe { $crate::StackToken::__private_new() };
    };
}
