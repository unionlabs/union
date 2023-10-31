use core::ops::{Div, Rem};

pub trait Euclid: Sized + Div<Self, Output = Self> + Rem<Self, Output = Self> {
    /// Calculates Euclidean division, the matching method for `rem_euclid`.
    ///
    /// This computes the integer `n` such that
    /// `self = n * v + self.rem_euclid(v)`.
    /// In other words, the result is `self / v` rounded to the integer `n`
    /// such that `self >= n * v`.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::Euclid;
    ///
    /// let a: i32 = 7;
    /// let b: i32 = 4;
    /// assert_eq!(Euclid::div_euclid(&a, &b), 1); // 7 > 4 * 1
    /// assert_eq!(Euclid::div_euclid(&-a, &b), -2); // -7 >= 4 * -2
    /// assert_eq!(Euclid::div_euclid(&a, &-b), -1); // 7 >= -4 * -1
    /// assert_eq!(Euclid::div_euclid(&-a, &-b), 2); // -7 >= -4 * 2
    /// ```
    fn div_euclid(&self, v: &Self) -> Self;

    /// Calculates the least nonnegative remainder of `self (mod v)`.
    ///
    /// In particular, the return value `r` satisfies `0.0 <= r < v.abs()` in
    /// most cases. However, due to a floating point round-off error it can
    /// result in `r == v.abs()`, violating the mathematical definition, if
    /// `self` is much smaller than `v.abs()` in magnitude and `self < 0.0`.
    /// This result is not an element of the function's codomain, but it is the
    /// closest floating point number in the real numbers and thus fulfills the
    /// property `self == self.div_euclid(v) * v + self.rem_euclid(v)`
    /// approximatively.
    ///
    /// # Examples
    ///
    /// ```
    /// use num_traits::Euclid;
    ///
    /// let a: i32 = 7;
    /// let b: i32 = 4;
    /// assert_eq!(Euclid::rem_euclid(&a, &b), 3);
    /// assert_eq!(Euclid::rem_euclid(&-a, &b), 1);
    /// assert_eq!(Euclid::rem_euclid(&a, &-b), 3);
    /// assert_eq!(Euclid::rem_euclid(&-a, &-b), 1);
    /// ```
    fn rem_euclid(&self, v: &Self) -> Self;
}

macro_rules! euclid_forward_impl {
    ($($t:ty)*) => {$(
        #[cfg(has_div_euclid)]
        impl Euclid for $t {
            #[inline]
            fn div_euclid(&self, v: &$t) -> Self {
                <$t>::div_euclid(*self, *v)
            }

            #[inline]
            fn rem_euclid(&self, v: &$t) -> Self {
                <$t>::rem_euclid(*self, *v)
            }
        }
    )*}
}

macro_rules! euclid_int_impl {
    ($($t:ty)*) => {$(
        euclid_forward_impl!($t);

        #[cfg(not(has_div_euclid))]
        impl Euclid for $t {
            #[inline]
            fn div_euclid(&self, v: &$t) -> Self {
                let q = self / v;
                if self % v < 0 {
                    return if *v > 0 { q - 1 } else { q + 1 }
                }
                q
            }

            #[inline]
            fn rem_euclid(&self, v: &$t) -> Self {
                let r = self % v;
                if r < 0 {
                    if *v < 0 {
                        r - v
                    } else {
                        r + v
                    }
                } else {
                    r
                }
            }
        }
    )*}
}

macro_rules! euclid_uint_impl {
    ($($t:ty)*) => {$(
        euclid_forward_impl!($t);

        #[cfg(not(has_div_euclid))]
        impl Euclid for $t {
            #[inline]
            fn div_euclid(&self, v: &$t) -> Self {
                self / v
            }

            #[inline]
            fn rem_euclid(&self, v: &$t) -> Self {
                self % v
            }
        }
    )*}
}

euclid_int_impl!(isize i8 i16 i32 i64);
euclid_uint_impl!(usize u8 u16 u32 u64);
#[cfg(has_i128)]
euclid_int_impl!(i128);
#[cfg(has_i128)]
euclid_uint_impl!(u128);

#[cfg(all(has_div_euclid, feature = "std"))]
euclid_forward_impl!(f32 f64);

#[cfg(not(all(has_div_euclid, feature = "std")))]
impl Euclid for f32 {
    #[inline]
    fn div_euclid(&self, v: &f32) -> f32 {
        let q = <f32 as ::float::FloatCore>::trunc(self / v);
        if self % v < 0.0 {
            return if *v > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    #[inline]
    fn rem_euclid(&self, v: &f32) -> f32 {
        let r = self % v;
        if r < 0.0 {
            r + <f32 as ::float::FloatCore>::abs(*v)
        } else {
            r
        }
    }
}

#[cfg(not(all(has_div_euclid, feature = "std")))]
impl Euclid for f64 {
    #[inline]
    fn div_euclid(&self, v: &f64) -> f64 {
        let q = <f64 as ::float::FloatCore>::trunc(self / v);
        if self % v < 0.0 {
            return if *v > 0.0 { q - 1.0 } else { q + 1.0 };
        }
        q
    }

    #[inline]
    fn rem_euclid(&self, v: &f64) -> f64 {
        let r = self % v;
        if r < 0.0 {
            r + <f64 as ::float::FloatCore>::abs(*v)
        } else {
            r
        }
    }
}

pub trait CheckedEuclid: Euclid {
    /// Performs euclid division that returns `None` instead of panicking on division by zero
    /// and instead of wrapping around on underflow and overflow.
    fn checked_div_euclid(&self, v: &Self) -> Option<Self>;

    /// Finds the euclid remainder of dividing two numbers, checking for underflow, overflow and
    /// division by zero. If any of that happens, `None` is returned.
    fn checked_rem_euclid(&self, v: &Self) -> Option<Self>;
}

macro_rules! checked_euclid_forward_impl {
    ($($t:ty)*) => {$(
        #[cfg(has_div_euclid)]
        impl CheckedEuclid for $t {
            #[inline]
            fn checked_div_euclid(&self, v: &$t) -> Option<Self> {
                <$t>::checked_div_euclid(*self, *v)
            }

            #[inline]
            fn checked_rem_euclid(&self, v: &$t) -> Option<Self> {
                <$t>::checked_rem_euclid(*self, *v)
            }
        }
    )*}
}

macro_rules! checked_euclid_int_impl {
    ($($t:ty)*) => {$(
        checked_euclid_forward_impl!($t);

        #[cfg(not(has_div_euclid))]
        impl CheckedEuclid for $t {
            #[inline]
            fn checked_div_euclid(&self, v: &$t) -> Option<$t> {
                if *v == 0 || (*self == Self::min_value() && *v == -1) {
                    None
                } else {
                    Some(Euclid::div_euclid(self, v))
                }
            }

            #[inline]
            fn checked_rem_euclid(&self, v: &$t) -> Option<$t> {
                if *v == 0 || (*self == Self::min_value() && *v == -1) {
                    None
                } else {
                    Some(Euclid::rem_euclid(self, v))
                }
            }
        }
    )*}
}

macro_rules! checked_euclid_uint_impl {
    ($($t:ty)*) => {$(
        checked_euclid_forward_impl!($t);

        #[cfg(not(has_div_euclid))]
        impl CheckedEuclid for $t {
            #[inline]
            fn checked_div_euclid(&self, v: &$t) -> Option<$t> {
                if *v == 0 {
                    None
                } else {
                    Some(Euclid::div_euclid(self, v))
                }
            }

            #[inline]
            fn checked_rem_euclid(&self, v: &$t) -> Option<$t> {
                if *v == 0 {
                    None
                } else {
                    Some(Euclid::rem_euclid(self, v))
                }
            }
        }
    )*}
}

checked_euclid_int_impl!(isize i8 i16 i32 i64);
checked_euclid_uint_impl!(usize u8 u16 u32 u64);
#[cfg(has_i128)]
checked_euclid_int_impl!(i128);
#[cfg(has_i128)]
checked_euclid_uint_impl!(u128);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclid_unsigned() {
        macro_rules! test_euclid {
            ($($t:ident)+) => {
                $(
                    {
                        let x: $t = 10;
                        let y: $t = 3;
                        assert_eq!(Euclid::div_euclid(&x, &y), 3);
                        assert_eq!(Euclid::rem_euclid(&x, &y), 1);
                    }
                )+
            };
        }

        test_euclid!(usize u8 u16 u32 u64);
    }

    #[test]
    fn euclid_signed() {
        macro_rules! test_euclid {
            ($($t:ident)+) => {
                $(
                    {
                        let x: $t = 10;
                        let y: $t = -3;
                        assert_eq!(Euclid::div_euclid(&x, &y), -3);
                        assert_eq!(Euclid::div_euclid(&-x, &y), 4);
                        assert_eq!(Euclid::rem_euclid(&x, &y), 1);
                        assert_eq!(Euclid::rem_euclid(&-x, &y), 2);
                        let x: $t = $t::min_value() + 1;
                        let y: $t = -1;
                        assert_eq!(Euclid::div_euclid(&x, &y), $t::max_value());
                    }
                )+
            };
        }

        test_euclid!(isize i8 i16 i32 i64);
    }

    #[test]
    fn euclid_float() {
        macro_rules! test_euclid {
            ($($t:ident)+) => {
                $(
                    {
                        let x: $t = 12.1;
                        let y: $t = 3.2;
                        assert!(Euclid::div_euclid(&x, &y) * y + Euclid::rem_euclid(&x, &y) - x
                        <= 46.4 * <$t as ::float::FloatCore>::epsilon());
                        assert!(Euclid::div_euclid(&x, &-y) * -y + Euclid::rem_euclid(&x, &-y) - x
                        <= 46.4 * <$t as ::float::FloatCore>::epsilon());
                        assert!(Euclid::div_euclid(&-x, &y) * y + Euclid::rem_euclid(&-x, &y) + x
                        <= 46.4 * <$t as ::float::FloatCore>::epsilon());
                        assert!(Euclid::div_euclid(&-x, &-y) * -y + Euclid::rem_euclid(&-x, &-y) + x
                        <= 46.4 * <$t as ::float::FloatCore>::epsilon());
                    }
                )+
            };
        }

        test_euclid!(f32 f64);
    }

    #[test]
    fn euclid_checked() {
        macro_rules! test_euclid_checked {
            ($($t:ident)+) => {
                $(
                    {
                        assert_eq!(CheckedEuclid::checked_div_euclid(&$t::min_value(), &-1), None);
                        assert_eq!(CheckedEuclid::checked_rem_euclid(&$t::min_value(), &-1), None);
                        assert_eq!(CheckedEuclid::checked_div_euclid(&1, &0), None);
                        assert_eq!(CheckedEuclid::checked_rem_euclid(&1, &0), None);
                    }
                )+
            };
        }

        test_euclid_checked!(isize i8 i16 i32 i64);
    }
}
