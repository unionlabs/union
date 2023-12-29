macro_rules! impls {
    () => {
        #[inline]
        pub const fn max(self, other: Self) -> Self {
            match self.cmp(&other) {
                Ordering::Less | Ordering::Equal => other,
                _ => self,
            }
        }

        #[inline]
        pub const fn min(self, other: Self) -> Self {
            match self.cmp(&other) {
                Ordering::Less | Ordering::Equal => self,
                _ => other,
            }
        }

        #[inline]
        pub const fn clamp(self, min: Self, max: Self) -> Self {
            assert!(min.le(max));
            if let Ordering::Less = self.cmp(&min) {
                min
            } else if let Ordering::Greater = self.cmp(&max) {
                max
            } else {
                self
            }
        }

        #[inline]
        pub const fn lt(self, other: Self) -> bool {
            match self.cmp(&other) {
                Ordering::Less => true,
                _ => false,
            }
        }

        #[inline]
        pub const fn le(self, other: Self) -> bool {
            match self.cmp(&other) {
                Ordering::Less | Ordering::Equal => true,
                _ => false,
            }
        }

        #[inline]
        pub const fn gt(self, other: Self) -> bool {
            match self.cmp(&other) {
                Ordering::Greater => true,
                _ => false,
            }
        }

        #[inline]
        pub const fn ge(self, other: Self) -> bool {
            match self.cmp(&other) {
                Ordering::Greater | Ordering::Equal => true,
                _ => false,
            }
        }
    };
}

pub(crate) use impls;

#[cfg(test)]
macro_rules! tests {
    ($int: ty) => {
        use crate::test::{test_bignum, types::*};

        test_bignum! {
            function: <$int>::eq(a: ref &$int, b: ref &$int)
        }
        test_bignum! {
            function: <$int>::partial_cmp(a: ref &$int, b: ref &$int)
        }

        test_bignum! {
            function: <$int>::cmp(a: ref &$int, b: ref &$int)
        }
        test_bignum! {
            function: <$int>::max(a: $int, b: $int)
        }
        test_bignum! {
            function: <$int>::min(a: $int, b: $int)
        }
        test_bignum! {
            function: <$int>::clamp(a: $int, min: $int, max: $int),
            skip: min > max
        }
    };
}

#[cfg(test)]
pub(crate) use tests;
