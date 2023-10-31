#[cfg(feature = "Foundation_Collections")]
pub mod Collections;
#[cfg(feature = "Foundation_Diagnostics")]
pub mod Diagnostics;
#[cfg(feature = "Foundation_Metadata")]
pub mod Metadata;
#[cfg(feature = "Foundation_Numerics")]
pub mod Numerics;
pub type AsyncActionCompletedHandler = *mut ::core::ffi::c_void;
pub type AsyncActionProgressHandler = *mut ::core::ffi::c_void;
pub type AsyncActionWithProgressCompletedHandler = *mut ::core::ffi::c_void;
pub type AsyncOperationCompletedHandler = *mut ::core::ffi::c_void;
pub type AsyncOperationProgressHandler = *mut ::core::ffi::c_void;
pub type AsyncOperationWithProgressCompletedHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Foundation\"`*"]
#[repr(transparent)]
pub struct AsyncStatus(pub i32);
impl AsyncStatus {
    pub const Canceled: Self = Self(2i32);
    pub const Completed: Self = Self(1i32);
    pub const Error: Self = Self(3i32);
    pub const Started: Self = Self(0i32);
}
impl ::core::marker::Copy for AsyncStatus {}
impl ::core::clone::Clone for AsyncStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation\"`*"]
pub struct DateTime {
    pub UniversalTime: i64,
}
impl ::core::marker::Copy for DateTime {}
impl ::core::clone::Clone for DateTime {
    fn clone(&self) -> Self {
        *self
    }
}
pub type Deferral = *mut ::core::ffi::c_void;
pub type DeferralCompletedHandler = *mut ::core::ffi::c_void;
pub type EventHandler = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Foundation\"`*"]
pub struct EventRegistrationToken {
    pub Value: i64,
}
impl ::core::marker::Copy for EventRegistrationToken {}
impl ::core::clone::Clone for EventRegistrationToken {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IAsyncAction = *mut ::core::ffi::c_void;
pub type IAsyncActionWithProgress = *mut ::core::ffi::c_void;
pub type IAsyncInfo = *mut ::core::ffi::c_void;
pub type IAsyncOperation = *mut ::core::ffi::c_void;
pub type IAsyncOperationWithProgress = *mut ::core::ffi::c_void;
pub type IClosable = *mut ::core::ffi::c_void;
pub type IGetActivationFactory = *mut ::core::ffi::c_void;
pub type IMemoryBuffer = *mut ::core::ffi::c_void;
pub type IMemoryBufferReference = *mut ::core::ffi::c_void;
pub type IPropertyValue = *mut ::core::ffi::c_void;
pub type IReference = *mut ::core::ffi::c_void;
pub type IReferenceArray = *mut ::core::ffi::c_void;
pub type IStringable = *mut ::core::ffi::c_void;
pub type IWwwFormUrlDecoderEntry = *mut ::core::ffi::c_void;
pub type MemoryBuffer = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Foundation\"`*"]
pub struct Point {
    pub X: f32,
    pub Y: f32,
}
impl ::core::marker::Copy for Point {}
impl ::core::clone::Clone for Point {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Foundation\"`*"]
#[repr(transparent)]
pub struct PropertyType(pub i32);
impl PropertyType {
    pub const Empty: Self = Self(0i32);
    pub const UInt8: Self = Self(1i32);
    pub const Int16: Self = Self(2i32);
    pub const UInt16: Self = Self(3i32);
    pub const Int32: Self = Self(4i32);
    pub const UInt32: Self = Self(5i32);
    pub const Int64: Self = Self(6i32);
    pub const UInt64: Self = Self(7i32);
    pub const Single: Self = Self(8i32);
    pub const Double: Self = Self(9i32);
    pub const Char16: Self = Self(10i32);
    pub const Boolean: Self = Self(11i32);
    pub const String: Self = Self(12i32);
    pub const Inspectable: Self = Self(13i32);
    pub const DateTime: Self = Self(14i32);
    pub const TimeSpan: Self = Self(15i32);
    pub const Guid: Self = Self(16i32);
    pub const Point: Self = Self(17i32);
    pub const Size: Self = Self(18i32);
    pub const Rect: Self = Self(19i32);
    pub const OtherType: Self = Self(20i32);
    pub const UInt8Array: Self = Self(1025i32);
    pub const Int16Array: Self = Self(1026i32);
    pub const UInt16Array: Self = Self(1027i32);
    pub const Int32Array: Self = Self(1028i32);
    pub const UInt32Array: Self = Self(1029i32);
    pub const Int64Array: Self = Self(1030i32);
    pub const UInt64Array: Self = Self(1031i32);
    pub const SingleArray: Self = Self(1032i32);
    pub const DoubleArray: Self = Self(1033i32);
    pub const Char16Array: Self = Self(1034i32);
    pub const BooleanArray: Self = Self(1035i32);
    pub const StringArray: Self = Self(1036i32);
    pub const InspectableArray: Self = Self(1037i32);
    pub const DateTimeArray: Self = Self(1038i32);
    pub const TimeSpanArray: Self = Self(1039i32);
    pub const GuidArray: Self = Self(1040i32);
    pub const PointArray: Self = Self(1041i32);
    pub const SizeArray: Self = Self(1042i32);
    pub const RectArray: Self = Self(1043i32);
    pub const OtherTypeArray: Self = Self(1044i32);
}
impl ::core::marker::Copy for PropertyType {}
impl ::core::clone::Clone for PropertyType {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation\"`*"]
pub struct Rect {
    pub X: f32,
    pub Y: f32,
    pub Width: f32,
    pub Height: f32,
}
impl ::core::marker::Copy for Rect {}
impl ::core::clone::Clone for Rect {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation\"`*"]
pub struct Size {
    pub Width: f32,
    pub Height: f32,
}
impl ::core::marker::Copy for Size {}
impl ::core::clone::Clone for Size {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Foundation\"`*"]
pub struct TimeSpan {
    pub Duration: i64,
}
impl ::core::marker::Copy for TimeSpan {}
impl ::core::clone::Clone for TimeSpan {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TypedEventHandler = *mut ::core::ffi::c_void;
pub type Uri = *mut ::core::ffi::c_void;
pub type WwwFormUrlDecoder = *mut ::core::ffi::c_void;
pub type WwwFormUrlDecoderEntry = *mut ::core::ffi::c_void;
