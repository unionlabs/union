// See ./unsafe_unpin-expanded.rs for generated code.

#![allow(dead_code)]

use pin_project::{pin_project, UnsafeUnpin};

#[pin_project(UnsafeUnpin)]
pub struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}

unsafe impl<T: Unpin, U> UnsafeUnpin for Struct<T, U> {}

fn main() {}
