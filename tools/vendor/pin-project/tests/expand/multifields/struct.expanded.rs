use pin_project::pin_project;
#[pin(__private(project_replace))]
struct Struct<T, U> {
    #[pin]
    pinned1: T,
    #[pin]
    pinned2: T,
    unpinned1: U,
    unpinned2: U,
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
        pinned1: ::pin_project::__private::Pin<&'pin mut (T)>,
        pinned2: ::pin_project::__private::Pin<&'pin mut (T)>,
        unpinned1: &'pin mut (U),
        unpinned2: &'pin mut (U),
    }
    #[allow(dead_code)]
    #[allow(clippy::ref_option_ref)]
    struct __StructProjectionRef<'pin, T, U>
    where
        Struct<T, U>: 'pin,
    {
        pinned1: ::pin_project::__private::Pin<&'pin (T)>,
        pinned2: ::pin_project::__private::Pin<&'pin (T)>,
        unpinned1: &'pin (U),
        unpinned2: &'pin (U),
    }
    #[allow(dead_code)]
    struct __StructProjectionOwned<T, U> {
        pinned1: ::pin_project::__private::PhantomData<T>,
        pinned2: ::pin_project::__private::PhantomData<T>,
        unpinned1: U,
        unpinned2: U,
    }
    impl<T, U> Struct<T, U> {
        #[allow(dead_code)]
        fn project<'pin>(
            self: _pin_project::__private::Pin<&'pin mut Self>,
        ) -> __StructProjection<'pin, T, U> {
            unsafe {
                let Self { pinned1, pinned2, unpinned1, unpinned2 } = self
                    .get_unchecked_mut();
                __StructProjection {
                    pinned1: _pin_project::__private::Pin::new_unchecked(pinned1),
                    pinned2: _pin_project::__private::Pin::new_unchecked(pinned2),
                    unpinned1,
                    unpinned2,
                }
            }
        }
        #[allow(dead_code)]
        #[allow(clippy::missing_const_for_fn)]
        fn project_ref<'pin>(
            self: _pin_project::__private::Pin<&'pin Self>,
        ) -> __StructProjectionRef<'pin, T, U> {
            unsafe {
                let Self { pinned1, pinned2, unpinned1, unpinned2 } = self.get_ref();
                __StructProjectionRef {
                    pinned1: _pin_project::__private::Pin::new_unchecked(pinned1),
                    pinned2: _pin_project::__private::Pin::new_unchecked(pinned2),
                    unpinned1,
                    unpinned2,
                }
            }
        }
        #[allow(dead_code)]
        fn project_replace(
            self: _pin_project::__private::Pin<&mut Self>,
            __replacement: Self,
        ) -> __StructProjectionOwned<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let __guard = _pin_project::__private::UnsafeOverwriteGuard::new(
                    __self_ptr,
                    __replacement,
                );
                let Self { pinned1, pinned2, unpinned1, unpinned2 } = &mut *__self_ptr;
                let __result = __StructProjectionOwned {
                    pinned1: _pin_project::__private::PhantomData,
                    pinned2: _pin_project::__private::PhantomData,
                    unpinned1: _pin_project::__private::ptr::read(unpinned1),
                    unpinned2: _pin_project::__private::ptr::read(unpinned2),
                };
                {
                    let __guard = _pin_project::__private::UnsafeDropInPlaceGuard::new(
                        pinned2,
                    );
                    let __guard = _pin_project::__private::UnsafeDropInPlaceGuard::new(
                        pinned1,
                    );
                }
                __result
            }
        }
    }
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &Struct<T, U>) {
        let _ = &this.pinned1;
        let _ = &this.pinned2;
        let _ = &this.unpinned1;
        let _ = &this.unpinned2;
    }
    #[allow(missing_debug_implementations)]
    struct __Struct<'pin, T, U> {
        __pin_project_use_generics: _pin_project::__private::AlwaysUnpin<
            'pin,
            (
                _pin_project::__private::PhantomData<T>,
                _pin_project::__private::PhantomData<U>,
            ),
        >,
        __field0: T,
        __field1: T,
    }
    impl<'pin, T, U> _pin_project::__private::Unpin for Struct<T, U>
    where
        __Struct<'pin, T, U>: _pin_project::__private::Unpin,
    {}
    #[doc(hidden)]
    unsafe impl<'pin, T, U> _pin_project::UnsafeUnpin for Struct<T, U>
    where
        __Struct<'pin, T, U>: _pin_project::__private::Unpin,
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
