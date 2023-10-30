use pin_project::pin_project;
#[pin(__private(!Unpin))]
struct Struct<T, U> {
    #[pin]
    pinned: T,
    unpinned: U,
}
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
    struct __StructProjection<'pin, T, U>
    where
        Struct<T, U>: 'pin,
    {
        pinned: ::pin_project::__private::Pin<&'pin mut (T)>,
        unpinned: &'pin mut (U),
    }
    #[allow(dead_code)]
    #[allow(clippy::ref_option_ref)]
    struct __StructProjectionRef<'pin, T, U>
    where
        Struct<T, U>: 'pin,
    {
        pinned: ::pin_project::__private::Pin<&'pin (T)>,
        unpinned: &'pin (U),
    }
    impl<T, U> Struct<T, U> {
        #[allow(dead_code)]
        fn project<'pin>(
            self: _pin_project::__private::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                __StructProjection {
                    pinned: _pin_project::__private::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
        #[allow(dead_code)]
        #[allow(clippy::missing_const_for_fn)]
        fn project_ref<'pin>(
            self: _pin_project::__private::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                __StructProjectionRef {
                    pinned: _pin_project::__private::Pin::new_unchecked(pinned),
                    unpinned,
                }
            }
        }
    }
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &Struct<T, U>) {
        let _ = &this.pinned;
        let _ = &this.unpinned;
    }
    impl<'pin, T, U> _pin_project::__private::Unpin for Struct<T, U>
    where
        _pin_project::__private::Wrapper<
            'pin,
            _pin_project::__private::PhantomPinned,
        >: _pin_project::__private::Unpin,
    {}
    #[doc(hidden)]
    unsafe impl<'pin, T, U> _pin_project::UnsafeUnpin for Struct<T, U>
    where
        _pin_project::__private::Wrapper<
            'pin,
            _pin_project::__private::PhantomPinned,
        >: _pin_project::__private::Unpin,
    {}
    trait StructMustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: _pin_project::__private::Drop> StructMustNotImplDrop for T {}
    impl<T, U> StructMustNotImplDrop for Struct<T, U> {}
    #[doc(hidden)]
    impl<T, U> _pin_project::__private::PinnedDrop for Struct<T, U> {
        unsafe fn drop(self: _pin_project::__private::Pin<&mut Self>) {}
    }
};
fn main() {}
