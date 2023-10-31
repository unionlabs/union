pub type GattCharacteristic = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattCharacteristicProperties(pub u32);
impl GattCharacteristicProperties {
    pub const None: Self = Self(0u32);
    pub const Broadcast: Self = Self(1u32);
    pub const Read: Self = Self(2u32);
    pub const WriteWithoutResponse: Self = Self(4u32);
    pub const Write: Self = Self(8u32);
    pub const Notify: Self = Self(16u32);
    pub const Indicate: Self = Self(32u32);
    pub const AuthenticatedSignedWrites: Self = Self(64u32);
    pub const ExtendedProperties: Self = Self(128u32);
    pub const ReliableWrites: Self = Self(256u32);
    pub const WritableAuxiliaries: Self = Self(512u32);
}
impl ::core::marker::Copy for GattCharacteristicProperties {}
impl ::core::clone::Clone for GattCharacteristicProperties {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattCharacteristicsResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattClientCharacteristicConfigurationDescriptorValue(pub i32);
impl GattClientCharacteristicConfigurationDescriptorValue {
    pub const None: Self = Self(0i32);
    pub const Notify: Self = Self(1i32);
    pub const Indicate: Self = Self(2i32);
}
impl ::core::marker::Copy for GattClientCharacteristicConfigurationDescriptorValue {}
impl ::core::clone::Clone for GattClientCharacteristicConfigurationDescriptorValue {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattClientNotificationResult = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattCommunicationStatus(pub i32);
impl GattCommunicationStatus {
    pub const Success: Self = Self(0i32);
    pub const Unreachable: Self = Self(1i32);
    pub const ProtocolError: Self = Self(2i32);
    pub const AccessDenied: Self = Self(3i32);
}
impl ::core::marker::Copy for GattCommunicationStatus {}
impl ::core::clone::Clone for GattCommunicationStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattDescriptor = *mut ::core::ffi::c_void;
pub type GattDescriptorsResult = *mut ::core::ffi::c_void;
pub type GattDeviceService = *mut ::core::ffi::c_void;
pub type GattDeviceServicesResult = *mut ::core::ffi::c_void;
pub type GattLocalCharacteristic = *mut ::core::ffi::c_void;
pub type GattLocalCharacteristicParameters = *mut ::core::ffi::c_void;
pub type GattLocalCharacteristicResult = *mut ::core::ffi::c_void;
pub type GattLocalDescriptor = *mut ::core::ffi::c_void;
pub type GattLocalDescriptorParameters = *mut ::core::ffi::c_void;
pub type GattLocalDescriptorResult = *mut ::core::ffi::c_void;
pub type GattLocalService = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattOpenStatus(pub i32);
impl GattOpenStatus {
    pub const Unspecified: Self = Self(0i32);
    pub const Success: Self = Self(1i32);
    pub const AlreadyOpened: Self = Self(2i32);
    pub const NotFound: Self = Self(3i32);
    pub const SharingViolation: Self = Self(4i32);
    pub const AccessDenied: Self = Self(5i32);
}
impl ::core::marker::Copy for GattOpenStatus {}
impl ::core::clone::Clone for GattOpenStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattPresentationFormat = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattProtectionLevel(pub i32);
impl GattProtectionLevel {
    pub const Plain: Self = Self(0i32);
    pub const AuthenticationRequired: Self = Self(1i32);
    pub const EncryptionRequired: Self = Self(2i32);
    pub const EncryptionAndAuthenticationRequired: Self = Self(3i32);
}
impl ::core::marker::Copy for GattProtectionLevel {}
impl ::core::clone::Clone for GattProtectionLevel {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattReadClientCharacteristicConfigurationDescriptorResult = *mut ::core::ffi::c_void;
pub type GattReadRequest = *mut ::core::ffi::c_void;
pub type GattReadRequestedEventArgs = *mut ::core::ffi::c_void;
pub type GattReadResult = *mut ::core::ffi::c_void;
pub type GattReliableWriteTransaction = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattRequestState(pub i32);
impl GattRequestState {
    pub const Pending: Self = Self(0i32);
    pub const Completed: Self = Self(1i32);
    pub const Canceled: Self = Self(2i32);
}
impl ::core::marker::Copy for GattRequestState {}
impl ::core::clone::Clone for GattRequestState {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattRequestStateChangedEventArgs = *mut ::core::ffi::c_void;
pub type GattServiceProvider = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattServiceProviderAdvertisementStatus(pub i32);
impl GattServiceProviderAdvertisementStatus {
    pub const Created: Self = Self(0i32);
    pub const Stopped: Self = Self(1i32);
    pub const Started: Self = Self(2i32);
    pub const Aborted: Self = Self(3i32);
    pub const StartedWithoutAllAdvertisementData: Self = Self(4i32);
}
impl ::core::marker::Copy for GattServiceProviderAdvertisementStatus {}
impl ::core::clone::Clone for GattServiceProviderAdvertisementStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattServiceProviderAdvertisementStatusChangedEventArgs = *mut ::core::ffi::c_void;
pub type GattServiceProviderAdvertisingParameters = *mut ::core::ffi::c_void;
pub type GattServiceProviderResult = *mut ::core::ffi::c_void;
pub type GattSession = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattSessionStatus(pub i32);
impl GattSessionStatus {
    pub const Closed: Self = Self(0i32);
    pub const Active: Self = Self(1i32);
}
impl ::core::marker::Copy for GattSessionStatus {}
impl ::core::clone::Clone for GattSessionStatus {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattSessionStatusChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattSharingMode(pub i32);
impl GattSharingMode {
    pub const Unspecified: Self = Self(0i32);
    pub const Exclusive: Self = Self(1i32);
    pub const SharedReadOnly: Self = Self(2i32);
    pub const SharedReadAndWrite: Self = Self(3i32);
}
impl ::core::marker::Copy for GattSharingMode {}
impl ::core::clone::Clone for GattSharingMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattSubscribedClient = *mut ::core::ffi::c_void;
pub type GattValueChangedEventArgs = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Bluetooth_GenericAttributeProfile\"`*"]
#[repr(transparent)]
pub struct GattWriteOption(pub i32);
impl GattWriteOption {
    pub const WriteWithResponse: Self = Self(0i32);
    pub const WriteWithoutResponse: Self = Self(1i32);
}
impl ::core::marker::Copy for GattWriteOption {}
impl ::core::clone::Clone for GattWriteOption {
    fn clone(&self) -> Self {
        *self
    }
}
pub type GattWriteRequest = *mut ::core::ffi::c_void;
pub type GattWriteRequestedEventArgs = *mut ::core::ffi::c_void;
pub type GattWriteResult = *mut ::core::ffi::c_void;
