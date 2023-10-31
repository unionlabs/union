use pin_project_lite::pin_project;
enum Enum<T, U> {
    Struct {
        pinned1: T,
        pinned2: T,
        unpinned1: U,
        unpinned2: U,
    },
    Unit,
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::type_repetition_in_bounds)]
enum EnumProjReplace<T, U> {
    Struct {
        pinned1: ::pin_project_lite::__private::PhantomData<T>,
        pinned2: ::pin_project_lite::__private::PhantomData<T>,
        unpinned1: U,
        unpinned2: U,
    },
    Unit,
}
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> Enum<T, U> {
        fn project_replace(
            self: ::pin_project_lite::__private::Pin<&mut Self>,
            replacement: Self,
        ) -> EnumProjReplace<T, U> {
            unsafe {
                let __self_ptr: *mut Self = self.get_unchecked_mut();
                let __guard = ::pin_project_lite::__private::UnsafeOverwriteGuard::new(
                    __self_ptr,
                    replacement,
                );
                match &mut *__self_ptr {
                    Self::Struct {
                        pinned1,
                        pinned2,
                        unpinned1,
                        unpinned2,
                    } => {
                        let result = EnumProjReplace::Struct {
                            pinned1: ::pin_project_lite::__private::PhantomData,
                            pinned2: ::pin_project_lite::__private::PhantomData,
                            unpinned1: ::pin_project_lite::__private::ptr::read(unpinned1),
                            unpinned2: ::pin_project_lite::__private::ptr::read(unpinned2),
                        };
                        {
                            (
                                ::pin_project_lite::__private::UnsafeDropInPlaceGuard::new(pinned1),
                                ::pin_project_lite::__private::UnsafeDropInPlaceGuard::new(pinned2),
                                (),
                                (),
                            );
                        }
                        result
                    }
                    Self::Unit => EnumProjReplace::Unit,
                }
            }
        }
    }
    #[allow(non_snake_case)]
    struct __Origin<'__pin, T, U> {
        __dummy_lifetime: ::pin_project_lite::__private::PhantomData<&'__pin ()>,
        Struct: (
            T,
            T,
            ::pin_project_lite::__private::AlwaysUnpin<U>,
            ::pin_project_lite::__private::AlwaysUnpin<U>,
        ),
        Unit: (),
    }
    impl<'__pin, T, U> ::pin_project_lite::__private::Unpin for Enum<T, U> where
        __Origin<'__pin, T, U>: ::pin_project_lite::__private::Unpin
    {
    }
    trait MustNotImplDrop {}
    #[allow(clippy::drop_bounds, drop_bounds)]
    impl<T: ::pin_project_lite::__private::Drop> MustNotImplDrop for T {}
    impl<T, U> MustNotImplDrop for Enum<T, U> {}
};
fn main() {}
