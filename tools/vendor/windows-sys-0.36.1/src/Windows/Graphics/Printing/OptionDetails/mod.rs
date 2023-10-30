pub type IPrintCustomOptionDetails = *mut ::core::ffi::c_void;
pub type IPrintItemListOptionDetails = *mut ::core::ffi::c_void;
pub type IPrintNumberOptionDetails = *mut ::core::ffi::c_void;
pub type IPrintOptionDetails = *mut ::core::ffi::c_void;
pub type IPrintTextOptionDetails = *mut ::core::ffi::c_void;
pub type PrintBindingOptionDetails = *mut ::core::ffi::c_void;
pub type PrintBorderingOptionDetails = *mut ::core::ffi::c_void;
pub type PrintCollationOptionDetails = *mut ::core::ffi::c_void;
pub type PrintColorModeOptionDetails = *mut ::core::ffi::c_void;
pub type PrintCopiesOptionDetails = *mut ::core::ffi::c_void;
pub type PrintCustomItemDetails = *mut ::core::ffi::c_void;
pub type PrintCustomItemListOptionDetails = *mut ::core::ffi::c_void;
pub type PrintCustomTextOptionDetails = *mut ::core::ffi::c_void;
pub type PrintCustomToggleOptionDetails = *mut ::core::ffi::c_void;
pub type PrintDuplexOptionDetails = *mut ::core::ffi::c_void;
pub type PrintHolePunchOptionDetails = *mut ::core::ffi::c_void;
pub type PrintMediaSizeOptionDetails = *mut ::core::ffi::c_void;
pub type PrintMediaTypeOptionDetails = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Graphics_Printing_OptionDetails\"`*"]
#[repr(transparent)]
pub struct PrintOptionStates(pub u32);
impl PrintOptionStates {
    pub const None: Self = Self(0u32);
    pub const Enabled: Self = Self(1u32);
    pub const Constrained: Self = Self(2u32);
}
impl ::core::marker::Copy for PrintOptionStates {}
impl ::core::clone::Clone for PrintOptionStates {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Graphics_Printing_OptionDetails\"`*"]
#[repr(transparent)]
pub struct PrintOptionType(pub i32);
impl PrintOptionType {
    pub const Unknown: Self = Self(0i32);
    pub const Number: Self = Self(1i32);
    pub const Text: Self = Self(2i32);
    pub const ItemList: Self = Self(3i32);
    pub const Toggle: Self = Self(4i32);
}
impl ::core::marker::Copy for PrintOptionType {}
impl ::core::clone::Clone for PrintOptionType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PrintOrientationOptionDetails = *mut ::core::ffi::c_void;
pub type PrintPageRangeOptionDetails = *mut ::core::ffi::c_void;
pub type PrintQualityOptionDetails = *mut ::core::ffi::c_void;
pub type PrintStapleOptionDetails = *mut ::core::ffi::c_void;
pub type PrintTaskOptionChangedEventArgs = *mut ::core::ffi::c_void;
pub type PrintTaskOptionDetails = *mut ::core::ffi::c_void;
