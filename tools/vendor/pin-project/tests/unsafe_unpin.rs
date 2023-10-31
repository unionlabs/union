#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

#[macro_use]
mod auxiliary;

use std::marker::PhantomPinned;

use pin_project::{pin_project, UnsafeUnpin};

#[pin_project(UnsafeUnpin)]
pub struct Blah<T, U> {
    f1: U,
    #[pin]
    f2: T,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Blah<T, U> {}

assert_unpin!(Blah<(), ()>);
assert_unpin!(Blah<(), PhantomPinned>);
assert_not_unpin!(Blah<PhantomPinned, ()>);
assert_not_unpin!(Blah<PhantomPinned, PhantomPinned>);

#[pin_project(UnsafeUnpin)]
struct OverlappingLifetimeNames<'pin, T, U> {
    #[pin]
    f1: U,
    #[pin]
    f2: Option<T>,
    f3: &'pin (),
}

unsafe impl<T: Unpin, U: Unpin> UnsafeUnpin for OverlappingLifetimeNames<'_, T, U> {}

assert_unpin!(OverlappingLifetimeNames<'_, (), ()>);
assert_not_unpin!(OverlappingLifetimeNames<'_, PhantomPinned, ()>);
assert_not_unpin!(OverlappingLifetimeNames<'_, (), PhantomPinned>);
assert_not_unpin!(OverlappingLifetimeNames<'_, PhantomPinned, PhantomPinned>);

#[test]
fn trivial_bounds() {
    #[pin_project(UnsafeUnpin)]
    pub struct NotImplementUnsafUnpin {
        #[pin]
        f: PhantomPinned,
    }

    assert_not_unpin!(NotImplementUnsafUnpin);
}
