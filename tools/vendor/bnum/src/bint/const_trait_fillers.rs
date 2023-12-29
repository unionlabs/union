use crate::doc;
use crate::ExpType;
use core::cmp::Ordering;

macro_rules! const_trait_fillers {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        #[doc = doc::const_trait_fillers::impl_desc!()]
        impl<const N: usize> $BInt<N> {
            #[inline]
            pub const fn bitand(self, rhs: Self) -> Self {
                Self::from_bits(self.bits.bitand(rhs.bits))
            }

            #[inline]
            pub const fn bitor(self, rhs: Self) -> Self {
                Self::from_bits(self.bits.bitor(rhs.bits))
            }

            #[inline]
            pub const fn bitxor(self, rhs: Self) -> Self {
                Self::from_bits(self.bits.bitxor(rhs.bits))
            }

            #[inline]
            pub const fn not(self) -> Self {
                Self::from_bits(self.bits.not())
            }

            #[inline]
            pub const fn eq(&self, other: &Self) -> bool {
                $BUint::eq(&self.bits, &other.bits)
            }

            #[inline]
            pub const fn ne(&self, other: &Self) -> bool {
                !Self::eq(self, other)
            }

            #[inline]
            pub const fn cmp(&self, other: &Self) -> Ordering {
                let s1 = self.signed_digit();
                let s2 = other.signed_digit();

                // Don't use match here as `cmp` is not yet const for primitive integers
                #[allow(clippy::comparison_chain)]
                if s1 == s2 {
                    $BUint::cmp(&self.bits, &other.bits)
                } else if s1 > s2 {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            }

            crate::int::cmp::impls!();
            #[inline]
            pub const fn neg(self) -> Self {
                #[cfg(debug_assertions)]
                return crate::errors::option_expect!(self.checked_neg(), crate::errors::err_msg!("attempt to negate with overflow"));

                #[cfg(not(debug_assertions))]
                self.wrapping_neg()
            }

            crate::int::ops::trait_fillers!();

            #[inline]
            pub const fn div(self, rhs: Self) -> Self {
                if self.eq(&Self::MIN) && rhs.eq(&Self::NEG_ONE) {
                    panic!(crate::errors::err_msg!("attempt to divide with overflow"))
                } else {
                    if rhs.is_zero() {
                        crate::errors::div_zero!()
                    }
                    self.div_rem_unchecked(rhs).0
                }
            }

            #[inline]
            pub const fn rem(self, rhs: Self) -> Self {
                if self.eq(&Self::MIN) && rhs.eq(&Self::NEG_ONE) {
                    panic!(crate::errors::err_msg!("attempt to calculate remainder with overflow"))
                } else {
                    if rhs.is_zero() {
                        crate::errors::rem_zero!()
                    }
                    self.div_rem_unchecked(rhs).1
                }
            }
        }
    };
}

crate::macro_impl!(const_trait_fillers);
