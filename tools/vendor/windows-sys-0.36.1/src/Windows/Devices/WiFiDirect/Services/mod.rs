pub type WiFiDirectService = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceAdvertisementStatus(pub i32);
impl WiFiDirectServiceAdvertisementStatus {
    pub const Created: Self = Self(0i32);
    pub const Started: Self = Self(1i32);
    pub const Stopped: Self = Self(2i32);
    pub const Aborted: Self = Self(3i32);
}
impl ::core::marker::Copy for WiFiDirectServiceAdvertisementStatus {}
impl ::core::clone::Clone for WiFiDirectServiceAdvertisementStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WiFiDirectServiceAdvertiser = *mut ::core::ffi::c_void;
pub type WiFiDirectServiceAutoAcceptSessionConnectedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceConfigurationMethod(pub i32);
impl WiFiDirectServiceConfigurationMethod {
    pub const Default: Self = Self(0i32);
    pub const PinDisplay: Self = Self(1i32);
    pub const PinEntry: Self = Self(2i32);
}
impl ::core::marker::Copy for WiFiDirectServiceConfigurationMethod {}
impl ::core::clone::Clone for WiFiDirectServiceConfigurationMethod {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceError(pub i32);
impl WiFiDirectServiceError {
    pub const Success: Self = Self(0i32);
    pub const RadioNotAvailable: Self = Self(1i32);
    pub const ResourceInUse: Self = Self(2i32);
    pub const UnsupportedHardware: Self = Self(3i32);
    pub const NoHardware: Self = Self(4i32);
}
impl ::core::marker::Copy for WiFiDirectServiceError {}
impl ::core::clone::Clone for WiFiDirectServiceError {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceIPProtocol(pub i32);
impl WiFiDirectServiceIPProtocol {
    pub const Tcp: Self = Self(6i32);
    pub const Udp: Self = Self(17i32);
}
impl ::core::marker::Copy for WiFiDirectServiceIPProtocol {}
impl ::core::clone::Clone for WiFiDirectServiceIPProtocol {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WiFiDirectServiceProvisioningInfo = *mut ::core::ffi::c_void;
pub type WiFiDirectServiceRemotePortAddedEventArgs = *mut ::core::ffi::c_void;
pub type WiFiDirectServiceSession = *mut ::core::ffi::c_void;
pub type WiFiDirectServiceSessionDeferredEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceSessionErrorStatus(pub i32);
impl WiFiDirectServiceSessionErrorStatus {
    pub const Ok: Self = Self(0i32);
    pub const Disassociated: Self = Self(1i32);
    pub const LocalClose: Self = Self(2i32);
    pub const RemoteClose: Self = Self(3i32);
    pub const SystemFailure: Self = Self(4i32);
    pub const NoResponseFromRemote: Self = Self(5i32);
}
impl ::core::marker::Copy for WiFiDirectServiceSessionErrorStatus {}
impl ::core::clone::Clone for WiFiDirectServiceSessionErrorStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type WiFiDirectServiceSessionRequest = *mut ::core::ffi::c_void;
pub type WiFiDirectServiceSessionRequestedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceSessionStatus(pub i32);
impl WiFiDirectServiceSessionStatus {
    pub const Closed: Self = Self(0i32);
    pub const Initiated: Self = Self(1i32);
    pub const Requested: Self = Self(2i32);
    pub const Open: Self = Self(3i32);
}
impl ::core::marker::Copy for WiFiDirectServiceSessionStatus {}
impl ::core::clone::Clone for WiFiDirectServiceSessionStatus {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_WiFiDirect_Services\"`*"]
#[repr(transparent)]
pub struct WiFiDirectServiceStatus(pub i32);
impl WiFiDirectServiceStatus {
    pub const Available: Self = Self(0i32);
    pub const Busy: Self = Self(1i32);
    pub const Custom: Self = Self(2i32);
}
impl ::core::marker::Copy for WiFiDirectServiceStatus {}
impl ::core::clone::Clone for WiFiDirectServiceStatus {
    fn clone(&self) -> Self {
        *self
    }
}
