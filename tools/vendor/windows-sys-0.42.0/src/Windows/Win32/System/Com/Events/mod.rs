pub type IDontSupportEventSubscription = *mut ::core::ffi::c_void;
pub type IEnumEventObject = *mut ::core::ffi::c_void;
pub type IEventClass = *mut ::core::ffi::c_void;
pub type IEventClass2 = *mut ::core::ffi::c_void;
pub type IEventControl = *mut ::core::ffi::c_void;
pub type IEventObjectChange = *mut ::core::ffi::c_void;
pub type IEventObjectChange2 = *mut ::core::ffi::c_void;
pub type IEventObjectCollection = *mut ::core::ffi::c_void;
pub type IEventProperty = *mut ::core::ffi::c_void;
pub type IEventPublisher = *mut ::core::ffi::c_void;
pub type IEventSubscription = *mut ::core::ffi::c_void;
pub type IEventSystem = *mut ::core::ffi::c_void;
pub type IFiringControl = *mut ::core::ffi::c_void;
pub type IMultiInterfaceEventControl = *mut ::core::ffi::c_void;
pub type IMultiInterfacePublisherFilter = *mut ::core::ffi::c_void;
pub type IPublisherFilter = *mut ::core::ffi::c_void;
pub const CEventClass: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3451832768, data2: 31336, data3: 4561, data4: [136, 249, 0, 128, 199, 215, 113, 191] };
pub const CEventPublisher: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2878621216, data2: 31174, data3: 4561, data4: [136, 249, 0, 128, 199, 215, 113, 191] };
pub const CEventSubscription: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1967319392, data2: 31175, data3: 4561, data4: [136, 249, 0, 128, 199, 215, 113, 191] };
pub const CEventSystem: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1309997986, data2: 11810, data3: 4561, data4: [153, 100, 0, 192, 79, 187, 179, 69] };
pub const EventObjectChange: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3495317504, data2: 40436, data3: 4561, data4: [162, 129, 0, 192, 79, 202, 10, 167] };
pub const EventObjectChange2: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3137845965, data2: 52566, data3: 20067, data4: [168, 255, 203, 240, 53, 95, 185, 244] };
#[doc = "*Required features: `\"Win32_System_Com_Events\"`*"]
pub type EOC_ChangeType = i32;
#[doc = "*Required features: `\"Win32_System_Com_Events\"`*"]
pub const EOC_NewObject: EOC_ChangeType = 0i32;
#[doc = "*Required features: `\"Win32_System_Com_Events\"`*"]
pub const EOC_ModifiedObject: EOC_ChangeType = 1i32;
#[doc = "*Required features: `\"Win32_System_Com_Events\"`*"]
pub const EOC_DeletedObject: EOC_ChangeType = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_Com_Events\"`*"]
pub struct COMEVENTSYSCHANGEINFO {
    pub cbSize: u32,
    pub changeType: EOC_ChangeType,
    pub objectId: ::windows_sys::core::BSTR,
    pub partitionId: ::windows_sys::core::BSTR,
    pub applicationId: ::windows_sys::core::BSTR,
    pub reserved: [::windows_sys::core::GUID; 10],
}
impl ::core::marker::Copy for COMEVENTSYSCHANGEINFO {}
impl ::core::clone::Clone for COMEVENTSYSCHANGEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
