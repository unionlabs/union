#[doc = "*Required features: `\"Devices_Bluetooth_Background\"`*"]
#[repr(transparent)]
pub struct BluetoothEventTriggeringMode(pub i32);
impl BluetoothEventTriggeringMode {
    pub const Serial: Self = Self(0i32);
    pub const Batch: Self = Self(1i32);
    pub const KeepLatest: Self = Self(2i32);
}
impl ::core::marker::Copy for BluetoothEventTriggeringMode {}
impl ::core::clone::Clone for BluetoothEventTriggeringMode {
    fn clone(&self) -> Self {
        *self
    }
}
pub type BluetoothLEAdvertisementPublisherTriggerDetails = *mut ::core::ffi::c_void;
pub type BluetoothLEAdvertisementWatcherTriggerDetails = *mut ::core::ffi::c_void;
pub type GattCharacteristicNotificationTriggerDetails = *mut ::core::ffi::c_void;
pub type GattServiceProviderConnection = *mut ::core::ffi::c_void;
pub type GattServiceProviderTriggerDetails = *mut ::core::ffi::c_void;
pub type RfcommConnectionTriggerDetails = *mut ::core::ffi::c_void;
pub type RfcommInboundConnectionInformation = *mut ::core::ffi::c_void;
pub type RfcommOutboundConnectionInformation = *mut ::core::ffi::c_void;
