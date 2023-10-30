#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

#[macro_use]
mod auxiliary;

pub mod default {
    use std::marker::PhantomPinned;

    use pin_project_lite::pin_project;

    struct Inner<T> {
        f: T,
    }

    assert_unpin!(Inner<()>);
    assert_not_unpin!(Inner<PhantomPinned>);

    pin_project! {
        struct Struct<T, U> {
            #[pin]
            f1: Inner<T>,
            f2: U,
        }
    }

    assert_unpin!(Struct<(), ()>);
    assert_unpin!(Struct<(), PhantomPinned>);
    assert_not_unpin!(Struct<PhantomPinned, ()>);
    assert_not_unpin!(Struct<PhantomPinned, PhantomPinned>);

    pin_project! {
        #[project = EnumProj]
        #[project_ref = EnumProjRef]
        enum Enum<T, U> {
            V1 {
                #[pin]
                f1: Inner<T>,
                f2: U,
            },
        }
    }

    assert_unpin!(Enum<(), ()>);
    assert_unpin!(Enum<(), PhantomPinned>);
    assert_not_unpin!(Enum<PhantomPinned, ()>);
    assert_not_unpin!(Enum<PhantomPinned, PhantomPinned>);

    pin_project! {
        struct TrivialBounds {
            #[pin]
            f: PhantomPinned,
        }
    }

    assert_not_unpin!(TrivialBounds);

    pin_project! {
        struct PinRef<'a, T, U> {
            #[pin]
            f1: &'a mut Inner<T>,
            f2: U,
        }
    }

    assert_unpin!(PinRef<'_, PhantomPinned, PhantomPinned>);
}
