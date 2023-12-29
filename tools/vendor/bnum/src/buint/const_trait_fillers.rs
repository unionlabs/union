use crate::doc;
use crate::ExpType;
use core::cmp::Ordering;

macro_rules! const_trait_fillers {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        #[doc = doc::const_trait_fillers::impl_desc!()]
        impl<const N: usize> $BUint<N> {
            #[inline]
            pub const fn bitand(self, rhs: Self) -> Self {
                let mut out = Self::ZERO;
                let mut i = 0;
                while i < N {
                    out.digits[i] = self.digits[i] & rhs.digits[i];
                    i += 1;
                }
                out
            }

            #[inline]
            pub const fn bitor(self, rhs: Self) -> Self {
                let mut out = Self::ZERO;
                let mut i = 0;
                while i < N {
                    out.digits[i] = self.digits[i] | rhs.digits[i];
                    i += 1;
                }
                out
            }

            #[inline]
            pub const fn bitxor(self, rhs: Self) -> Self {
                let mut out = Self::ZERO;
                let mut i = 0;
                while i < N {
                    out.digits[i] = self.digits[i] ^ rhs.digits[i];
                    i += 1;
                }
                out
            }

            #[inline]
            pub const fn not(self) -> Self {
                let mut out = Self::ZERO;
                let mut i = 0;
                while i < N {
                    out.digits[i] = !self.digits[i];
                    i += 1;
                }
                out
            }

            #[inline]
            pub const fn eq(&self, other: &Self) -> bool {
                let mut i = 0;
                while i < N {
                    if self.digits[i] != other.digits[i] {
                        return false;
                    }
                    i += 1;
                }
                true
            }

            #[inline]
            pub const fn ne(&self, other: &Self) -> bool {
                !Self::eq(self, other)
            }

            #[inline]
            pub const fn cmp(&self, other: &Self) -> Ordering {
                let mut i = N;
                while i > 0 {
                    i -= 1;
                    let a = self.digits[i];
                    let b = other.digits[i];

                    // Clippy: don't use match here as `cmp` is not yet const for primitive integers
                    #[allow(clippy::comparison_chain)]
                    if a > b {
                        return Ordering::Greater;
                    } else if a < b {
                        return Ordering::Less;
                    }
                }
                Ordering::Equal
            }

            crate::int::cmp::impls!();

            crate::int::ops::trait_fillers!();

            #[inline]
            pub const fn div(self, rhs: Self) -> Self {
                self.wrapping_div(rhs)
            }

            #[inline]
            pub const fn rem(self, rhs: Self) -> Self {
                self.wrapping_rem(rhs)
            }
        }
    };
}

crate::macro_impl!(const_trait_fillers);
