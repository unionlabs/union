macro_rules! digit_module {
    ($Digit: ident, $SignedDigit: ty, $DoubleDigit: ty) => {
        pub mod $Digit {
            mod types {
                pub type Digit = $Digit;

                pub type SignedDigit = $SignedDigit;

                pub type DoubleDigit = $DoubleDigit;
            }

            use crate::ExpType;

            pub use types::*;

            pub const BITS: ExpType = $Digit::BITS as ExpType;

            pub const BITS_U8: u8 = BITS as u8;

            pub const BITS_MINUS_1: ExpType = BITS - 1;

            pub const BYTES: ExpType = BITS / 8;

            // This calculates log2 of BYTES as BYTES is guaranteed to only have one '1' bit, since it must be a power of two.
            pub const BYTE_SHIFT: ExpType = BYTES.trailing_zeros() as ExpType;

            pub const BIT_SHIFT: ExpType = BITS.trailing_zeros() as ExpType;

            #[inline]
            pub const fn to_double_digit(low: Digit, high: Digit) -> DoubleDigit {
                ((high as DoubleDigit) << BITS) | low as DoubleDigit
            }

            // TODO: these will no longer be necessary once const_bigint_helper_methods is stabilised: https://github.com/rust-lang/rust/issues/85532

            #[inline]
            pub const fn carrying_add(a: Digit, b: Digit, carry: bool) -> (Digit, bool) {
                let (s1, o1) = a.overflowing_add(b);
                if carry {
                    let (s2, o2) = s1.overflowing_add(1);
                    (s2, o1 || o2)
                } else {
                    (s1, o1)
                }
            }

            #[inline]
            pub const fn borrowing_sub(a: Digit, b: Digit, borrow: bool) -> (Digit, bool) {
                let (s1, o1) = a.overflowing_sub(b);
                if borrow {
                    let (s2, o2) = s1.overflowing_sub(1);
                    (s2, o1 || o2)
                } else {
                    (s1, o1)
                }
            }

            #[inline]
            pub const fn carrying_add_signed(
                a: SignedDigit,
                b: SignedDigit,
                carry: bool,
            ) -> (SignedDigit, bool) {
                let (s1, o1) = a.overflowing_add(b);
                if carry {
                    let (s2, o2) = s1.overflowing_add(1);
                    (s2, o1 != o2)
                } else {
                    (s1, o1)
                }
            }

            #[inline]
            pub const fn borrowing_sub_signed(
                a: SignedDigit,
                b: SignedDigit,
                borrow: bool,
            ) -> (SignedDigit, bool) {
                let (s1, o1) = a.overflowing_sub(b);
                if borrow {
                    let (s2, o2) = s1.overflowing_sub(1);
                    (s2, o1 != o2)
                } else {
                    (s1, o1)
                }
            }

            #[inline]
            pub const fn widening_mul(a: Digit, b: Digit) -> (Digit, Digit) {
                let prod = a as DoubleDigit * b as DoubleDigit;
                (prod as Digit, (prod >> BITS) as Digit)
            }

            #[inline]
            pub const fn carrying_mul(
                a: Digit,
                b: Digit,
                carry: Digit,
                current: Digit,
            ) -> (Digit, Digit) {
                let prod = carry as DoubleDigit
                    + current as DoubleDigit
                    + (a as DoubleDigit) * (b as DoubleDigit);
                (prod as Digit, (prod >> BITS) as Digit)
            }

            #[inline]
            pub const fn div_rem_wide(low: Digit, high: Digit, rhs: Digit) -> (Digit, Digit) {
                debug_assert!(high < rhs);

                let a = to_double_digit(low, high);
                (
                    (a / rhs as DoubleDigit) as Digit,
                    (a % rhs as DoubleDigit) as Digit,
                )
            }

            pub const HEX_PADDING: usize = BITS as usize / 4;
        }
    };
}

digit_module!(u8, i8, u16);
digit_module!(u16, i16, u32);
digit_module!(u32, i32, u64);
digit_module!(u64, i64, u128);
