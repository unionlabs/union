use pin_project::{pin_project, UnsafeUnpin};

#[pin_project(UnsafeUnpin)]
struct TupleStruct<T, U>(#[pin] T, U);

unsafe impl<T: Unpin, U> UnsafeUnpin for Struct<T, U> {}

fn main() {}
