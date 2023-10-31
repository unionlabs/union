#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Storage_Xps\"`*"]
    #[cfg(feature = "Win32_Storage_Xps")]
    pub fn PTCloseProvider(hprovider: super::super::super::Storage::Xps::HPTPROVIDER) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_Storage_Xps\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_Storage_Xps", feature = "Win32_System_Com"))]
    pub fn PTConvertDevModeToPrintTicket(hprovider: super::super::super::Storage::Xps::HPTPROVIDER, cbdevmode: u32, pdevmode: *const super::super::Gdi::DEVMODEA, scope: EPrintTicketScope, pprintticket: super::super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_Storage_Xps\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_Storage_Xps", feature = "Win32_System_Com"))]
    pub fn PTConvertPrintTicketToDevMode(hprovider: super::super::super::Storage::Xps::HPTPROVIDER, pprintticket: super::super::super::System::Com::IStream, basedevmodetype: EDefaultDevmodeType, scope: EPrintTicketScope, pcbdevmode: *mut u32, ppdevmode: *mut *mut super::super::Gdi::DEVMODEA, pbstrerrormessage: *mut super::super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Foundation\"`, `\"Win32_Storage_Xps\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Storage_Xps", feature = "Win32_System_Com"))]
    pub fn PTGetPrintCapabilities(hprovider: super::super::super::Storage::Xps::HPTPROVIDER, pprintticket: super::super::super::System::Com::IStream, pcapabilities: super::super::super::System::Com::IStream, pbstrerrormessage: *mut super::super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Foundation\"`, `\"Win32_Storage_Xps\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Storage_Xps", feature = "Win32_System_Com"))]
    pub fn PTGetPrintDeviceCapabilities(hprovider: super::super::super::Storage::Xps::HPTPROVIDER, pprintticket: super::super::super::System::Com::IStream, pdevicecapabilities: super::super::super::System::Com::IStream, pbstrerrormessage: *mut super::super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Foundation\"`, `\"Win32_Storage_Xps\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Storage_Xps", feature = "Win32_System_Com"))]
    pub fn PTGetPrintDeviceResources(hprovider: super::super::super::Storage::Xps::HPTPROVIDER, pszlocalename: ::windows_sys::core::PCWSTR, pprintticket: super::super::super::System::Com::IStream, pdeviceresources: super::super::super::System::Com::IStream, pbstrerrormessage: *mut super::super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Foundation\"`, `\"Win32_Storage_Xps\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Storage_Xps", feature = "Win32_System_Com"))]
    pub fn PTMergeAndValidatePrintTicket(hprovider: super::super::super::Storage::Xps::HPTPROVIDER, pbaseticket: super::super::super::System::Com::IStream, pdeltaticket: super::super::super::System::Com::IStream, scope: EPrintTicketScope, presultticket: super::super::super::System::Com::IStream, pbstrerrormessage: *mut super::super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Storage_Xps\"`*"]
    #[cfg(feature = "Win32_Storage_Xps")]
    pub fn PTOpenProvider(pszprintername: ::windows_sys::core::PCWSTR, dwversion: u32, phprovider: *mut super::super::super::Storage::Xps::HPTPROVIDER) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`, `\"Win32_Storage_Xps\"`*"]
    #[cfg(feature = "Win32_Storage_Xps")]
    pub fn PTOpenProviderEx(pszprintername: ::windows_sys::core::PCWSTR, dwmaxversion: u32, dwprefversion: u32, phprovider: *mut super::super::super::Storage::Xps::HPTPROVIDER, pusedversion: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
    pub fn PTQuerySchemaVersionSupport(pszprintername: ::windows_sys::core::PCWSTR, pmaxversion: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
    pub fn PTReleaseMemory(pbuffer: *const ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
}
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub type EDefaultDevmodeType = i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const kUserDefaultDevmode: EDefaultDevmodeType = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const kPrinterDefaultDevmode: EDefaultDevmodeType = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub type EPrintTicketScope = i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const kPTPageScope: EPrintTicketScope = 0i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const kPTDocumentScope: EPrintTicketScope = 1i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const kPTJobScope: EPrintTicketScope = 2i32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const E_DELTA_PRINTTICKET_FORMAT: u32 = 2147745797u32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const E_PRINTCAPABILITIES_FORMAT: u32 = 2147745796u32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const E_PRINTDEVICECAPABILITIES_FORMAT: u32 = 2147745798u32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const E_PRINTTICKET_FORMAT: u32 = 2147745795u32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const PRINTTICKET_ISTREAM_APIS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const S_PT_CONFLICT_RESOLVED: u32 = 262146u32;
#[doc = "*Required features: `\"Win32_Graphics_Printing_PrintTicket\"`*"]
pub const S_PT_NO_CONFLICT: u32 = 262145u32;
