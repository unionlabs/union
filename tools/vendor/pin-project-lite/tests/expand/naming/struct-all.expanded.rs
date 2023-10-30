use pin_project_lite::pin_project;
struct Struct<T, U> {
    pinned: T,
    unpinned: U,
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::ref_option_ref)]
#[allow(clippy::type_repetition_in_bounds)]
struct StructProj<'__pin, T, U>
where
    Struct<T, U>: '__pin,
{
    pinned: ::pin_project_lite::__private::Pin<&'__pin mut (T)>,
    unpinned: &'__pin mut (U),
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::ref_option_ref)]
#[allow(clippy::type_repetition_in_bounds)]
struct StructProjRef<'__pin, T, U>
where
    Struct<T, U>: '__pin,
{
    pinned: ::pin_project_lite::__private::Pin<&'__pin (T)>,
    unpinned: &'__pin (U),
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::type_repetition_in_bounds)]
struct StructProjReplace<T, U> {
    pinned: ::pin_project_lite::__private::PhantomData<T>,
    unpinned: U,
}
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> Struct<T, U> {
        fn project<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin mut Self>,
        ) -> StructProj<'__pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                StructProj {
                    pinned: ::pin_project_lite::__private::Pin::new_unchecked(pinned),
                    unpinned: unpinned,
                }
            }
        }
        fn project_ref<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin Self>,
        ) -> StructProjRef<'__pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_ref();
                StructProjRef {
                    pinned: ::pin_project_lite::__private::Pin::new_unchecked(pinned),
                    unpinned: unpinned,
                }
            }
        }
        fn project_replace(
            self: ::pin_project_lite::__private::Pin<&mut Self>,
            replacement: Self,
        ) -> StructProjReplace<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let __guard = ::pin_project_lite::__private::UnsafeOverwriteGuard::new(
                    __self_ptr,
                    replacement,
                );
                let Self { pinned, unpinned } = &mut *__self_ptr;
                let result = StructProjReplace {
                    pinned: ::pin_project_lite::__private::PhantomData,
                    unpinned: ::pin_project_lite::__private::ptr::read(unpinned),
                };
                {
                    (
                        ::pin_project_lite::__private::UnsafeDropInPlaceGuard::new(pinned),
                        (),
                    );
                }
                result
            }
        }
    }
    #[allow(non_snake_case)]
    struct __Origin<'__pin, T, U> {
        __dummy_lifetime: ::pin_project_lite::__private::PhantomData<&'__pin ()>,
        pinned: T,
        unpinned: ::pin_project_lite::__private::AlwaysUnpin<U>,
    }
    impl<'__pin, T, U> ::pin_project_lite::__private::Unpin for Struct<T, U> where
        __Origin<'__pin, T, U>: ::pin_project_lite::__private::Unpin
    {
    }
    trait MustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project_lite::__private::Drop> MustNotImplDrop for T {}
    impl<T, U> MustNotImplDrop for Struct<T, U> {}
    #[forbid(unaligned_references, safe_packed_borrows)]
    fn __assert_not_repr_packed<T, U>(this: &Struct<T, U>) {
        let _ = &this.pinned;
        let _ = &this.unpinned;
    }
};
fn main() {}
