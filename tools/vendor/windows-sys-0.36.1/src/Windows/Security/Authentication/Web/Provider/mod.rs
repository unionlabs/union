pub type IWebAccountProviderBaseReportOperation = *mut ::core::ffi::c_void;
pub type IWebAccountProviderOperation = *mut ::core::ffi::c_void;
pub type IWebAccountProviderSilentReportOperation = *mut ::core::ffi::c_void;
pub type IWebAccountProviderTokenObjects = *mut ::core::ffi::c_void;
pub type IWebAccountProviderTokenObjects2 = *mut ::core::ffi::c_void;
pub type IWebAccountProviderTokenOperation = *mut ::core::ffi::c_void;
pub type IWebAccountProviderUIReportOperation = *mut ::core::ffi::c_void;
pub type WebAccountClientView = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Authentication_Web_Provider\"`*"]
#[repr(transparent)]
pub struct WebAccountClientViewType(pub i32);
impl WebAccountClientViewType {
    pub const IdOnly: Self = Self(0i32);
    pub const IdAndProperties: Self = Self(1i32);
}
impl ::core::marker::Copy for WebAccountClientViewType {}
impl ::core::clone::Clone for WebAccountClientViewType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebAccountProviderAddAccountOperation = *mut ::core::ffi::c_void;
pub type WebAccountProviderDeleteAccountOperation = *mut ::core::ffi::c_void;
pub type WebAccountProviderGetTokenSilentOperation = *mut ::core::ffi::c_void;
pub type WebAccountProviderManageAccountOperation = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Authentication_Web_Provider\"`*"]
#[repr(transparent)]
pub struct WebAccountProviderOperationKind(pub i32);
impl WebAccountProviderOperationKind {
    pub const RequestToken: Self = Self(0i32);
    pub const GetTokenSilently: Self = Self(1i32);
    pub const AddAccount: Self = Self(2i32);
    pub const ManageAccount: Self = Self(3i32);
    pub const DeleteAccount: Self = Self(4i32);
    pub const RetrieveCookies: Self = Self(5i32);
    pub const SignOutAccount: Self = Self(6i32);
}
impl ::core::marker::Copy for WebAccountProviderOperationKind {}
impl ::core::clone::Clone for WebAccountProviderOperationKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebAccountProviderRequestTokenOperation = *mut ::core::ffi::c_void;
pub type WebAccountProviderRetrieveCookiesOperation = *mut ::core::ffi::c_void;
pub type WebAccountProviderSignOutAccountOperation = *mut ::core::ffi::c_void;
pub type WebAccountProviderTriggerDetails = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Authentication_Web_Provider\"`*"]
#[repr(transparent)]
pub struct WebAccountScope(pub i32);
impl WebAccountScope {
    pub const PerUser: Self = Self(0i32);
    pub const PerApplication: Self = Self(1i32);
}
impl ::core::marker::Copy for WebAccountScope {}
impl ::core::clone::Clone for WebAccountScope {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Security_Authentication_Web_Provider\"`*"]
#[repr(transparent)]
pub struct WebAccountSelectionOptions(pub u32);
impl WebAccountSelectionOptions {
    pub const Default: Self = Self(0u32);
    pub const New: Self = Self(1u32);
}
impl ::core::marker::Copy for WebAccountSelectionOptions {}
impl ::core::clone::Clone for WebAccountSelectionOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WebProviderTokenRequest = *mut ::core::ffi::c_void;
pub type WebProviderTokenResponse = *mut ::core::ffi::c_void;
