use pin_project::pin_project;
#[pin(__private(!Unpin))]
struct TupleStruct<T, U>(#[pin] T, U);
#[allow(box_pointers)]
#[allow(deprecated)]
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(unreachable_pub)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::pattern_type_mismatch)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::type_repetition_in_bounds)]
#[allow(unused_qualifications)]
#[allow(clippy::semicolon_if_nothing_returned)]
#[allow(clippy::use_self)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    #[allow(unused_extern_crates)]
    extern crate pin_project as _pin_project;
    #[allow(dead_code)]
    #[allow(clippy::mut_mut)]
    struct __TupleStructProjection<'pin, T, U>(
        ::pin_project::__private::Pin<&'pin mut (T)>,
        &'pin mut (U),
    )
    where
        TupleStruct<T, U>: 'pin;
    #[allow(dead_code)]
    #[allow(clippy::ref_option_ref)]
    struct __TupleStructProjectionRef<'pin, T, U>(
        ::pin_project::__private::Pin<&'pin (T)>,
        &'pin (U),
    )
    where
        TupleStruct<T, U>: 'pin;
    impl<T, U> TupleStruct<T, U> {
        #[allow(dead_code)]
        fn project<'pin>(
            self: _pin_project::__private::Pin<&'pin mut Self>,
        ) -> __TupleStructProjection<'pin, T, U> {
            unsafe {
                let Self(_0, _1) = self.get_unchecked_mut();
                __TupleStructProjection(
                    _pin_project::__private::Pin::new_unchecked(_0),
                    _1,
                )
            }
        }
        #[allow(dead_code)]
        #[allow(clippy::missing_const_for_fn)]
        fn project_ref<'pin>(
            self: _pin_project::__private::Pin<&'pin Self>,
        ) -> __TupleStructProjectionRef<'pin, T, U> {
            unsafe {
                let Self(_0, _1) = self.get_ref();
                __TupleStructProjectionRef(
                    _pin_project::__private::Pin::new_unchecked(_0),
                    _1,
                )
            }
        }
    }
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &TupleStruct<T, U>) {
        let _ = &this.0;
        let _ = &this.1;
    }
    impl<'pin, T, U> _pin_project::__private::Unpin for TupleStruct<T, U>
    where
        _pin_project::__private::Wrapper<
            'pin,
            _pin_project::__private::PhantomPinned,
        >: _pin_project::__private::Unpin,
    {}
    #[doc(hidden)]
    unsafe impl<'pin, T, U> _pin_project::UnsafeUnpin for TupleStruct<T, U>
    where
        _pin_project::__private::Wrapper<
            'pin,
            _pin_project::__private::PhantomPinned,
        >: _pin_project::__private::Unpin,
    {}
    trait TupleStructMustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: _pin_project::__private::Drop> TupleStructMustNotImplDrop for T {}
    impl<T, U> TupleStructMustNotImplDrop for TupleStruct<T, U> {}
    #[doc(hidden)]
    impl<T, U> _pin_project::__private::PinnedDrop for TupleStruct<T, U> {
        unsafe fn drop(self: _pin_project::__private::Pin<&mut Self>) {}
    }
};
fn main() {}
