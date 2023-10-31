pub type NamedResource = *mut ::core::ffi::c_void;
pub type ResourceCandidate = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Resources_Core\"`*"]
#[repr(transparent)]
pub struct ResourceCandidateKind(pub i32);
impl ResourceCandidateKind {
    pub const String: Self = Self(0i32);
    pub const File: Self = Self(1i32);
    pub const EmbeddedData: Self = Self(2i32);
}
impl ::core::marker::Copy for ResourceCandidateKind {}
impl ::core::clone::Clone for ResourceCandidateKind {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ResourceCandidateVectorView = *mut ::core::ffi::c_void;
pub type ResourceContext = *mut ::core::ffi::c_void;
pub type ResourceContextLanguagesVectorView = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"ApplicationModel_Resources_Core\"`*"]
pub struct ResourceLayoutInfo {
    pub MajorVersion: u32,
    pub MinorVersion: u32,
    pub ResourceSubtreeCount: u32,
    pub NamedResourceCount: u32,
    pub Checksum: i32,
}
impl ::core::marker::Copy for ResourceLayoutInfo {}
impl ::core::clone::Clone for ResourceLayoutInfo {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ResourceManager = *mut ::core::ffi::c_void;
pub type ResourceMap = *mut ::core::ffi::c_void;
pub type ResourceMapIterator = *mut ::core::ffi::c_void;
pub type ResourceMapMapView = *mut ::core::ffi::c_void;
pub type ResourceMapMapViewIterator = *mut ::core::ffi::c_void;
pub type ResourceQualifier = *mut ::core::ffi::c_void;
pub type ResourceQualifierMapView = *mut ::core::ffi::c_void;
pub type ResourceQualifierObservableMap = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"ApplicationModel_Resources_Core\"`*"]
#[repr(transparent)]
pub struct ResourceQualifierPersistence(pub i32);
impl ResourceQualifierPersistence {
    pub const None: Self = Self(0i32);
    pub const LocalMachine: Self = Self(1i32);
}
impl ::core::marker::Copy for ResourceQualifierPersistence {}
impl ::core::clone::Clone for ResourceQualifierPersistence {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ResourceQualifierVectorView = *mut ::core::ffi::c_void;
