#[doc = "*Required features: `\"ApplicationModel_Contacts_Provider\"`*"]
#[repr(transparent)]
pub struct AddContactResult(pub i32);
impl AddContactResult {
    pub const Added: Self = Self(0i32);
    pub const AlreadyAdded: Self = Self(1i32);
    pub const Unavailable: Self = Self(2i32);
}
impl ::core::marker::Copy for AddContactResult {}
impl ::core::clone::Clone for AddContactResult {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ContactPickerUI = *mut ::core::ffi::c_void;
pub type ContactRemovedEventArgs = *mut ::core::ffi::c_void;
