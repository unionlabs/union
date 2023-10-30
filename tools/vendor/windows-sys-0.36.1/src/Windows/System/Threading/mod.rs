#[cfg(feature = "System_Threading_Core")]
pub mod Core;
pub type ThreadPoolTimer = *mut ::core::ffi::c_void;
pub type TimerDestroyedHandler = *mut ::core::ffi::c_void;
pub type TimerElapsedHandler = *mut ::core::ffi::c_void;
pub type WorkItemHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Threading\"`*"]
#[repr(transparent)]
pub struct WorkItemOptions(pub u32);
impl WorkItemOptions {
    pub const None: Self = Self(0u32);
    pub const TimeSliced: Self = Self(1u32);
}
impl ::core::marker::Copy for WorkItemOptions {}
impl ::core::clone::Clone for WorkItemOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_Threading\"`*"]
#[repr(transparent)]
pub struct WorkItemPriority(pub i32);
impl WorkItemPriority {
    pub const Low: Self = Self(-1i32);
    pub const Normal: Self = Self(0i32);
    pub const High: Self = Self(1i32);
}
impl ::core::marker::Copy for WorkItemPriority {}
impl ::core::clone::Clone for WorkItemPriority {
    fn clone(&self) -> Self {
        *self
    }
}
