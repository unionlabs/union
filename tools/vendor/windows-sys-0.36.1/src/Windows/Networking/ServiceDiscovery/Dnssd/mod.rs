pub type DnssdRegistrationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_ServiceDiscovery_Dnssd\"`*"]
#[repr(transparent)]
pub struct DnssdRegistrationStatus(pub i32);
impl DnssdRegistrationStatus {
    pub const Success: Self = Self(0i32);
    pub const InvalidServiceName: Self = Self(1i32);
    pub const ServerError: Self = Self(2i32);
    pub const SecurityError: Self = Self(3i32);
}
impl ::core::marker::Copy for DnssdRegistrationStatus {}
impl ::core::clone::Clone for DnssdRegistrationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DnssdServiceInstance = *mut ::core::ffi::c_void;
pub type DnssdServiceInstanceCollection = *mut ::core::ffi::c_void;
pub type DnssdServiceWatcher = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Networking_ServiceDiscovery_Dnssd\"`*"]
#[repr(transparent)]
pub struct DnssdServiceWatcherStatus(pub i32);
impl DnssdServiceWatcherStatus {
    pub const Created: Self = Self(0i32);
    pub const Started: Self = Self(1i32);
    pub const EnumerationCompleted: Self = Self(2i32);
    pub const Stopping: Self = Self(3i32);
    pub const Stopped: Self = Self(4i32);
    pub const Aborted: Self = Self(5i32);
}
impl ::core::marker::Copy for DnssdServiceWatcherStatus {}
impl ::core::clone::Clone for DnssdServiceWatcherStatus {
    fn clone(&self) -> Self {
        *self
    }
}
