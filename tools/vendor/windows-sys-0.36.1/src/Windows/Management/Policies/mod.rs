pub type NamedPolicyData = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Management_Policies\"`*"]
#[repr(transparent)]
pub struct NamedPolicyKind(pub i32);
impl NamedPolicyKind {
    pub const Invalid: Self = Self(0i32);
    pub const Binary: Self = Self(1i32);
    pub const Boolean: Self = Self(2i32);
    pub const Int32: Self = Self(3i32);
    pub const Int64: Self = Self(4i32);
    pub const String: Self = Self(5i32);
}
impl ::core::marker::Copy for NamedPolicyKind {}
impl ::core::clone::Clone for NamedPolicyKind {
    fn clone(&self) -> Self {
        *self
    }
}
