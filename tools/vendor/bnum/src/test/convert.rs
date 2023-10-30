use crate::cast::CastFrom;
use crate::test::types::*;
use crate::*;

pub trait TestConvert {
    type Output;

    fn into(self) -> Self::Output;
}

macro_rules! test_convert_big {
    ($($big: ty), *; $output: ty) => {
        $(
            impl TestConvert for $big {
                type Output = $output;

                #[inline]
                fn into(self) -> Self::Output {
                    Self::Output::cast_from(self)
                }
            }
        )*
    };
}

macro_rules! test_convert_bigints {
    ($($bits: literal), *) => {
        paste::paste! {
            $(
                test_convert_big!(BUint<{$bits / 64}>, BUintD32<{$bits / 32}>, BUintD16<{$bits / 16}>, BUintD8<{$bits / 8}>; [<u $bits>]);

                test_convert_big!(BInt<{$bits / 64}>, BIntD32<{$bits / 32}>, BIntD16<{$bits / 16}>, BIntD8<{$bits / 8}>; [<i $bits>]);
            )*
        }
    };
}

test_convert_bigints!(128, 64);

test_convert_big!(BUintD32<{32 / 32}>, BUintD16<{32 / 16}>, BUintD8<{32 / 8}>; u32);
test_convert_big!(BIntD32<{32 / 32}>, BIntD16<{32 / 16}>, BIntD8<{32 / 8}>; i32);

impl<T: TestConvert> TestConvert for Option<T> {
    type Output = Option<<T as TestConvert>::Output>;

    #[inline]
    fn into(self) -> Self::Output {
        self.map(TestConvert::into)
    }
}

impl TestConvert for f64 {
    type Output = u64;

    #[inline]
    fn into(self) -> Self::Output {
        self.to_bits()
    }
}

impl TestConvert for f32 {
    type Output = u32;

    #[inline]
    fn into(self) -> Self::Output {
        self.to_bits()
    }
}

/*#[cfg(feature = "nightly")]
impl TestConvert for crate::float::F64 {
    type Output = u64;

    #[inline]
    fn into(self) -> Self::Output {
        use crate::cast::As;

        self.to_bits().as_()
    }
}

#[cfg(feature = "nightly")]
impl TestConvert for crate::float::F32 {
    type Output = u32;

    #[inline]
    fn into(self) -> Self::Output {
        use crate::cast::As;

        self.to_bits().as_()
    }
}*/

impl<T: TestConvert, U: TestConvert> TestConvert for (T, U) {
    type Output = (<T as TestConvert>::Output, <U as TestConvert>::Output);

    #[inline]
    fn into(self) -> Self::Output {
        (TestConvert::into(self.0), TestConvert::into(self.1))
    }
}

impl<T, const N: usize> TestConvert for [T; N] {
    type Output = Self;

    #[inline]
    fn into(self) -> Self::Output {
        self
    }
}

impl TestConvert for crate::errors::ParseIntError {
    type Output = core::num::IntErrorKind;

    #[inline]
    fn into(self) -> Self::Output {
        self.kind().clone()
    }
}

impl TestConvert for core::num::ParseIntError {
    type Output = core::num::IntErrorKind;

    #[inline]
    fn into(self) -> Self::Output {
        self.kind().clone()
    }
}

impl<T: TestConvert, E: TestConvert> TestConvert for Result<T, E> {
    type Output = Result<<T as TestConvert>::Output, <E as TestConvert>::Output>;

    #[inline]
    fn into(self) -> Self::Output {
        match self {
            Ok(val) => Ok(TestConvert::into(val)),
            Err(err) => Err(TestConvert::into(err)),
        }
    }
}

impl TestConvert for core::num::TryFromIntError {
    type Output = ();

    #[inline]
    fn into(self) -> Self::Output {
        ()
    }
}

impl TestConvert for crate::errors::TryFromIntError {
    type Output = ();

    #[inline]
    fn into(self) -> Self::Output {
        ()
    }
}

impl TestConvert for core::convert::Infallible {
    type Output = ();

    #[inline]
    fn into(self) -> Self::Output {
        ()
    }
}

macro_rules! test_convert_to_self {
    ($($ty: ty), *) => {
        $(
            impl TestConvert for $ty {
                type Output = Self;

                #[inline]
                fn into(self) -> Self::Output {
                    self
                }
            }
        )*
    };
}

test_convert_to_self!(
    core::num::FpCategory,
    core::cmp::Ordering,
    bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    alloc::string::String
);
