#[doc = "*Required features: `\"Storage_AccessCache\"`*"]
#[repr(transparent)]
pub struct AccessCacheOptions(pub u32);
impl AccessCacheOptions {
    pub const None: Self = Self(0u32);
    pub const DisallowUserInput: Self = Self(1u32);
    pub const FastLocationsOnly: Self = Self(2u32);
    pub const UseReadOnlyCachedCopy: Self = Self(4u32);
    pub const SuppressAccessTimeUpdate: Self = Self(8u32);
}
impl ::core::marker::Copy for AccessCacheOptions {}
impl ::core::clone::Clone for AccessCacheOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Storage_AccessCache\"`*"]
pub struct AccessListEntry {
    pub Token: ::windows_sys::core::HSTRING,
    pub Metadata: ::windows_sys::core::HSTRING,
}
impl ::core::marker::Copy for AccessListEntry {}
impl ::core::clone::Clone for AccessListEntry {
    fn clone(&self) -> Self {
        *self
    }
}
pub type AccessListEntryView = *mut ::core::ffi::c_void;
pub type IStorageItemAccessList = *mut ::core::ffi::c_void;
pub type ItemRemovedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Storage_AccessCache\"`*"]
#[repr(transparent)]
pub struct RecentStorageItemVisibility(pub i32);
impl RecentStorageItemVisibility {
    pub const AppOnly: Self = Self(0i32);
    pub const AppAndSystem: Self = Self(1i32);
}
impl ::core::marker::Copy for RecentStorageItemVisibility {}
impl ::core::clone::Clone for RecentStorageItemVisibility {
    fn clone(&self) -> Self {
        *self
    }
}
pub type StorageItemAccessList = *mut ::core::ffi::c_void;
pub type StorageItemMostRecentlyUsedList = *mut ::core::ffi::c_void;
