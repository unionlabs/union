pub type DateTimeFormatter = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct DayFormat(pub i32);
impl DayFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
}
impl ::core::marker::Copy for DayFormat {}
impl ::core::clone::Clone for DayFormat {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct DayOfWeekFormat(pub i32);
impl DayOfWeekFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
    pub const Abbreviated: Self = Self(2i32);
    pub const Full: Self = Self(3i32);
}
impl ::core::marker::Copy for DayOfWeekFormat {}
impl ::core::clone::Clone for DayOfWeekFormat {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct HourFormat(pub i32);
impl HourFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
}
impl ::core::marker::Copy for HourFormat {}
impl ::core::clone::Clone for HourFormat {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct MinuteFormat(pub i32);
impl MinuteFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
}
impl ::core::marker::Copy for MinuteFormat {}
impl ::core::clone::Clone for MinuteFormat {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct MonthFormat(pub i32);
impl MonthFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
    pub const Abbreviated: Self = Self(2i32);
    pub const Full: Self = Self(3i32);
    pub const Numeric: Self = Self(4i32);
}
impl ::core::marker::Copy for MonthFormat {}
impl ::core::clone::Clone for MonthFormat {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct SecondFormat(pub i32);
impl SecondFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
}
impl ::core::marker::Copy for SecondFormat {}
impl ::core::clone::Clone for SecondFormat {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Globalization_DateTimeFormatting\"`*"]
#[repr(transparent)]
pub struct YearFormat(pub i32);
impl YearFormat {
    pub const None: Self = Self(0i32);
    pub const Default: Self = Self(1i32);
    pub const Abbreviated: Self = Self(2i32);
    pub const Full: Self = Self(3i32);
}
impl ::core::marker::Copy for YearFormat {}
impl ::core::clone::Clone for YearFormat {
    fn clone(&self) -> Self {
        *self
    }
}
