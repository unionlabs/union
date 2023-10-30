pub type UsbBulkInEndpointDescriptor = *mut ::core::ffi::c_void;
pub type UsbBulkInPipe = *mut ::core::ffi::c_void;
pub type UsbBulkOutEndpointDescriptor = *mut ::core::ffi::c_void;
pub type UsbBulkOutPipe = *mut ::core::ffi::c_void;
pub type UsbConfiguration = *mut ::core::ffi::c_void;
pub type UsbConfigurationDescriptor = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Usb\"`*"]
#[repr(transparent)]
pub struct UsbControlRecipient(pub i32);
impl UsbControlRecipient {
    pub const Device: Self = Self(0i32);
    pub const SpecifiedInterface: Self = Self(1i32);
    pub const Endpoint: Self = Self(2i32);
    pub const Other: Self = Self(3i32);
    pub const DefaultInterface: Self = Self(4i32);
}
impl ::core::marker::Copy for UsbControlRecipient {}
impl ::core::clone::Clone for UsbControlRecipient {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UsbControlRequestType = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Usb\"`*"]
#[repr(transparent)]
pub struct UsbControlTransferType(pub i32);
impl UsbControlTransferType {
    pub const Standard: Self = Self(0i32);
    pub const Class: Self = Self(1i32);
    pub const Vendor: Self = Self(2i32);
}
impl ::core::marker::Copy for UsbControlTransferType {}
impl ::core::clone::Clone for UsbControlTransferType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UsbDescriptor = *mut ::core::ffi::c_void;
pub type UsbDevice = *mut ::core::ffi::c_void;
pub type UsbDeviceClass = *mut ::core::ffi::c_void;
pub type UsbDeviceClasses = *mut ::core::ffi::c_void;
pub type UsbDeviceDescriptor = *mut ::core::ffi::c_void;
pub type UsbEndpointDescriptor = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Usb\"`*"]
#[repr(transparent)]
pub struct UsbEndpointType(pub i32);
impl UsbEndpointType {
    pub const Control: Self = Self(0i32);
    pub const Isochronous: Self = Self(1i32);
    pub const Bulk: Self = Self(2i32);
    pub const Interrupt: Self = Self(3i32);
}
impl ::core::marker::Copy for UsbEndpointType {}
impl ::core::clone::Clone for UsbEndpointType {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UsbInterface = *mut ::core::ffi::c_void;
pub type UsbInterfaceDescriptor = *mut ::core::ffi::c_void;
pub type UsbInterfaceSetting = *mut ::core::ffi::c_void;
pub type UsbInterruptInEndpointDescriptor = *mut ::core::ffi::c_void;
pub type UsbInterruptInEventArgs = *mut ::core::ffi::c_void;
pub type UsbInterruptInPipe = *mut ::core::ffi::c_void;
pub type UsbInterruptOutEndpointDescriptor = *mut ::core::ffi::c_void;
pub type UsbInterruptOutPipe = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Usb\"`*"]
#[repr(transparent)]
pub struct UsbReadOptions(pub u32);
impl UsbReadOptions {
    pub const None: Self = Self(0u32);
    pub const AutoClearStall: Self = Self(1u32);
    pub const OverrideAutomaticBufferManagement: Self = Self(2u32);
    pub const IgnoreShortPacket: Self = Self(4u32);
    pub const AllowPartialReads: Self = Self(8u32);
}
impl ::core::marker::Copy for UsbReadOptions {}
impl ::core::clone::Clone for UsbReadOptions {
    fn clone(&self) -> Self {
        *self
    }
}
pub type UsbSetupPacket = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Devices_Usb\"`*"]
#[repr(transparent)]
pub struct UsbTransferDirection(pub i32);
impl UsbTransferDirection {
    pub const Out: Self = Self(0i32);
    pub const In: Self = Self(1i32);
}
impl ::core::marker::Copy for UsbTransferDirection {}
impl ::core::clone::Clone for UsbTransferDirection {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Devices_Usb\"`*"]
#[repr(transparent)]
pub struct UsbWriteOptions(pub u32);
impl UsbWriteOptions {
    pub const None: Self = Self(0u32);
    pub const AutoClearStall: Self = Self(1u32);
    pub const ShortPacketTerminate: Self = Self(2u32);
}
impl ::core::marker::Copy for UsbWriteOptions {}
impl ::core::clone::Clone for UsbWriteOptions {
    fn clone(&self) -> Self {
        *self
    }
}
