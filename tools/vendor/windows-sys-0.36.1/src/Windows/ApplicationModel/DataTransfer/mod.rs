#[cfg(feature = "ApplicationModel_DataTransfer_DragDrop")]
pub mod DragDrop;
#[cfg(feature = "ApplicationModel_DataTransfer_ShareTarget")]
pub mod ShareTarget;
pub type ClipboardContentOptions = *mut ::core::ffi::c_void;
pub type ClipboardHistoryChangedEventArgs = *mut ::core::ffi::c_void;
pub type ClipboardHistoryItem = *mut ::core::ffi::c_void;
pub type ClipboardHistoryItemsResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_DataTransfer\"`*"]
#[repr(transparent)]
pub struct ClipboardHistoryItemsResultStatus(pub i32);
impl ClipboardHistoryItemsResultStatus {
    pub const Success: Self = Self(0i32);
    pub const AccessDenied: Self = Self(1i32);
    pub const ClipboardHistoryDisabled: Self = Self(2i32);
}
impl ::core::marker::Copy for ClipboardHistoryItemsResultStatus {}
impl ::core::clone::Clone for ClipboardHistoryItemsResultStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DataPackage = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_DataTransfer\"`*"]
#[repr(transparent)]
pub struct DataPackageOperation(pub u32);
impl DataPackageOperation {
    pub const None: Self = Self(0u32);
    pub const Copy: Self = Self(1u32);
    pub const Move: Self = Self(2u32);
    pub const Link: Self = Self(4u32);
}
impl ::core::marker::Copy for DataPackageOperation {}
impl ::core::clone::Clone for DataPackageOperation {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DataPackagePropertySet = *mut ::core::ffi::c_void;
pub type DataPackagePropertySetView = *mut ::core::ffi::c_void;
pub type DataPackageView = *mut ::core::ffi::c_void;
pub type DataProviderDeferral = *mut ::core::ffi::c_void;
pub type DataProviderHandler = *mut ::core::ffi::c_void;
pub type DataProviderRequest = *mut ::core::ffi::c_void;
pub type DataRequest = *mut ::core::ffi::c_void;
pub type DataRequestDeferral = *mut ::core::ffi::c_void;
pub type DataRequestedEventArgs = *mut ::core::ffi::c_void;
pub type DataTransferManager = *mut ::core::ffi::c_void;
pub type OperationCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_DataTransfer\"`*"]
#[repr(transparent)]
pub struct SetHistoryItemAsContentStatus(pub i32);
impl SetHistoryItemAsContentStatus {
    pub const Success: Self = Self(0i32);
    pub const AccessDenied: Self = Self(1i32);
    pub const ItemDeleted: Self = Self(2i32);
}
impl ::core::marker::Copy for SetHistoryItemAsContentStatus {}
impl ::core::clone::Clone for SetHistoryItemAsContentStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ShareCompletedEventArgs = *mut ::core::ffi::c_void;
pub type ShareProvider = *mut ::core::ffi::c_void;
pub type ShareProviderHandler = *mut ::core::ffi::c_void;
pub type ShareProviderOperation = *mut ::core::ffi::c_void;
pub type ShareProvidersRequestedEventArgs = *mut ::core::ffi::c_void;
pub type ShareTargetInfo = *mut ::core::ffi::c_void;
pub type ShareUIOptions = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_DataTransfer\"`*"]
#[repr(transparent)]
pub struct ShareUITheme(pub i32);
impl ShareUITheme {
    pub const Default: Self = Self(0i32);
    pub const Light: Self = Self(1i32);
    pub const Dark: Self = Self(2i32);
}
impl ::core::marker::Copy for ShareUITheme {}
impl ::core::clone::Clone for ShareUITheme {
    fn clone(&self) -> Self {
        *self
    }
}
pub type TargetApplicationChosenEventArgs = *mut ::core::ffi::c_void;
