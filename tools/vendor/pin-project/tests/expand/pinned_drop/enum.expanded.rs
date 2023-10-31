use std::pin::Pin;
use pin_project::{pin_project, pinned_drop};
#[pin(__private(PinnedDrop, project = EnumProj, project_ref = EnumProjRef))]
enum Enum<T, U> {
    Struct { #[pin] pinned: T, unpinned: U },
    Tuple(#[pin] T, U),
    Unit,
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
#[allow(dead_code)]
#[allow(clippy::mut_mut)]
enum EnumProj<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Struct {
        pinned: ::pin_project::__private::Pin<&'pin mut (T)>,
        unpinned: &'pin mut (U),
    },
    Tuple(::pin_project::__private::Pin<&'pin mut (T)>, &'pin mut (U)),
    Unit,
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
#[allow(dead_code)]
#[allow(clippy::ref_option_ref)]
enum EnumProjRef<'pin, T, U>
where
    Enum<T, U>: 'pin,
{
    Struct { pinned: ::pin_project::__private::Pin<&'pin (T)>, unpinned: &'pin (U) },
    Tuple(::pin_project::__private::Pin<&'pin (T)>, &'pin (U)),
    Unit,
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
    impl<T, U> Enum<T, U> {
        #[allow(dead_code)]
        fn project<'pin>(
            self: _pin_project::__private::Pin<&'pin mut Self>,
        ) -> EnumProj<'pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Self::Struct { pinned, unpinned } => {
                        EnumProj::Struct {
                            pinned: _pin_project::__private::Pin::new_unchecked(pinned),
                            unpinned,
                        }
                    }
                    Self::Tuple(_0, _1) => {
                        EnumProj::Tuple(
                            _pin_project::__private::Pin::new_unchecked(_0),
                            _1,
                        )
                    }
                    Self::Unit => EnumProj::Unit,
                }
            }
        }
        #[allow(dead_code)]
        #[allow(clippy::missing_const_for_fn)]
        fn project_ref<'pin>(
            self: _pin_project::__private::Pin<&'pin Self>,
        ) -> EnumProjRef<'pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Self::Struct { pinned, unpinned } => {
                        EnumProjRef::Struct {
                            pinned: _pin_project::__private::Pin::new_unchecked(pinned),
                            unpinned,
                        }
                    }
                    Self::Tuple(_0, _1) => {
                        EnumProjRef::Tuple(
                            _pin_project::__private::Pin::new_unchecked(_0),
                            _1,
                        )
                    }
                    Self::Unit => EnumProjRef::Unit,
                }
            }
        }
    }
    #[allow(missing_debug_implementations)]
    struct __Enum<'pin, T, U> {
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
    impl<'pin, T, U> _pin_project::__private::Unpin for Enum<T, U>
    where
        __Enum<'pin, T, U>: _pin_project::__private::Unpin,
    {}
    #[doc(hidden)]
    unsafe impl<'pin, T, U> _pin_project::UnsafeUnpin for Enum<T, U>
    where
        __Enum<'pin, T, U>: _pin_project::__private::Unpin,
    {}
    impl<T, U> _pin_project::__private::Drop for Enum<T, U> {
        fn drop(&mut self) {
            unsafe {
                let __pinned_self = _pin_project::__private::Pin::new_unchecked(self);
                _pin_project::__private::PinnedDrop::drop(__pinned_self);
            }
        }
    }
};
#[doc(hidden)]
impl<T, U> ::pin_project::__private::PinnedDrop for Enum<T, U> {
    unsafe fn drop(self: Pin<&mut Self>) {
        #[allow(clippy::needless_pass_by_value)]
        fn __drop_inner<T, U>(__self: Pin<&mut Enum<T, U>>) {
            fn __drop_inner() {}
            let _ = __self;
        }
        __drop_inner(self);
    }
}
fn main() {}
