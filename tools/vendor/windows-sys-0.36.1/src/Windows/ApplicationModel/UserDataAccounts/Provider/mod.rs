pub type IUserDataAccountProviderOperation = *mut ::core::ffi::c_void;
pub type UserDataAccountPartnerAccountInfo = *mut ::core::ffi::c_void;
pub type UserDataAccountProviderAddAccountOperation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_UserDataAccounts_Provider\"`*"]
#[repr(transparent)]
pub struct UserDataAccountProviderOperationKind(pub i32);
impl UserDataAccountProviderOperationKind {
    pub const AddAccount: Self = Self(0i32);
    pub const Settings: Self = Self(1i32);
    pub const ResolveErrors: Self = Self(2i32);
}
impl ::core::marker::Copy for UserDataAccountProviderOperationKind {}
impl ::core::clone::Clone for UserDataAccountProviderOperationKind {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"ApplicationModel_UserDataAccounts_Provider\"`*"]
#[repr(transparent)]
pub struct UserDataAccountProviderPartnerAccountKind(pub i32);
impl UserDataAccountProviderPartnerAccountKind {
    pub const Exchange: Self = Self(0i32);
    pub const PopOrImap: Self = Self(1i32);
}
impl ::core::marker::Copy for UserDataAccountProviderPartnerAccountKind {}
impl ::core::clone::Clone for UserDataAccountProviderPartnerAccountKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UserDataAccountProviderResolveErrorsOperation = *mut ::core::ffi::c_void;
pub type UserDataAccountProviderSettingsOperation = *mut ::core::ffi::c_void;
