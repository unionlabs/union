#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct APPLICATION_EVENT_DATA {
    pub cbApplicationEventData: u32,
    pub ApplicationId: ::windows_sys::core::GUID,
    pub EndpointId: ::windows_sys::core::GUID,
    pub dwEventId: u32,
    pub cbEventData: u32,
    pub bEventData: [u8; 1],
}
impl ::core::marker::Copy for APPLICATION_EVENT_DATA {}
impl ::core::clone::Clone for APPLICATION_EVENT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const CONTENT_ID_GLANCE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const CONTENT_ID_HOME: u32 = 1u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct CONTENT_MISSING_EVENT_DATA {
    pub cbContentMissingEventData: u32,
    pub ApplicationId: ::windows_sys::core::GUID,
    pub EndpointId: ::windows_sys::core::GUID,
    pub ContentId: u32,
}
impl ::core::marker::Copy for CONTENT_MISSING_EVENT_DATA {}
impl ::core::clone::Clone for CONTENT_MISSING_EVENT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct DEVICE_USER_CHANGE_EVENT_DATA {
    pub cbDeviceUserChangeEventData: u32,
    pub wszUser: u16,
}
impl ::core::marker::Copy for DEVICE_USER_CHANGE_EVENT_DATA {}
impl ::core::clone::Clone for DEVICE_USER_CHANGE_EVENT_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct EVENT_DATA_HEADER {
    pub cbEventDataHeader: u32,
    pub guidEventType: ::windows_sys::core::GUID,
    pub dwVersion: u32,
    pub cbEventDataSid: u32,
}
impl ::core::marker::Copy for EVENT_DATA_HEADER {}
impl ::core::clone::Clone for EVENT_DATA_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
pub const GUID_DEVINTERFACE_SIDESHOW: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 355358737, data2: 65209, data3: 19200, data4: [144, 244, 211, 41, 71, 174, 22, 129] };
pub type ISideShowBulkCapabilities = *mut ::core::ffi::c_void;
pub type ISideShowCapabilities = *mut ::core::ffi::c_void;
pub type ISideShowCapabilitiesCollection = *mut ::core::ffi::c_void;
pub type ISideShowContent = *mut ::core::ffi::c_void;
pub type ISideShowContentManager = *mut ::core::ffi::c_void;
pub type ISideShowEvents = *mut ::core::ffi::c_void;
pub type ISideShowKeyCollection = *mut ::core::ffi::c_void;
pub type ISideShowNotification = *mut ::core::ffi::c_void;
pub type ISideShowNotificationManager = *mut ::core::ffi::c_void;
pub type ISideShowPropVariantCollection = *mut ::core::ffi::c_void;
pub type ISideShowSession = *mut ::core::ffi::c_void;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct NEW_EVENT_DATA_AVAILABLE {
    pub cbNewEventDataAvailable: u32,
    pub dwVersion: u32,
}
impl ::core::marker::Copy for NEW_EVENT_DATA_AVAILABLE {}
impl ::core::clone::Clone for NEW_EVENT_DATA_AVAILABLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub type SCF_BUTTON_IDS = i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_MENU: SCF_BUTTON_IDS = 1i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_SELECT: SCF_BUTTON_IDS = 2i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_UP: SCF_BUTTON_IDS = 3i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_DOWN: SCF_BUTTON_IDS = 4i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_LEFT: SCF_BUTTON_IDS = 5i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_RIGHT: SCF_BUTTON_IDS = 6i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_PLAY: SCF_BUTTON_IDS = 7i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_PAUSE: SCF_BUTTON_IDS = 8i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_FASTFORWARD: SCF_BUTTON_IDS = 9i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_REWIND: SCF_BUTTON_IDS = 10i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_STOP: SCF_BUTTON_IDS = 11i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_BUTTON_BACK: SCF_BUTTON_IDS = 65280i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct SCF_CONTEXTMENU_EVENT {
    pub PreviousPage: u32,
    pub TargetPage: u32,
    pub PreviousItemId: u32,
    pub MenuPage: u32,
    pub MenuItemId: u32,
}
impl ::core::marker::Copy for SCF_CONTEXTMENU_EVENT {}
impl ::core::clone::Clone for SCF_CONTEXTMENU_EVENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct SCF_EVENT_HEADER {
    pub PreviousPage: u32,
    pub TargetPage: u32,
}
impl ::core::marker::Copy for SCF_EVENT_HEADER {}
impl ::core::clone::Clone for SCF_EVENT_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub type SCF_EVENT_IDS = i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_EVENT_NAVIGATION: SCF_EVENT_IDS = 1i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_EVENT_MENUACTION: SCF_EVENT_IDS = 2i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SCF_EVENT_CONTEXTMENU: SCF_EVENT_IDS = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct SCF_MENUACTION_EVENT {
    pub PreviousPage: u32,
    pub TargetPage: u32,
    pub Button: u32,
    pub ItemId: u32,
}
impl ::core::marker::Copy for SCF_MENUACTION_EVENT {}
impl ::core::clone::Clone for SCF_MENUACTION_EVENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub struct SCF_NAVIGATION_EVENT {
    pub PreviousPage: u32,
    pub TargetPage: u32,
    pub Button: u32,
}
impl ::core::marker::Copy for SCF_NAVIGATION_EVENT {}
impl ::core::clone::Clone for SCF_NAVIGATION_EVENT {
    fn clone(&self) -> Self {
        *self
    }
}
pub const SIDESHOW_APPLICATION_EVENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1286959866, data2: 7483, data3: 18867, data4: [161, 122, 46, 107, 255, 5, 40, 84] };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_CLIENT_AREA_HEIGHT: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 16u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_CLIENT_AREA_WIDTH: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 15u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_COLOR_DEPTH: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 5u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_COLOR_TYPE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 6u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_CURRENT_LANGUAGE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 9u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_DATA_CACHE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 7u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_DEVICE_ID: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 1u32 };
pub const SIDESHOW_CAPABILITY_DEVICE_PROPERTIES: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_SCREEN_HEIGHT: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 4u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_SCREEN_TYPE: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 2u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_SCREEN_WIDTH: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 3u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_SUPPORTED_IMAGE_FORMATS: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 14u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_SUPPORTED_LANGUAGES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 8u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub const SIDESHOW_CAPABILITY_SUPPORTED_THEMES: super::super::UI::Shell::PropertiesSystem::PROPERTYKEY = super::super::UI::Shell::PropertiesSystem::PROPERTYKEY { fmtid: ::windows_sys::core::GUID { data1: 2327611560, data2: 34171, data3: 19159, data4: [163, 90, 181, 148, 47, 73, 43, 153] }, pid: 10u32 };
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub type SIDESHOW_COLOR_TYPE = i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_COLOR_TYPE_COLOR: SIDESHOW_COLOR_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_COLOR_TYPE_GREYSCALE: SIDESHOW_COLOR_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_COLOR_TYPE_BLACK_AND_WHITE: SIDESHOW_COLOR_TYPE = 2i32;
pub const SIDESHOW_CONTENT_MISSING_EVENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1342700456, data2: 54035, data3: 17311, data4: [190, 162, 165, 2, 1, 211, 233, 168] };
pub const SIDESHOW_ENDPOINT_ICAL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1308571317, data2: 40414, data3: 20342, data4: [154, 42, 150, 67, 80, 71, 6, 61] };
pub const SIDESHOW_ENDPOINT_SIMPLE_CONTENT_FORMAT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2846176575, data2: 11595, data3: 18382, data4: [147, 238, 117, 159, 58, 125, 218, 79] };
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_EVENTID_APPLICATION_ENTER: u32 = 4294901760u32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_EVENTID_APPLICATION_EXIT: u32 = 4294901761u32;
pub const SIDESHOW_NEW_EVENT_DATA_AVAILABLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1468086356, data2: 12225, data3: 16668, data4: [165, 159, 242, 73, 39, 96, 136, 4] };
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub type SIDESHOW_SCREEN_TYPE = i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_SCREEN_TYPE_BITMAP: SIDESHOW_SCREEN_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const SIDESHOW_SCREEN_TYPE_TEXT: SIDESHOW_SCREEN_TYPE = 1i32;
pub const SIDESHOW_USER_CHANGE_REQUEST_EVENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1342793532, data2: 16253, data3: 19582, data4: [153, 113, 234, 162, 233, 31, 21, 117] };
pub const SideShowKeyCollection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3753630712, data2: 6366, data3: 18872, data4: [131, 220, 235, 199, 39, 198, 45, 148] };
pub const SideShowNotification: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 216262767, data2: 54733, data3: 17701, data4: [167, 102, 26, 186, 177, 167, 82, 245] };
pub const SideShowPropVariantCollection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3863016469, data2: 21406, data3: 18723, data4: [150, 205, 95, 9, 59, 194, 80, 205] };
pub const SideShowSession: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3791995833, data2: 63365, data3: 20130, data4: [152, 30, 196, 255, 167, 107, 188, 124] };
#[doc = "*Required features: `\"Win32_System_SideShow\"`*"]
pub const VERSION_1_WINDOWS_7: u32 = 0u32;
