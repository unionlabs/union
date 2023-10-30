/// A fraction `p`/`q` with integers `p` and `q`.
///
/// `p` is called the numerator and `q` is called the denominator.
pub trait Fraction<T>: Sized {
    /// Returns the numerator `p`
    fn numerator(&self) -> T;
    /// Returns the denominator `q`
    fn denominator(&self) -> T;

    /// Returns the multiplicative inverse `q/p` for fraction `p/q`.
    ///
    /// If `p` is zero, None is returned.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn inv(&self) -> Option<Self>;
}

impl<T: Copy + From<u8> + PartialEq> Fraction<T> for (T, T) {
    fn numerator(&self) -> T {
        self.0
    }

    fn denominator(&self) -> T {
        self.1
    }

    fn inv(&self) -> Option<Self> {
        if self.numerator() == 0u8.into() {
            None
        } else {
            Some((self.1, self.0))
        }
    }
}

#[macro_export]
macro_rules! impl_mul_fraction {
    ($Uint:ident) => {
        impl $Uint {
            /// Multiply `self` with a struct implementing [`Fraction`] (e.g. [`crate::Decimal`]).
            /// Result is rounded down.
            ///
            /// ## Examples
            ///
            /// ```
            /// use cosmwasm_std::Uint128;
            /// let fraction = (8u128, 21u128);
            /// let res = Uint128::new(123456).checked_mul_floor(fraction).unwrap();
            /// assert_eq!(Uint128::new(47030), res); // 47030.8571 rounds down
            /// ```
            pub fn checked_mul_floor<F: Fraction<T>, T: Into<$Uint>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionError> {
                let divisor = rhs.denominator().into();
                let res = self
                    .full_mul(rhs.numerator().into())
                    .checked_div(divisor.into())?;
                Ok(res.try_into()?)
            }

            /// Same operation as `checked_mul_floor` except unwrapped
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub fn mul_floor<F: Fraction<T>, T: Into<$Uint>>(self, rhs: F) -> Self {
                self.checked_mul_floor(rhs).unwrap()
            }

            /// Multiply `self` with a struct implementing [`Fraction`] (e.g. [`crate::Decimal`]).
            /// Result is rounded up.
            ///
            /// ## Examples
            ///
            /// ```
            /// use cosmwasm_std::Uint128;
            /// let fraction = (8u128, 21u128);
            /// let res = Uint128::new(123456).checked_mul_ceil(fraction).unwrap();
            /// assert_eq!(Uint128::new(47031), res); // 47030.8571 rounds up
            /// ```
            pub fn checked_mul_ceil<F: Fraction<T>, T: Into<$Uint>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionError> {
                let dividend = self.full_mul(rhs.numerator().into());
                let divisor = rhs.denominator().into().into();
                let floor_result = dividend.checked_div(divisor)?.try_into()?;
                let remainder = dividend.checked_rem(divisor)?;
                if !remainder.is_zero() {
                    Ok($Uint::one().checked_add(floor_result)?)
                } else {
                    Ok(floor_result)
                }
            }

            /// Same operation as `checked_mul_ceil` except unwrapped
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub fn mul_ceil<F: Fraction<T>, T: Into<$Uint>>(self, rhs: F) -> Self {
                self.checked_mul_ceil(rhs).unwrap()
            }

            /// Divide `self` with a struct implementing [`Fraction`] (e.g. [`crate::Decimal`]).
            /// Result is rounded down.
            ///
            /// ## Examples
            ///
            /// ```
            /// use cosmwasm_std::Uint128;
            /// let fraction = (4u128, 5u128);
            /// let res = Uint128::new(789).checked_div_floor(fraction).unwrap();
            /// assert_eq!(Uint128::new(986), res); // 986.25 rounds down
            /// ```
            pub fn checked_div_floor<F: Fraction<T>, T: Into<$Uint>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionError>
            where
                Self: Sized,
            {
                let divisor = rhs.numerator().into();
                let res = self
                    .full_mul(rhs.denominator().into())
                    .checked_div(divisor.into())?;
                Ok(res.try_into()?)
            }

            /// Same operation as `checked_div_floor` except unwrapped
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub fn div_floor<F: Fraction<T>, T: Into<$Uint>>(self, rhs: F) -> Self
            where
                Self: Sized,
            {
                self.checked_div_floor(rhs).unwrap()
            }

            /// Divide `self` with a struct implementing [`Fraction`] (e.g. [`crate::Decimal`]).
            /// Result is rounded up.
            ///
            /// ## Examples
            ///
            /// ```
            /// use cosmwasm_std::Uint128;
            /// let fraction = (4u128, 5u128);
            /// let res = Uint128::new(789).checked_div_ceil(fraction).unwrap();
            /// assert_eq!(Uint128::new(987), res); // 986.25 rounds up
            /// ```
            pub fn checked_div_ceil<F: Fraction<T>, T: Into<$Uint>>(
                self,
                rhs: F,
            ) -> Result<Self, CheckedMultiplyFractionError>
            where
                Self: Sized,
            {
                let dividend = self.full_mul(rhs.denominator().into());
                let divisor = rhs.numerator().into().into();
                let floor_result = dividend.checked_div(divisor)?.try_into()?;
                let remainder = dividend.checked_rem(divisor)?;
                if !remainder.is_zero() {
                    Ok($Uint::one().checked_add(floor_result)?)
                } else {
                    Ok(floor_result)
                }
            }

            /// Same operation as `checked_div_ceil` except unwrapped
            #[must_use = "this returns the result of the operation, without modifying the original"]
            pub fn div_ceil<F: Fraction<T>, T: Into<$Uint>>(self, rhs: F) -> Self
            where
                Self: Sized,
            {
                self.checked_div_ceil(rhs).unwrap()
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{Fraction, Uint128, Uint64};

    #[test]
    fn fraction_tuple_methods() {
        let fraction = (Uint64::one(), Uint64::new(2));
        assert_eq!(Uint64::one(), fraction.numerator());
        assert_eq!(Uint64::new(2), fraction.denominator());
        assert_eq!(Some((Uint64::new(2), Uint64::one())), fraction.inv());
    }

    #[test]
    fn inverse_with_zero_denominator() {
        let fraction = (Uint128::zero(), Uint128::one());
        assert_eq!(None, fraction.inv());
    }
}
