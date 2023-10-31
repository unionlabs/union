use pin_project_lite::pin_project;
use std::pin::Pin;
enum Enum<T, U> {
    Struct { pinned: T, unpinned: U },
    Unit,
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::ref_option_ref)]
#[allow(clippy::type_repetition_in_bounds)]
enum EnumProj<'__pin, T, U>
where
    Enum<T, U>: '__pin,
{
    Struct {
        pinned: ::pin_project_lite::__private::Pin<&'__pin mut (T)>,
        unpinned: &'__pin mut (U),
    },
    Unit,
}
#[allow(dead_code)]
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::mut_mut)]
#[allow(clippy::redundant_pub_crate)]
#[allow(clippy::ref_option_ref)]
#[allow(clippy::type_repetition_in_bounds)]
enum EnumProjRef<'__pin, T, U>
where
    Enum<T, U>: '__pin,
{
    Struct {
        pinned: ::pin_project_lite::__private::Pin<&'__pin (T)>,
        unpinned: &'__pin (U),
    },
    Unit,
}
#[allow(single_use_lifetimes)]
#[allow(clippy::unknown_clippy_lints)]
#[allow(clippy::used_underscore_binding)]
const _: () = {
    impl<T, U> Enum<T, U> {
        fn project<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin mut Self>,
        ) -> EnumProj<'__pin, T, U> {
            unsafe {
                match self.get_unchecked_mut() {
                    Self::Struct { pinned, unpinned } => EnumProj::Struct {
                        pinned: ::pin_project_lite::__private::Pin::new_unchecked(pinned),
                        unpinned: unpinned,
                    },
                    Self::Unit => EnumProj::Unit,
                }
            }
        }
        fn project_ref<'__pin>(
            self: ::pin_project_lite::__private::Pin<&'__pin Self>,
        ) -> EnumProjRef<'__pin, T, U> {
            unsafe {
                match self.get_ref() {
                    Self::Struct { pinned, unpinned } => EnumProjRef::Struct {
                        pinned: ::pin_project_lite::__private::Pin::new_unchecked(pinned),
                        unpinned: unpinned,
                    },
                    Self::Unit => EnumProjRef::Unit,
                }
            }
        }
    }
    #[allow(non_snake_case)]
    struct __Origin<'__pin, T, U> {
        __dummy_lifetime: ::pin_project_lite::__private::PhantomData<&'__pin ()>,
        Struct: (T, ::pin_project_lite::__private::AlwaysUnpin<U>),
        Unit: (),
    }
    impl<'__pin, T, U> ::pin_project_lite::__private::Unpin for Enum<T, U> where
        __Origin<'__pin, T, U>: ::pin_project_lite::__private::Unpin
    {
    }
    impl<T, U> ::pin_project_lite::__private::Drop for Enum<T, U> {
        fn drop(&mut self) {
            fn __drop_inner<T, U>(this: ::pin_project_lite::__private::Pin<&mut Enum<T, U>>) {
                fn __drop_inner() {}
                let _ = this;
            }
            let pinned_self: ::pin_project_lite::__private::Pin<&mut Self> =
                unsafe { ::pin_project_lite::__private::Pin::new_unchecked(self) };
            __drop_inner(pinned_self);
        }
    }
};
fn main() {}
