#[cfg(feature = "Phone_PersonalInformation_Provisioning")]
pub mod Provisioning;
pub type ContactAddress = *mut ::core::ffi::c_void;
pub type ContactChangeRecord = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Phone_PersonalInformation\"`*"]
#[repr(transparent)]
pub struct ContactChangeType(pub i32);
impl ContactChangeType {
    pub const Created: Self = Self(0i32);
    pub const Modified: Self = Self(1i32);
    pub const Deleted: Self = Self(2i32);
}
impl ::core::marker::Copy for ContactChangeType {}
impl ::core::clone::Clone for ContactChangeType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ContactInformation = *mut ::core::ffi::c_void;
pub type ContactQueryOptions = *mut ::core::ffi::c_void;
pub type ContactQueryResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Phone_PersonalInformation\"`*"]
#[repr(transparent)]
pub struct ContactQueryResultOrdering(pub i32);
impl ContactQueryResultOrdering {
    pub const SystemDefault: Self = Self(0i32);
    pub const GivenNameFamilyName: Self = Self(1i32);
    pub const FamilyNameGivenName: Self = Self(2i32);
}
impl ::core::marker::Copy for ContactQueryResultOrdering {}
impl ::core::clone::Clone for ContactQueryResultOrdering {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ContactStore = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Phone_PersonalInformation\"`*"]
#[repr(transparent)]
pub struct ContactStoreApplicationAccessMode(pub i32);
impl ContactStoreApplicationAccessMode {
    pub const LimitedReadOnly: Self = Self(0i32);
    pub const ReadOnly: Self = Self(1i32);
}
impl ::core::marker::Copy for ContactStoreApplicationAccessMode {}
impl ::core::clone::Clone for ContactStoreApplicationAccessMode {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Phone_PersonalInformation\"`*"]
#[repr(transparent)]
pub struct ContactStoreSystemAccessMode(pub i32);
impl ContactStoreSystemAccessMode {
    pub const ReadOnly: Self = Self(0i32);
    pub const ReadWrite: Self = Self(1i32);
}
impl ::core::marker::Copy for ContactStoreSystemAccessMode {}
impl ::core::clone::Clone for ContactStoreSystemAccessMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type IContactInformation = *mut ::core::ffi::c_void;
pub type IContactInformation2 = *mut ::core::ffi::c_void;
pub type StoredContact = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Phone_PersonalInformation\"`*"]
#[repr(transparent)]
pub struct VCardFormat(pub i32);
impl VCardFormat {
    pub const Version2_1: Self = Self(0i32);
    pub const Version3: Self = Self(1i32);
}
impl ::core::marker::Copy for VCardFormat {}
impl ::core::clone::Clone for VCardFormat {
    fn clone(&self) -> Self {
        *self
    }
}
