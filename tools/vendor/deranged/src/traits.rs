use crate::{
    RangedI128, RangedI16, RangedI32, RangedI64, RangedI8, RangedIsize, RangedU128, RangedU16,
    RangedU32, RangedU64, RangedU8, RangedUsize,
};

macro_rules! declare_traits {
    ($($trait_name:ident),* $(,)?) => {$(
        pub(crate) trait $trait_name {
            const ASSERT: ();
        }
    )*};
}

macro_rules! impl_traits_for_all {
    ($($ranged_ty:ident $inner_ty:ident),* $(,)?) => {$(
        impl<const MIN: $inner_ty, const MAX: $inner_ty> RangeIsValid for $ranged_ty<MIN, MAX> {
            const ASSERT: () = assert!(MIN <= MAX);
        }

        impl<
            const CURRENT_MIN: $inner_ty,
            const CURRENT_MAX: $inner_ty,
            const NEW_MIN: $inner_ty,
            const NEW_MAX: $inner_ty,
        > ExpandIsValid for ($ranged_ty<CURRENT_MIN, CURRENT_MAX>, $ranged_ty<NEW_MIN, NEW_MAX>) {
            const ASSERT: () = {
                assert!(NEW_MIN <= CURRENT_MIN);
                assert!(NEW_MAX >= CURRENT_MAX);
            };
        }

        impl<
            const CURRENT_MIN: $inner_ty,
            const CURRENT_MAX: $inner_ty,
            const NEW_MIN: $inner_ty,
            const NEW_MAX: $inner_ty,
        > NarrowIsValid for ($ranged_ty<CURRENT_MIN, CURRENT_MAX>, $ranged_ty<NEW_MIN, NEW_MAX>) {
            const ASSERT: () = {
                assert!(NEW_MIN >= CURRENT_MIN);
                assert!(NEW_MAX <= CURRENT_MAX);
            };
        }

        impl<
            const VALUE: $inner_ty,
            const MIN: $inner_ty,
            const MAX: $inner_ty,
        > StaticIsValid for ($ranged_ty<MIN, VALUE>, $ranged_ty<VALUE, MAX>) {
            const ASSERT: () = {
                assert!(VALUE >= MIN);
                assert!(VALUE <= MAX);
            };
        }
    )*};
}

macro_rules! impl_traits_for_signed {
    ($($ranged_ty:ident $inner_ty:ident),* $(,)?) => {$(
        impl<const MIN: $inner_ty, const MAX: $inner_ty> AbsIsSafe for $ranged_ty<MIN, MAX> {
            const ASSERT: () = {
                assert!(MIN != <$inner_ty>::MIN);
                assert!(-MIN <= MAX);
            };
        }

        impl<const MIN: $inner_ty, const MAX: $inner_ty> NegIsSafe for $ranged_ty<MIN, MAX> {
            const ASSERT: () = {
                assert!(MIN != <$inner_ty>::MIN);
                assert!(-MIN <= MAX);
                assert!(-MAX >= MIN);
            };
        }

        impl_traits_for_all!($ranged_ty $inner_ty);
    )*};
}

macro_rules! impl_traits_for_unsigned {
    ($($ranged_ty:ident $inner_ty:ident),* $(,)?) => {$(
        impl<const MIN: $inner_ty, const MAX: $inner_ty> AbsIsSafe for $ranged_ty<MIN, MAX> {
            const ASSERT: () = ();
        }

        impl<const MIN: $inner_ty, const MAX: $inner_ty> NegIsSafe for $ranged_ty<MIN, MAX> {
            const ASSERT: () = assert!(MAX == 0);
        }

        impl_traits_for_all!($ranged_ty $inner_ty);
    )*};
}

declare_traits![
    RangeIsValid,
    AbsIsSafe,
    NegIsSafe,
    ExpandIsValid,
    NarrowIsValid,
    StaticIsValid,
];

impl_traits_for_signed! {
    RangedI8 i8,
    RangedI16 i16,
    RangedI32 i32,
    RangedI64 i64,
    RangedI128 i128,
    RangedIsize isize,
}

impl_traits_for_unsigned! {
    RangedU8 u8,
    RangedU16 u16,
    RangedU32 u32,
    RangedU64 u64,
    RangedU128 u128,
    RangedUsize usize,
}
