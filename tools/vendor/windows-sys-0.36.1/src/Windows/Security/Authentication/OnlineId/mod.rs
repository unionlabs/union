#[doc = "*Required features: `\"Security_Authentication_OnlineId\"`*"]
#[repr(transparent)]
pub struct CredentialPromptType(pub i32);
impl CredentialPromptType {
    pub const PromptIfNeeded: Self = Self(0i32);
    pub const RetypeCredentials: Self = Self(1i32);
    pub const DoNotPrompt: Self = Self(2i32);
}
impl ::core::marker::Copy for CredentialPromptType {}
impl ::core::clone::Clone for CredentialPromptType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type OnlineIdAuthenticator = *mut ::core::ffi::c_void;
pub type OnlineIdServiceTicket = *mut ::core::ffi::c_void;
pub type OnlineIdServiceTicketRequest = *mut ::core::ffi::c_void;
pub type OnlineIdSystemAuthenticatorForUser = *mut ::core::ffi::c_void;
pub type OnlineIdSystemIdentity = *mut ::core::ffi::c_void;
pub type OnlineIdSystemTicketResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Security_Authentication_OnlineId\"`*"]
#[repr(transparent)]
pub struct OnlineIdSystemTicketStatus(pub i32);
impl OnlineIdSystemTicketStatus {
    pub const Success: Self = Self(0i32);
    pub const Error: Self = Self(1i32);
    pub const ServiceConnectionError: Self = Self(2i32);
}
impl ::core::marker::Copy for OnlineIdSystemTicketStatus {}
impl ::core::clone::Clone for OnlineIdSystemTicketStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type SignOutUserOperation = *mut ::core::ffi::c_void;
pub type UserAuthenticationOperation = *mut ::core::ffi::c_void;
pub type UserIdentity = *mut ::core::ffi::c_void;
