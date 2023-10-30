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
struct StructProjRef<'__pin, T, U>
where
    Struct<T, U>: '__pin,
{
    pinned: ::pin_project_lite::__private::Pin<&'__pin (T)>,
    unpinned: &'__pin (U),
}
#[allow(explicit_outlives_requirements)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    #[allow(dead_code)]
    #[allow(single_use_lifetimes)]
    #[allow(clippy::unknown_clippy_lints)]
    #[allow(clippy::mut_mut)]
    #[allow(clippy::redundant_pub_crate)]
    #[allow(clippy::ref_option_ref)]
    #[allow(clippy::type_repetition_in_bounds)]
    struct Projection<'__pin, T, U>
    where
        Struct<T, U>: '__pin,
    {
        pinned: ::pin_project_lite::__private::Pin<&'__pin mut (T)>,
        unpinned: &'__pin mut (U),
    }
    impl<T, U> Struct<T, U> {
        fn project<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin mut Self>,
        ) -> Projection<'__pin, T, U> {
            unsafe {
                let Self { pinned, unpinned } = self.get_unchecked_mut();
                Projection {
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
