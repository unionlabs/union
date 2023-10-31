#[doc = "*Required features: `\"System_Update\"`*"]
#[repr(transparent)]
pub struct SystemUpdateAttentionRequiredReason(pub i32);
impl SystemUpdateAttentionRequiredReason {
    pub const None: Self = Self(0i32);
    pub const NetworkRequired: Self = Self(1i32);
    pub const InsufficientDiskSpace: Self = Self(2i32);
    pub const InsufficientBattery: Self = Self(3i32);
    pub const UpdateBlocked: Self = Self(4i32);
}
impl ::core::marker::Copy for SystemUpdateAttentionRequiredReason {}
impl ::core::clone::Clone for SystemUpdateAttentionRequiredReason {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SystemUpdateItem = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Update\"`*"]
#[repr(transparent)]
pub struct SystemUpdateItemState(pub i32);
impl SystemUpdateItemState {
    pub const NotStarted: Self = Self(0i32);
    pub const Initializing: Self = Self(1i32);
    pub const Preparing: Self = Self(2i32);
    pub const Calculating: Self = Self(3i32);
    pub const Downloading: Self = Self(4i32);
    pub const Installing: Self = Self(5i32);
    pub const Completed: Self = Self(6i32);
    pub const RebootRequired: Self = Self(7i32);
    pub const Error: Self = Self(8i32);
}
impl ::core::marker::Copy for SystemUpdateItemState {}
impl ::core::clone::Clone for SystemUpdateItemState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SystemUpdateLastErrorInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"System_Update\"`*"]
#[repr(transparent)]
pub struct SystemUpdateManagerState(pub i32);
impl SystemUpdateManagerState {
    pub const Idle: Self = Self(0i32);
    pub const Detecting: Self = Self(1i32);
    pub const ReadyToDownload: Self = Self(2i32);
    pub const Downloading: Self = Self(3i32);
    pub const ReadyToInstall: Self = Self(4i32);
    pub const Installing: Self = Self(5i32);
    pub const RebootRequired: Self = Self(6i32);
    pub const ReadyToFinalize: Self = Self(7i32);
    pub const Finalizing: Self = Self(8i32);
    pub const Completed: Self = Self(9i32);
    pub const AttentionRequired: Self = Self(10i32);
    pub const Error: Self = Self(11i32);
}
impl ::core::marker::Copy for SystemUpdateManagerState {}
impl ::core::clone::Clone for SystemUpdateManagerState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"System_Update\"`*"]
#[repr(transparent)]
pub struct SystemUpdateStartInstallAction(pub i32);
impl SystemUpdateStartInstallAction {
    pub const UpToReboot: Self = Self(0i32);
    pub const AllowReboot: Self = Self(1i32);
}
impl ::core::marker::Copy for SystemUpdateStartInstallAction {}
impl ::core::clone::Clone for SystemUpdateStartInstallAction {
    fn clone(&self) -> Self {
        *self
    }
}
