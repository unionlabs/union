macro_rules! impls {
    ($Int: ident, $sign: ident) => {
        #[doc = doc::unchecked::impl_desc!()]
        impl<const N: usize> $Int<N> {
            crate::nightly::const_fns! {
                #[doc = doc::unchecked::unchecked_add!($sign)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const unsafe fn unchecked_add(self, rhs: Self) -> Self {
                    self.checked_add(rhs).unwrap_unchecked()
                }

                #[doc = doc::unchecked::unchecked_sub!($sign)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const unsafe fn unchecked_sub(self, rhs: Self) -> Self {
                    self.checked_sub(rhs).unwrap_unchecked()
                }

                #[doc = doc::unchecked::unchecked_mul!($sign)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const unsafe fn unchecked_mul(self, rhs: Self) -> Self {
                    self.checked_mul(rhs).unwrap_unchecked()
                }

                #[doc = doc::unchecked::unchecked_shl!($sign)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const unsafe fn unchecked_shl(self, rhs: Self) -> Self {
                    let rhs = rhs.to_exp_type().unwrap_unchecked();
                    self.checked_shl(rhs).unwrap_unchecked()
                }

                #[doc = doc::unchecked::unchecked_shr!($sign)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const unsafe fn unchecked_shr(self, rhs: Self) -> Self {
                    let rhs = rhs.to_exp_type().unwrap_unchecked();
                    self.checked_shr(rhs).unwrap_unchecked()
                }
            }
        }
    };
}

pub(crate) use impls;
