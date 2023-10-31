pub type IndexedResourceCandidate = *mut ::core::ffi::c_void;
pub type IndexedResourceQualifier = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Resources_Management\"`*"]
#[repr(transparent)]
pub struct IndexedResourceType(pub i32);
impl IndexedResourceType {
    pub const String: Self = Self(0i32);
    pub const Path: Self = Self(1i32);
    pub const EmbeddedData: Self = Self(2i32);
}
impl ::core::marker::Copy for IndexedResourceType {}
impl ::core::clone::Clone for IndexedResourceType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ResourceIndexer = *mut ::core::ffi::c_void;
