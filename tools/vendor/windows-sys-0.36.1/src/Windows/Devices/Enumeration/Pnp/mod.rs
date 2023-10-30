pub type PnpObject = *mut ::core::ffi::c_void;
pub type PnpObjectCollection = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Enumeration_Pnp\"`*"]
#[repr(transparent)]
pub struct PnpObjectType(pub i32);
impl PnpObjectType {
    pub const Unknown: Self = Self(0i32);
    pub const DeviceInterface: Self = Self(1i32);
    pub const DeviceContainer: Self = Self(2i32);
    pub const Device: Self = Self(3i32);
    pub const DeviceInterfaceClass: Self = Self(4i32);
    pub const AssociationEndpoint: Self = Self(5i32);
    pub const AssociationEndpointContainer: Self = Self(6i32);
    pub const AssociationEndpointService: Self = Self(7i32);
    pub const DevicePanel: Self = Self(8i32);
}
impl ::core::marker::Copy for PnpObjectType {}
impl ::core::clone::Clone for PnpObjectType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type PnpObjectUpdate = *mut ::core::ffi::c_void;
pub type PnpObjectWatcher = *mut ::core::ffi::c_void;
