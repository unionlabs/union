pub type StoreAcquireLicenseResult = *mut ::core::ffi::c_void;
pub type StoreAppLicense = *mut ::core::ffi::c_void;
pub type StoreAvailability = *mut ::core::ffi::c_void;
pub type StoreCanAcquireLicenseResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreCanLicenseStatus(pub i32);
impl StoreCanLicenseStatus {
    pub const NotLicensableToUser: Self = Self(0i32);
    pub const Licensable: Self = Self(1i32);
    pub const LicenseActionNotApplicableToProduct: Self = Self(2i32);
    pub const NetworkError: Self = Self(3i32);
    pub const ServerError: Self = Self(4i32);
}
impl ::core::marker::Copy for StoreCanLicenseStatus {}
impl ::core::clone::Clone for StoreCanLicenseStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreCollectionData = *mut ::core::ffi::c_void;
pub type StoreConsumableResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreConsumableStatus(pub i32);
impl StoreConsumableStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const InsufficentQuantity: Self = Self(1i32);
    pub const NetworkError: Self = Self(2i32);
    pub const ServerError: Self = Self(3i32);
}
impl ::core::marker::Copy for StoreConsumableStatus {}
impl ::core::clone::Clone for StoreConsumableStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreContext = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreDurationUnit(pub i32);
impl StoreDurationUnit {
    pub const Minute: Self = Self(0i32);
    pub const Hour: Self = Self(1i32);
    pub const Day: Self = Self(2i32);
    pub const Week: Self = Self(3i32);
    pub const Month: Self = Self(4i32);
    pub const Year: Self = Self(5i32);
}
impl ::core::marker::Copy for StoreDurationUnit {}
impl ::core::clone::Clone for StoreDurationUnit {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreImage = *mut ::core::ffi::c_void;
pub type StoreLicense = *mut ::core::ffi::c_void;
pub type StorePackageInstallOptions = *mut ::core::ffi::c_void;
pub type StorePackageLicense = *mut ::core::ffi::c_void;
pub type StorePackageUpdate = *mut ::core::ffi::c_void;
pub type StorePackageUpdateResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StorePackageUpdateState(pub i32);
impl StorePackageUpdateState {
    pub const Pending: Self = Self(0i32);
    pub const Downloading: Self = Self(1i32);
    pub const Deploying: Self = Self(2i32);
    pub const Completed: Self = Self(3i32);
    pub const Canceled: Self = Self(4i32);
    pub const OtherError: Self = Self(5i32);
    pub const ErrorLowBattery: Self = Self(6i32);
    pub const ErrorWiFiRecommended: Self = Self(7i32);
    pub const ErrorWiFiRequired: Self = Self(8i32);
}
impl ::core::marker::Copy for StorePackageUpdateState {}
impl ::core::clone::Clone for StorePackageUpdateState {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Services_Store\"`*"]
pub struct StorePackageUpdateStatus {
    pub PackageFamilyName: ::windows_sys::core::HSTRING,
    pub PackageDownloadSizeInBytes: u64,
    pub PackageBytesDownloaded: u64,
    pub PackageDownloadProgress: f64,
    pub TotalDownloadProgress: f64,
    pub PackageUpdateState: StorePackageUpdateState,
}
impl ::core::marker::Copy for StorePackageUpdateStatus {}
impl ::core::clone::Clone for StorePackageUpdateStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StorePrice = *mut ::core::ffi::c_void;
pub type StoreProduct = *mut ::core::ffi::c_void;
pub type StoreProductOptions = *mut ::core::ffi::c_void;
pub type StoreProductPagedQueryResult = *mut ::core::ffi::c_void;
pub type StoreProductQueryResult = *mut ::core::ffi::c_void;
pub type StoreProductResult = *mut ::core::ffi::c_void;
pub type StorePurchaseProperties = *mut ::core::ffi::c_void;
pub type StorePurchaseResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StorePurchaseStatus(pub i32);
impl StorePurchaseStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const AlreadyPurchased: Self = Self(1i32);
    pub const NotPurchased: Self = Self(2i32);
    pub const NetworkError: Self = Self(3i32);
    pub const ServerError: Self = Self(4i32);
}
impl ::core::marker::Copy for StorePurchaseStatus {}
impl ::core::clone::Clone for StorePurchaseStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreQueueItem = *mut ::core::ffi::c_void;
pub type StoreQueueItemCompletedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreQueueItemExtendedState(pub i32);
impl StoreQueueItemExtendedState {
    pub const ActivePending: Self = Self(0i32);
    pub const ActiveStarting: Self = Self(1i32);
    pub const ActiveAcquiringLicense: Self = Self(2i32);
    pub const ActiveDownloading: Self = Self(3i32);
    pub const ActiveRestoringData: Self = Self(4i32);
    pub const ActiveInstalling: Self = Self(5i32);
    pub const Completed: Self = Self(6i32);
    pub const Canceled: Self = Self(7i32);
    pub const Paused: Self = Self(8i32);
    pub const Error: Self = Self(9i32);
    pub const PausedPackagesInUse: Self = Self(10i32);
    pub const PausedLowBattery: Self = Self(11i32);
    pub const PausedWiFiRecommended: Self = Self(12i32);
    pub const PausedWiFiRequired: Self = Self(13i32);
    pub const PausedReadyToInstall: Self = Self(14i32);
}
impl ::core::marker::Copy for StoreQueueItemExtendedState {}
impl ::core::clone::Clone for StoreQueueItemExtendedState {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreQueueItemKind(pub i32);
impl StoreQueueItemKind {
    pub const Install: Self = Self(0i32);
    pub const Update: Self = Self(1i32);
    pub const Repair: Self = Self(2i32);
}
impl ::core::marker::Copy for StoreQueueItemKind {}
impl ::core::clone::Clone for StoreQueueItemKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreQueueItemState(pub i32);
impl StoreQueueItemState {
    pub const Active: Self = Self(0i32);
    pub const Completed: Self = Self(1i32);
    pub const Canceled: Self = Self(2i32);
    pub const Error: Self = Self(3i32);
    pub const Paused: Self = Self(4i32);
}
impl ::core::marker::Copy for StoreQueueItemState {}
impl ::core::clone::Clone for StoreQueueItemState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreQueueItemStatus = *mut ::core::ffi::c_void;
pub type StoreRateAndReviewResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreRateAndReviewStatus(pub i32);
impl StoreRateAndReviewStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const CanceledByUser: Self = Self(1i32);
    pub const NetworkError: Self = Self(2i32);
    pub const Error: Self = Self(3i32);
}
impl ::core::marker::Copy for StoreRateAndReviewStatus {}
impl ::core::clone::Clone for StoreRateAndReviewStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreSendRequestResult = *mut ::core::ffi::c_void;
pub type StoreSku = *mut ::core::ffi::c_void;
pub type StoreSubscriptionInfo = *mut ::core::ffi::c_void;
pub type StoreUninstallStorePackageResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Services_Store\"`*"]
#[repr(transparent)]
pub struct StoreUninstallStorePackageStatus(pub i32);
impl StoreUninstallStorePackageStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const CanceledByUser: Self = Self(1i32);
    pub const NetworkError: Self = Self(2i32);
    pub const UninstallNotApplicable: Self = Self(3i32);
    pub const Error: Self = Self(4i32);
}
impl ::core::marker::Copy for StoreUninstallStorePackageStatus {}
impl ::core::clone::Clone for StoreUninstallStorePackageStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StoreVideo = *mut ::core::ffi::c_void;
