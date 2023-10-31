pub type CurrencyFormatter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Globalization_NumberFormatting\"`*"]
#[repr(transparent)]
pub struct CurrencyFormatterMode(pub i32);
impl CurrencyFormatterMode {
    pub const UseSymbol: Self = Self(0i32);
    pub const UseCurrencyCode: Self = Self(1i32);
}
impl ::core::marker::Copy for CurrencyFormatterMode {}
impl ::core::clone::Clone for CurrencyFormatterMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DecimalFormatter = *mut ::core::ffi::c_void;
pub type INumberFormatter = *mut ::core::ffi::c_void;
pub type INumberFormatter2 = *mut ::core::ffi::c_void;
pub type INumberFormatterOptions = *mut ::core::ffi::c_void;
pub type INumberParser = *mut ::core::ffi::c_void;
pub type INumberRounder = *mut ::core::ffi::c_void;
pub type INumberRounderOption = *mut ::core::ffi::c_void;
pub type ISignedZeroOption = *mut ::core::ffi::c_void;
pub type ISignificantDigitsOption = *mut ::core::ffi::c_void;
pub type IncrementNumberRounder = *mut ::core::ffi::c_void;
pub type NumeralSystemTranslator = *mut ::core::ffi::c_void;
pub type PercentFormatter = *mut ::core::ffi::c_void;
pub type PermilleFormatter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Globalization_NumberFormatting\"`*"]
#[repr(transparent)]
pub struct RoundingAlgorithm(pub i32);
impl RoundingAlgorithm {
    pub const None: Self = Self(0i32);
    pub const RoundDown: Self = Self(1i32);
    pub const RoundUp: Self = Self(2i32);
    pub const RoundTowardsZero: Self = Self(3i32);
    pub const RoundAwayFromZero: Self = Self(4i32);
    pub const RoundHalfDown: Self = Self(5i32);
    pub const RoundHalfUp: Self = Self(6i32);
    pub const RoundHalfTowardsZero: Self = Self(7i32);
    pub const RoundHalfAwayFromZero: Self = Self(8i32);
    pub const RoundHalfToEven: Self = Self(9i32);
    pub const RoundHalfToOdd: Self = Self(10i32);
}
impl ::core::marker::Copy for RoundingAlgorithm {}
impl ::core::clone::Clone for RoundingAlgorithm {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SignificantDigitsNumberRounder = *mut ::core::ffi::c_void;
