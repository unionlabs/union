#[cfg(feature = "ApplicationModel_Store_LicenseManagement")]
pub mod LicenseManagement;
#[cfg(feature = "ApplicationModel_Store_Preview")]
pub mod Preview;
#[doc = "*Required features: `\"ApplicationModel_Store\"`*"]
#[repr(transparent)]
pub struct FulfillmentResult(pub i32);
impl FulfillmentResult {
    pub const Succeeded: Self = Self(0i32);
    pub const NothingToFulfill: Self = Self(1i32);
    pub const PurchasePending: Self = Self(2i32);
    pub const PurchaseReverted: Self = Self(3i32);
    pub const ServerError: Self = Self(4i32);
}
impl ::core::marker::Copy for FulfillmentResult {}
impl ::core::clone::Clone for FulfillmentResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type LicenseChangedEventHandler = *mut ::core::ffi::c_void;
pub type LicenseInformation = *mut ::core::ffi::c_void;
pub type ListingInformation = *mut ::core::ffi::c_void;
pub type ProductLicense = *mut ::core::ffi::c_void;
pub type ProductListing = *mut ::core::ffi::c_void;
pub type ProductPurchaseDisplayProperties = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Store\"`*"]
#[repr(transparent)]
pub struct ProductPurchaseStatus(pub i32);
impl ProductPurchaseStatus {
    pub const Succeeded: Self = Self(0i32);
    pub const AlreadyPurchased: Self = Self(1i32);
    pub const NotFulfilled: Self = Self(2i32);
    pub const NotPurchased: Self = Self(3i32);
}
impl ::core::marker::Copy for ProductPurchaseStatus {}
impl ::core::clone::Clone for ProductPurchaseStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_Store\"`*"]
#[repr(transparent)]
pub struct ProductType(pub i32);
impl ProductType {
    pub const Unknown: Self = Self(0i32);
    pub const Durable: Self = Self(1i32);
    pub const Consumable: Self = Self(2i32);
}
impl ::core::marker::Copy for ProductType {}
impl ::core::clone::Clone for ProductType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PurchaseResults = *mut ::core::ffi::c_void;
pub type UnfulfilledConsumable = *mut ::core::ffi::c_void;
