#[cfg(feature = "Win32_UI_Shell_Common")]
pub mod Common;
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub mod PropertiesSystem;
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn AssocCreate(clsid: ::windows_sys::core::GUID, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn AssocCreateForClasses(rgclasses: *const ASSOCIATIONELEMENT, cclasses: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn AssocGetDetailsOfPropKey(psf: IShellFolder, pidl: *const Common::ITEMIDLIST, pkey: *const PropertiesSystem::PROPERTYKEY, pv: *mut super::super::System::Com::VARIANT, pffoundpropkey: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn AssocGetPerceivedType(pszext: ::windows_sys::core::PCWSTR, ptype: *mut Common::PERCEIVED, pflag: *mut u32, ppsztype: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn AssocIsDangerous(pszassoc: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn AssocQueryKeyA(flags: u32, key: ASSOCKEY, pszassoc: ::windows_sys::core::PCSTR, pszextra: ::windows_sys::core::PCSTR, phkeyout: *mut super::super::System::Registry::HKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn AssocQueryKeyW(flags: u32, key: ASSOCKEY, pszassoc: ::windows_sys::core::PCWSTR, pszextra: ::windows_sys::core::PCWSTR, phkeyout: *mut super::super::System::Registry::HKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn AssocQueryStringA(flags: u32, str: ASSOCSTR, pszassoc: ::windows_sys::core::PCSTR, pszextra: ::windows_sys::core::PCSTR, pszout: ::windows_sys::core::PSTR, pcchout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn AssocQueryStringByKeyA(flags: u32, str: ASSOCSTR, hkassoc: super::super::System::Registry::HKEY, pszextra: ::windows_sys::core::PCSTR, pszout: ::windows_sys::core::PSTR, pcchout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn AssocQueryStringByKeyW(flags: u32, str: ASSOCSTR, hkassoc: super::super::System::Registry::HKEY, pszextra: ::windows_sys::core::PCWSTR, pszout: ::windows_sys::core::PWSTR, pcchout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn AssocQueryStringW(flags: u32, str: ASSOCSTR, pszassoc: ::windows_sys::core::PCWSTR, pszextra: ::windows_sys::core::PCWSTR, pszout: ::windows_sys::core::PWSTR, pcchout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Registry\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Registry", feature = "Win32_UI_Shell_Common"))]
    pub fn CDefFolderMenu_Create2(pidlfolder: *const Common::ITEMIDLIST, hwnd: super::super::Foundation::HWND, cidl: u32, apidl: *const *const Common::ITEMIDLIST, psf: IShellFolder, pfn: LPFNDFMCALLBACK, nkeys: u32, ahkeys: *const super::super::System::Registry::HKEY, ppcm: *mut IContextMenu) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn CIDLData_CreateFromIDArray(pidlfolder: *const Common::ITEMIDLIST, cidl: u32, apidl: *const *const Common::ITEMIDLIST, ppdtobj: *mut super::super::System::Com::IDataObject) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ChrCmpIA(w1: u16, w2: u16) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ChrCmpIW(w1: u16, w2: u16) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ColorAdjustLuma(clrrgb: u32, n: i32, fscale: super::super::Foundation::BOOL) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn ColorHLSToRGB(whue: u16, wluminance: u16, wsaturation: u16) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn ColorRGBToHLS(clrrgb: u32, pwhue: *mut u16, pwluminance: *mut u16, pwsaturation: *mut u16);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn CommandLineToArgvW(lpcmdline: ::windows_sys::core::PCWSTR, pnumargs: *mut i32) -> *mut ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn ConnectToConnectionPoint(punk: ::windows_sys::core::IUnknown, riidevent: *const ::windows_sys::core::GUID, fconnect: super::super::Foundation::BOOL, punktarget: ::windows_sys::core::IUnknown, pdwcookie: *mut u32, ppcpout: *mut super::super::System::Com::IConnectionPoint) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn CreateProfile(pszusersid: ::windows_sys::core::PCWSTR, pszusername: ::windows_sys::core::PCWSTR, pszprofilepath: ::windows_sys::core::PWSTR, cchprofilepath: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DAD_AutoScroll(hwnd: super::super::Foundation::HWND, pad: *mut AUTO_SCROLL_DATA, pptnow: *const super::super::Foundation::POINT) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DAD_DragEnterEx(hwndtarget: super::super::Foundation::HWND, ptstart: super::super::Foundation::POINT) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn DAD_DragEnterEx2(hwndtarget: super::super::Foundation::HWND, ptstart: super::super::Foundation::POINT, pdtobject: super::super::System::Com::IDataObject) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DAD_DragLeave() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DAD_DragMove(pt: super::super::Foundation::POINT) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Controls\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
    pub fn DAD_SetDragImage(him: super::Controls::HIMAGELIST, pptoffset: *mut super::super::Foundation::POINT) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DAD_ShowDragImage(fshow: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DefSubclassProc(hwnd: super::super::Foundation::HWND, umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DeleteProfileA(lpsidstring: ::windows_sys::core::PCSTR, lpprofilepath: ::windows_sys::core::PCSTR, lpcomputername: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DeleteProfileW(lpsidstring: ::windows_sys::core::PCWSTR, lpprofilepath: ::windows_sys::core::PCWSTR, lpcomputername: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn DoEnvironmentSubstA(pszsrc: ::windows_sys::core::PSTR, cchsrc: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn DoEnvironmentSubstW(pszsrc: ::windows_sys::core::PWSTR, cchsrc: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DragAcceptFiles(hwnd: super::super::Foundation::HWND, faccept: super::super::Foundation::BOOL);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn DragFinish(hdrop: HDROP);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn DragQueryFileA(hdrop: HDROP, ifile: u32, lpszfile: ::windows_sys::core::PSTR, cch: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn DragQueryFileW(hdrop: HDROP, ifile: u32, lpszfile: ::windows_sys::core::PWSTR, cch: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DragQueryPoint(hdrop: HDROP, ppt: *mut super::super::Foundation::POINT) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn DriveType(idrive: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn DuplicateIcon(hinst: super::super::Foundation::HINSTANCE, hicon: super::WindowsAndMessaging::HICON) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ExtractAssociatedIconA(hinst: super::super::Foundation::HINSTANCE, psziconpath: ::windows_sys::core::PSTR, piicon: *mut u16) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ExtractAssociatedIconExA(hinst: super::super::Foundation::HINSTANCE, psziconpath: ::windows_sys::core::PSTR, piiconindex: *mut u16, piiconid: *mut u16) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ExtractAssociatedIconExW(hinst: super::super::Foundation::HINSTANCE, psziconpath: ::windows_sys::core::PWSTR, piiconindex: *mut u16, piiconid: *mut u16) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ExtractAssociatedIconW(hinst: super::super::Foundation::HINSTANCE, psziconpath: ::windows_sys::core::PWSTR, piicon: *mut u16) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ExtractIconA(hinst: super::super::Foundation::HINSTANCE, pszexefilename: ::windows_sys::core::PCSTR, niconindex: u32) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn ExtractIconExA(lpszfile: ::windows_sys::core::PCSTR, niconindex: i32, phiconlarge: *mut super::WindowsAndMessaging::HICON, phiconsmall: *mut super::WindowsAndMessaging::HICON, nicons: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn ExtractIconExW(lpszfile: ::windows_sys::core::PCWSTR, niconindex: i32, phiconlarge: *mut super::WindowsAndMessaging::HICON, phiconsmall: *mut super::WindowsAndMessaging::HICON, nicons: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ExtractIconW(hinst: super::super::Foundation::HINSTANCE, pszexefilename: ::windows_sys::core::PCWSTR, niconindex: u32) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FindExecutableA(lpfile: ::windows_sys::core::PCSTR, lpdirectory: ::windows_sys::core::PCSTR, lpresult: ::windows_sys::core::PSTR) -> super::super::Foundation::HINSTANCE;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FindExecutableW(lpfile: ::windows_sys::core::PCWSTR, lpdirectory: ::windows_sys::core::PCWSTR, lpresult: ::windows_sys::core::PWSTR) -> super::super::Foundation::HINSTANCE;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn GetAcceptLanguagesA(pszlanguages: ::windows_sys::core::PSTR, pcchlanguages: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn GetAcceptLanguagesW(pszlanguages: ::windows_sys::core::PWSTR, pcchlanguages: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetAllUsersProfileDirectoryA(lpprofiledir: ::windows_sys::core::PSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetAllUsersProfileDirectoryW(lpprofiledir: ::windows_sys::core::PWSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn GetCurrentProcessExplicitAppUserModelID(appid: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetDefaultUserProfileDirectoryA(lpprofiledir: ::windows_sys::core::PSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetDefaultUserProfileDirectoryW(lpprofiledir: ::windows_sys::core::PWSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn GetDpiForShellUIComponent(param0: SHELL_UI_COMPONENT) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetFileNameFromBrowse(hwnd: super::super::Foundation::HWND, pszfilepath: ::windows_sys::core::PWSTR, cchfilepath: u32, pszworkingdir: ::windows_sys::core::PCWSTR, pszdefext: ::windows_sys::core::PCWSTR, pszfilters: ::windows_sys::core::PCWSTR, psztitle: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn GetMenuContextHelpId(param0: super::WindowsAndMessaging::HMENU) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn GetMenuPosFromID(hmenu: super::WindowsAndMessaging::HMENU, id: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetProfileType(dwflags: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetProfilesDirectoryA(lpprofiledir: ::windows_sys::core::PSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetProfilesDirectoryW(lpprofiledir: ::windows_sys::core::PWSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn GetScaleFactorForDevice(devicetype: DISPLAY_DEVICE_TYPE) -> Common::DEVICE_SCALE_FACTOR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Graphics_Gdi", feature = "Win32_UI_Shell_Common"))]
    pub fn GetScaleFactorForMonitor(hmon: super::super::Graphics::Gdi::HMONITOR, pscale: *mut Common::DEVICE_SCALE_FACTOR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetUserProfileDirectoryA(htoken: super::super::Foundation::HANDLE, lpprofiledir: ::windows_sys::core::PSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetUserProfileDirectoryW(htoken: super::super::Foundation::HANDLE, lpprofiledir: ::windows_sys::core::PWSTR, lpcchsize: *mut u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetWindowContextHelpId(param0: super::super::Foundation::HWND) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetWindowSubclass(hwnd: super::super::Foundation::HWND, pfnsubclass: SUBCLASSPROC, uidsubclass: usize, pdwrefdata: *mut usize) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserFree(param0: *const u32, param1: *const super::super::Graphics::Gdi::HMONITOR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserFree64(param0: *const u32, param1: *const super::super::Graphics::Gdi::HMONITOR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserMarshal(param0: *const u32, param1: *mut u8, param2: *const super::super::Graphics::Gdi::HMONITOR) -> *mut u8;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserMarshal64(param0: *const u32, param1: *mut u8, param2: *const super::super::Graphics::Gdi::HMONITOR) -> *mut u8;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserSize(param0: *const u32, param1: u32, param2: *const super::super::Graphics::Gdi::HMONITOR) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserSize64(param0: *const u32, param1: u32, param2: *const super::super::Graphics::Gdi::HMONITOR) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserUnmarshal(param0: *const u32, param1: *const u8, param2: *mut super::super::Graphics::Gdi::HMONITOR) -> *mut u8;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn HMONITOR_UserUnmarshal64(param0: *const u32, param1: *const u8, param2: *mut super::super::Graphics::Gdi::HMONITOR) -> *mut u8;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HashData(pbdata: *const u8, cbdata: u32, pbhash: *mut u8, cbhash: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkClone(pihl: IHlink, riid: *const ::windows_sys::core::GUID, pihlsiteforclone: IHlinkSite, dwsitedata: u32, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkCreateBrowseContext(piunkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn HlinkCreateExtensionServices(pwzadditionalheaders: ::windows_sys::core::PCWSTR, phwnd: super::super::Foundation::HWND, pszusername: ::windows_sys::core::PCWSTR, pszpassword: ::windows_sys::core::PCWSTR, piunkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkCreateFromData(pidataobj: super::super::System::Com::IDataObject, pihlsite: IHlinkSite, dwsitedata: u32, piunkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkCreateFromMoniker(pimktrgt: super::super::System::Com::IMoniker, pwzlocation: ::windows_sys::core::PCWSTR, pwzfriendlyname: ::windows_sys::core::PCWSTR, pihlsite: IHlinkSite, dwsitedata: u32, piunkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkCreateFromString(pwztarget: ::windows_sys::core::PCWSTR, pwzlocation: ::windows_sys::core::PCWSTR, pwzfriendlyname: ::windows_sys::core::PCWSTR, pihlsite: IHlinkSite, dwsitedata: u32, piunkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkCreateShortcut(grfhlshortcutf: u32, pihl: IHlink, pwzdir: ::windows_sys::core::PCWSTR, pwzfilename: ::windows_sys::core::PCWSTR, ppwzshortcutfile: *mut ::windows_sys::core::PWSTR, dwreserved: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkCreateShortcutFromMoniker(grfhlshortcutf: u32, pimktarget: super::super::System::Com::IMoniker, pwzlocation: ::windows_sys::core::PCWSTR, pwzdir: ::windows_sys::core::PCWSTR, pwzfilename: ::windows_sys::core::PCWSTR, ppwzshortcutfile: *mut ::windows_sys::core::PWSTR, dwreserved: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkCreateShortcutFromString(grfhlshortcutf: u32, pwztarget: ::windows_sys::core::PCWSTR, pwzlocation: ::windows_sys::core::PCWSTR, pwzdir: ::windows_sys::core::PCWSTR, pwzfilename: ::windows_sys::core::PCWSTR, ppwzshortcutfile: *mut ::windows_sys::core::PWSTR, dwreserved: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkGetSpecialReference(ureference: u32, ppwzreference: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkGetValueFromParams(pwzparams: ::windows_sys::core::PCWSTR, pwzname: ::windows_sys::core::PCWSTR, ppwzvalue: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkIsShortcut(pwzfilename: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkNavigate(pihl: IHlink, pihlframe: IHlinkFrame, grfhlnf: u32, pbc: super::super::System::Com::IBindCtx, pibsc: super::super::System::Com::IBindStatusCallback, pihlbc: IHlinkBrowseContext) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkNavigateToStringReference(pwztarget: ::windows_sys::core::PCWSTR, pwzlocation: ::windows_sys::core::PCWSTR, pihlsite: IHlinkSite, dwsitedata: u32, pihlframe: IHlinkFrame, grfhlnf: u32, pibc: super::super::System::Com::IBindCtx, pibsc: super::super::System::Com::IBindStatusCallback, pihlbc: IHlinkBrowseContext) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkOnNavigate(pihlframe: IHlinkFrame, pihlbc: IHlinkBrowseContext, grfhlnf: u32, pimktarget: super::super::System::Com::IMoniker, pwzlocation: ::windows_sys::core::PCWSTR, pwzfriendlyname: ::windows_sys::core::PCWSTR, puhlid: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkOnRenameDocument(dwreserved: u32, pihlbc: IHlinkBrowseContext, pimkold: super::super::System::Com::IMoniker, pimknew: super::super::System::Com::IMoniker) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn HlinkParseDisplayName(pibc: super::super::System::Com::IBindCtx, pwzdisplayname: ::windows_sys::core::PCWSTR, fnoforceabs: super::super::Foundation::BOOL, pccheaten: *mut u32, ppimk: *mut super::super::System::Com::IMoniker) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkPreprocessMoniker(pibc: super::super::System::Com::IBindCtx, pimkin: super::super::System::Com::IMoniker, ppimkout: *mut super::super::System::Com::IMoniker) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkQueryCreateFromData(pidataobj: super::super::System::Com::IDataObject) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkResolveMonikerForData(pimkreference: super::super::System::Com::IMoniker, reserved: u32, pibc: super::super::System::Com::IBindCtx, cfmtetc: u32, rgfmtetc: *mut super::super::System::Com::FORMATETC, pibsc: super::super::System::Com::IBindStatusCallback, pimkbase: super::super::System::Com::IMoniker) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkResolveShortcut(pwzshortcutfilename: ::windows_sys::core::PCWSTR, pihlsite: IHlinkSite, dwsitedata: u32, piunkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkResolveShortcutToMoniker(pwzshortcutfilename: ::windows_sys::core::PCWSTR, ppimktarget: *mut super::super::System::Com::IMoniker, ppwzlocation: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkResolveShortcutToString(pwzshortcutfilename: ::windows_sys::core::PCWSTR, ppwztarget: *mut ::windows_sys::core::PWSTR, ppwzlocation: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkResolveStringForData(pwzreference: ::windows_sys::core::PCWSTR, reserved: u32, pibc: super::super::System::Com::IBindCtx, cfmtetc: u32, rgfmtetc: *mut super::super::System::Com::FORMATETC, pibsc: super::super::System::Com::IBindStatusCallback, pimkbase: super::super::System::Com::IMoniker) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkSetSpecialReference(ureference: u32, pwzreference: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn HlinkTranslateURL(pwzurl: ::windows_sys::core::PCWSTR, grfflags: u32, ppwztranslatedurl: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn HlinkUpdateStackItem(pihlframe: IHlinkFrame, pihlbc: IHlinkBrowseContext, uhlid: u32, pimktrgt: super::super::System::Com::IMoniker, pwzlocation: ::windows_sys::core::PCWSTR, pwzfriendlyname: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn ILAppendID(pidl: *const Common::ITEMIDLIST, pmkid: *const Common::SHITEMID, fappend: super::super::Foundation::BOOL) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILClone(pidl: *const Common::ITEMIDLIST) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILCloneFirst(pidl: *const Common::ITEMIDLIST) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILCombine(pidl1: *const Common::ITEMIDLIST, pidl2: *const Common::ITEMIDLIST) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILCreateFromPathA(pszpath: ::windows_sys::core::PCSTR) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILCreateFromPathW(pszpath: ::windows_sys::core::PCWSTR) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILFindChild(pidlparent: *const Common::ITEMIDLIST, pidlchild: *const Common::ITEMIDLIST) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILFindLastID(pidl: *const Common::ITEMIDLIST) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILFree(pidl: *const Common::ITEMIDLIST);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILGetNext(pidl: *const Common::ITEMIDLIST) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn ILGetSize(pidl: *const Common::ITEMIDLIST) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn ILIsEqual(pidl1: *const Common::ITEMIDLIST, pidl2: *const Common::ITEMIDLIST) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn ILIsParent(pidl1: *const Common::ITEMIDLIST, pidl2: *const Common::ITEMIDLIST, fimmediate: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn ILLoadFromStreamEx(pstm: super::super::System::Com::IStream, pidl: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn ILRemoveLastID(pidl: *mut Common::ITEMIDLIST) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn ILSaveToStream(pstm: super::super::System::Com::IStream, pidl: *const Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_Copy(pstmfrom: super::super::System::Com::IStream, pstmto: super::super::System::Com::IStream, cb: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_Read(pstm: super::super::System::Com::IStream, pv: *mut ::core::ffi::c_void, cb: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn IStream_ReadPidl(pstm: super::super::System::Com::IStream, ppidlout: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_ReadStr(pstm: super::super::System::Com::IStream, ppsz: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_Reset(pstm: super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_Size(pstm: super::super::System::Com::IStream, pui: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_Write(pstm: super::super::System::Com::IStream, pv: *const ::core::ffi::c_void, cb: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn IStream_WritePidl(pstm: super::super::System::Com::IStream, pidlwrite: *const Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn IStream_WriteStr(pstm: super::super::System::Com::IStream, psz: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn IUnknown_AtomicRelease(ppunk: *mut *mut ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn IUnknown_GetSite(punk: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IUnknown_GetWindow(punk: ::windows_sys::core::IUnknown, phwnd: *mut super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn IUnknown_QueryService(punk: ::windows_sys::core::IUnknown, guidservice: *const ::windows_sys::core::GUID, riid: *const ::windows_sys::core::GUID, ppvout: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn IUnknown_Set(ppunk: *mut ::windows_sys::core::IUnknown, punk: ::windows_sys::core::IUnknown);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn IUnknown_SetSite(punk: ::windows_sys::core::IUnknown, punksite: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImportPrivacySettings(pszfilename: ::windows_sys::core::PCWSTR, pfparseprivacypreferences: *mut super::super::Foundation::BOOL, pfparsepersiterules: *mut super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn InitNetworkAddressControl() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IntlStrEqWorkerA(fcasesens: super::super::Foundation::BOOL, lpstring1: ::windows_sys::core::PCSTR, lpstring2: ::windows_sys::core::PCSTR, nchar: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IntlStrEqWorkerW(fcasesens: super::super::Foundation::BOOL, lpstring1: ::windows_sys::core::PCWSTR, lpstring2: ::windows_sys::core::PCWSTR, nchar: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsCharSpaceA(wch: super::super::Foundation::CHAR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsCharSpaceW(wch: u16) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsInternetESCEnabled() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsLFNDriveA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsLFNDriveW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn IsNetDrive(idrive: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsOS(dwos: OS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsUserAnAdmin() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn LoadUserProfileA(htoken: super::super::Foundation::HANDLE, lpprofileinfo: *mut PROFILEINFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn LoadUserProfileW(htoken: super::super::Foundation::HANDLE, lpprofileinfo: *mut PROFILEINFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn OleSaveToStreamEx(piunk: ::windows_sys::core::IUnknown, pistm: super::super::System::Com::IStream, fcleardirty: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_System_Registry"))]
    pub fn OpenRegStream(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, grfmode: u32) -> super::super::System::Com::IStream;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn ParseURLA(pcszurl: ::windows_sys::core::PCSTR, ppu: *mut PARSEDURLA) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn ParseURLW(pcszurl: ::windows_sys::core::PCWSTR, ppu: *mut PARSEDURLW) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathAddBackslashA(pszpath: ::windows_sys::core::PSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathAddBackslashW(pszpath: ::windows_sys::core::PWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathAddExtensionA(pszpath: ::windows_sys::core::PSTR, pszext: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathAddExtensionW(pszpath: ::windows_sys::core::PWSTR, pszext: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathAllocCanonicalize(pszpathin: ::windows_sys::core::PCWSTR, dwflags: u32, ppszpathout: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathAllocCombine(pszpathin: ::windows_sys::core::PCWSTR, pszmore: ::windows_sys::core::PCWSTR, dwflags: u32, ppszpathout: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathAppendA(pszpath: ::windows_sys::core::PSTR, pszmore: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathAppendW(pszpath: ::windows_sys::core::PWSTR, pszmore: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathBuildRootA(pszroot: ::windows_sys::core::PSTR, idrive: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathBuildRootW(pszroot: ::windows_sys::core::PWSTR, idrive: i32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathCanonicalizeA(pszbuf: ::windows_sys::core::PSTR, pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathCanonicalizeW(pszbuf: ::windows_sys::core::PWSTR, pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchAddBackslash(pszpath: ::windows_sys::core::PWSTR, cchpath: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchAddBackslashEx(pszpath: ::windows_sys::core::PWSTR, cchpath: usize, ppszend: *mut ::windows_sys::core::PWSTR, pcchremaining: *mut usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchAddExtension(pszpath: ::windows_sys::core::PWSTR, cchpath: usize, pszext: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchAppend(pszpath: ::windows_sys::core::PWSTR, cchpath: usize, pszmore: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchAppendEx(pszpath: ::windows_sys::core::PWSTR, cchpath: usize, pszmore: ::windows_sys::core::PCWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchCanonicalize(pszpathout: ::windows_sys::core::PWSTR, cchpathout: usize, pszpathin: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchCanonicalizeEx(pszpathout: ::windows_sys::core::PWSTR, cchpathout: usize, pszpathin: ::windows_sys::core::PCWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchCombine(pszpathout: ::windows_sys::core::PWSTR, cchpathout: usize, pszpathin: ::windows_sys::core::PCWSTR, pszmore: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchCombineEx(pszpathout: ::windows_sys::core::PWSTR, cchpathout: usize, pszpathin: ::windows_sys::core::PCWSTR, pszmore: ::windows_sys::core::PCWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchFindExtension(pszpath: ::windows_sys::core::PCWSTR, cchpath: usize, ppszext: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathCchIsRoot(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchRemoveBackslash(pszpath: ::windows_sys::core::PWSTR, cchpath: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchRemoveBackslashEx(pszpath: ::windows_sys::core::PWSTR, cchpath: usize, ppszend: *mut ::windows_sys::core::PWSTR, pcchremaining: *mut usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchRemoveExtension(pszpath: ::windows_sys::core::PWSTR, cchpath: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchRemoveFileSpec(pszpath: ::windows_sys::core::PWSTR, cchpath: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchRenameExtension(pszpath: ::windows_sys::core::PWSTR, cchpath: usize, pszext: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchSkipRoot(pszpath: ::windows_sys::core::PCWSTR, ppszrootend: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchStripPrefix(pszpath: ::windows_sys::core::PWSTR, cchpath: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCchStripToRoot(pszpath: ::windows_sys::core::PWSTR, cchpath: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCleanupSpec(pszdir: ::windows_sys::core::PCWSTR, pszspec: ::windows_sys::core::PWSTR) -> PCS_RET;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCombineA(pszdest: ::windows_sys::core::PSTR, pszdir: ::windows_sys::core::PCSTR, pszfile: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCombineW(pszdest: ::windows_sys::core::PWSTR, pszdir: ::windows_sys::core::PCWSTR, pszfile: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCommonPrefixA(pszfile1: ::windows_sys::core::PCSTR, pszfile2: ::windows_sys::core::PCSTR, achpath: ::windows_sys::core::PSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCommonPrefixW(pszfile1: ::windows_sys::core::PCWSTR, pszfile2: ::windows_sys::core::PCWSTR, achpath: ::windows_sys::core::PWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn PathCompactPathA(hdc: super::super::Graphics::Gdi::HDC, pszpath: ::windows_sys::core::PSTR, dx: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathCompactPathExA(pszout: ::windows_sys::core::PSTR, pszsrc: ::windows_sys::core::PCSTR, cchmax: u32, dwflags: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathCompactPathExW(pszout: ::windows_sys::core::PWSTR, pszsrc: ::windows_sys::core::PCWSTR, cchmax: u32, dwflags: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn PathCompactPathW(hdc: super::super::Graphics::Gdi::HDC, pszpath: ::windows_sys::core::PWSTR, dx: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCreateFromUrlA(pszurl: ::windows_sys::core::PCSTR, pszpath: ::windows_sys::core::PSTR, pcchpath: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCreateFromUrlAlloc(pszin: ::windows_sys::core::PCWSTR, ppszout: *mut ::windows_sys::core::PWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathCreateFromUrlW(pszurl: ::windows_sys::core::PCWSTR, pszpath: ::windows_sys::core::PWSTR, pcchpath: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathFileExistsA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathFileExistsW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindExtensionA(pszpath: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindExtensionW(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindFileNameA(pszpath: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindFileNameW(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindNextComponentA(pszpath: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindNextComponentW(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathFindOnPathA(pszpath: ::windows_sys::core::PSTR, ppszotherdirs: *const *const i8) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathFindOnPathW(pszpath: ::windows_sys::core::PWSTR, ppszotherdirs: *const *const u16) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindSuffixArrayA(pszpath: ::windows_sys::core::PCSTR, apszsuffix: *const ::windows_sys::core::PSTR, iarraysize: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathFindSuffixArrayW(pszpath: ::windows_sys::core::PCWSTR, apszsuffix: *const ::windows_sys::core::PWSTR, iarraysize: i32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetArgsA(pszpath: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetArgsW(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetCharTypeA(ch: u8) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetCharTypeW(ch: u16) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetDriveNumberA(pszpath: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetDriveNumberW(pszpath: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathGetShortPath(pszlongpath: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsContentTypeA(pszpath: ::windows_sys::core::PCSTR, pszcontenttype: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsContentTypeW(pszpath: ::windows_sys::core::PCWSTR, pszcontenttype: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsDirectoryA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsDirectoryEmptyA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsDirectoryEmptyW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsDirectoryW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsExe(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsFileSpecA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsFileSpecW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsLFNFileSpecA(pszname: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsLFNFileSpecW(pszname: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsNetworkPathA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsNetworkPathW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsPrefixA(pszprefix: ::windows_sys::core::PCSTR, pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsPrefixW(pszprefix: ::windows_sys::core::PCWSTR, pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsRelativeA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsRelativeW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsRootA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsRootW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsSameRootA(pszpath1: ::windows_sys::core::PCSTR, pszpath2: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsSameRootW(pszpath1: ::windows_sys::core::PCWSTR, pszpath2: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsSlowA(pszfile: ::windows_sys::core::PCSTR, dwattr: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsSlowW(pszfile: ::windows_sys::core::PCWSTR, dwattr: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsSystemFolderA(pszpath: ::windows_sys::core::PCSTR, dwattrb: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsSystemFolderW(pszpath: ::windows_sys::core::PCWSTR, dwattrb: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCEx(pszpath: ::windows_sys::core::PCWSTR, ppszserver: *mut ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCServerA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCServerShareA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCServerShareW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCServerW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsUNCW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsURLA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathIsURLW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMakePrettyA(pszpath: ::windows_sys::core::PSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMakePrettyW(pszpath: ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMakeSystemFolderA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMakeSystemFolderW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMakeUniqueName(pszuniquename: ::windows_sys::core::PWSTR, cchmax: u32, psztemplate: ::windows_sys::core::PCWSTR, pszlongplate: ::windows_sys::core::PCWSTR, pszdir: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMatchSpecA(pszfile: ::windows_sys::core::PCSTR, pszspec: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathMatchSpecExA(pszfile: ::windows_sys::core::PCSTR, pszspec: ::windows_sys::core::PCSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathMatchSpecExW(pszfile: ::windows_sys::core::PCWSTR, pszspec: ::windows_sys::core::PCWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathMatchSpecW(pszfile: ::windows_sys::core::PCWSTR, pszspec: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathParseIconLocationA(psziconfile: ::windows_sys::core::PSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathParseIconLocationW(psziconfile: ::windows_sys::core::PWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathQualify(psz: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathQuoteSpacesA(lpsz: ::windows_sys::core::PSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathQuoteSpacesW(lpsz: ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathRelativePathToA(pszpath: ::windows_sys::core::PSTR, pszfrom: ::windows_sys::core::PCSTR, dwattrfrom: u32, pszto: ::windows_sys::core::PCSTR, dwattrto: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathRelativePathToW(pszpath: ::windows_sys::core::PWSTR, pszfrom: ::windows_sys::core::PCWSTR, dwattrfrom: u32, pszto: ::windows_sys::core::PCWSTR, dwattrto: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveArgsA(pszpath: ::windows_sys::core::PSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveArgsW(pszpath: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveBackslashA(pszpath: ::windows_sys::core::PSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveBackslashW(pszpath: ::windows_sys::core::PWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveBlanksA(pszpath: ::windows_sys::core::PSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveBlanksW(pszpath: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveExtensionA(pszpath: ::windows_sys::core::PSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathRemoveExtensionW(pszpath: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathRemoveFileSpecA(pszpath: ::windows_sys::core::PSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathRemoveFileSpecW(pszpath: ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathRenameExtensionA(pszpath: ::windows_sys::core::PSTR, pszext: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathRenameExtensionW(pszpath: ::windows_sys::core::PWSTR, pszext: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathResolve(pszpath: ::windows_sys::core::PWSTR, dirs: *const *const u16, fflags: PRF_FLAGS) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathSearchAndQualifyA(pszpath: ::windows_sys::core::PCSTR, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathSearchAndQualifyW(pszpath: ::windows_sys::core::PCWSTR, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathSetDlgItemPathA(hdlg: super::super::Foundation::HWND, id: i32, pszpath: ::windows_sys::core::PCSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathSetDlgItemPathW(hdlg: super::super::Foundation::HWND, id: i32, pszpath: ::windows_sys::core::PCWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathSkipRootA(pszpath: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathSkipRootW(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathStripPathA(pszpath: ::windows_sys::core::PSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathStripPathW(pszpath: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathStripToRootA(pszpath: ::windows_sys::core::PSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathStripToRootW(pszpath: ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathUnExpandEnvStringsA(pszpath: ::windows_sys::core::PCSTR, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathUnExpandEnvStringsW(pszpath: ::windows_sys::core::PCWSTR, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathUndecorateA(pszpath: ::windows_sys::core::PSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn PathUndecorateW(pszpath: ::windows_sys::core::PWSTR);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathUnmakeSystemFolderA(pszpath: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathUnmakeSystemFolderW(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathUnquoteSpacesA(lpsz: ::windows_sys::core::PSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathUnquoteSpacesW(lpsz: ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PathYetAnotherMakeUniqueName(pszuniquename: ::windows_sys::core::PWSTR, pszpath: ::windows_sys::core::PCWSTR, pszshort: ::windows_sys::core::PCWSTR, pszfilespec: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PickIconDlg(hwnd: super::super::Foundation::HWND, psziconpath: ::windows_sys::core::PWSTR, cchiconpath: u32, piiconindex: *mut i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn QISearch(that: *mut ::core::ffi::c_void, pqit: *const QITAB, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ReadCabinetState(pcs: *mut CABINETSTATE, clength: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RealDriveType(idrive: i32, foktohitnet: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterAppConstrainedChangeNotification(routine: PAPPCONSTRAIN_CHANGE_ROUTINE, context: *const ::core::ffi::c_void, registration: *mut *mut _APPCONSTRAIN_REGISTRATION) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterAppStateChangeNotification(routine: PAPPSTATE_CHANGE_ROUTINE, context: *const ::core::ffi::c_void, registration: *mut *mut _APPSTATE_REGISTRATION) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterScaleChangeEvent(hevent: super::super::Foundation::HANDLE, pdwcookie: *mut usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterScaleChangeNotifications(displaydevice: DISPLAY_DEVICE_TYPE, hwndnotify: super::super::Foundation::HWND, umsgnotify: u32, pdwcookie: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RemoveWindowSubclass(hwnd: super::super::Foundation::HWND, pfnsubclass: SUBCLASSPROC, uidsubclass: usize) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RestartDialog(hwnd: super::super::Foundation::HWND, pszprompt: ::windows_sys::core::PCWSTR, dwreturn: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RestartDialogEx(hwnd: super::super::Foundation::HWND, pszprompt: ::windows_sys::core::PCWSTR, dwreturn: u32, dwreasoncode: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn RevokeScaleChangeNotifications(displaydevice: DISPLAY_DEVICE_TYPE, dwcookie: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Controls\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
    pub fn SHAddFromPropSheetExtArray(hpsxa: HPSXA, lpfnaddpage: super::Controls::LPFNSVADDPROPSHEETPAGE, lparam: super::super::Foundation::LPARAM) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHAddToRecentDocs(uflags: u32, pv: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHAlloc(cb: usize) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHAllocShared(pvdata: *const ::core::ffi::c_void, dwsize: u32, dwprocessid: u32) -> super::super::Foundation::HANDLE;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHAnsiToAnsi(pszsrc: ::windows_sys::core::PCSTR, pszdst: ::windows_sys::core::PSTR, cchbuf: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHAnsiToUnicode(pszsrc: ::windows_sys::core::PCSTR, pwszdst: ::windows_sys::core::PWSTR, cwchbuf: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHAppBarMessage(dwmessage: u32, pdata: *mut APPBARDATA) -> usize;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHAssocEnumHandlers(pszextra: ::windows_sys::core::PCWSTR, affilter: ASSOC_FILTER, ppenumhandler: *mut IEnumAssocHandlers) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHAssocEnumHandlersForProtocolByApplication(protocol: ::windows_sys::core::PCWSTR, riid: *const ::windows_sys::core::GUID, enumhandlers: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHAutoComplete(hwndedit: super::super::Foundation::HWND, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHBindToFolderIDListParent(psfroot: IShellFolder, pidl: *const Common::ITEMIDLIST, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void, ppidllast: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn SHBindToFolderIDListParentEx(psfroot: IShellFolder, pidl: *const Common::ITEMIDLIST, ppbc: super::super::System::Com::IBindCtx, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void, ppidllast: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn SHBindToObject(psf: IShellFolder, pidl: *const Common::ITEMIDLIST, pbc: super::super::System::Com::IBindCtx, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHBindToParent(pidl: *const Common::ITEMIDLIST, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void, ppidllast: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHBrowseForFolderA(lpbi: *const BROWSEINFOA) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHBrowseForFolderW(lpbi: *const BROWSEINFOW) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCLSIDFromString(psz: ::windows_sys::core::PCWSTR, pclsid: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHChangeNotification_Lock(hchange: super::super::Foundation::HANDLE, dwprocid: u32, pppidl: *mut *mut *mut Common::ITEMIDLIST, plevent: *mut i32) -> ShFindChangeNotificationHandle;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHChangeNotification_Unlock(hlock: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHChangeNotify(weventid: SHCNE_ID, uflags: SHCNF_FLAGS, dwitem1: *const ::core::ffi::c_void, dwitem2: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHChangeNotifyDeregister(ulid: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHChangeNotifyRegister(hwnd: super::super::Foundation::HWND, fsources: SHCNRF_SOURCE, fevents: i32, wmsg: u32, centries: i32, pshcne: *const SHChangeNotifyEntry) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHChangeNotifyRegisterThread(status: SCNRT_STATUS);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHCloneSpecialIDList(hwnd: super::super::Foundation::HWND, csidl: i32, fcreate: super::super::Foundation::BOOL) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCoCreateInstance(pszclsid: ::windows_sys::core::PCWSTR, pclsid: *const ::windows_sys::core::GUID, punkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHCopyKeyA(hkeysrc: super::super::System::Registry::HKEY, pszsrcsubkey: ::windows_sys::core::PCSTR, hkeydest: super::super::System::Registry::HKEY, freserved: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHCopyKeyW(hkeysrc: super::super::System::Registry::HKEY, pszsrcsubkey: ::windows_sys::core::PCWSTR, hkeydest: super::super::System::Registry::HKEY, freserved: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateAssociationRegistration(riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn SHCreateDataObject(pidlfolder: *const Common::ITEMIDLIST, cidl: u32, apidl: *const *const Common::ITEMIDLIST, pdtinner: super::super::System::Com::IDataObject, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry", feature = "Win32_UI_Shell_Common"))]
    pub fn SHCreateDefaultContextMenu(pdcm: *const DEFCONTEXTMENU, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateDefaultExtractIcon(riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateDefaultPropertiesOp(psi: IShellItem, ppfileop: *mut IFileOperation) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHCreateDirectory(hwnd: super::super::Foundation::HWND, pszpath: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn SHCreateDirectoryExA(hwnd: super::super::Foundation::HWND, pszpath: ::windows_sys::core::PCSTR, psa: *const super::super::Security::SECURITY_ATTRIBUTES) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security"))]
    pub fn SHCreateDirectoryExW(hwnd: super::super::Foundation::HWND, pszpath: ::windows_sys::core::PCWSTR, psa: *const super::super::Security::SECURITY_ATTRIBUTES) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateFileExtractIconW(pszfile: ::windows_sys::core::PCWSTR, dwfileattributes: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHCreateItemFromIDList(pidl: *const Common::ITEMIDLIST, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateItemFromParsingName(pszpath: ::windows_sys::core::PCWSTR, pbc: super::super::System::Com::IBindCtx, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateItemFromRelativeName(psiparent: IShellItem, pszname: ::windows_sys::core::PCWSTR, pbc: super::super::System::Com::IBindCtx, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateItemInKnownFolder(kfid: *const ::windows_sys::core::GUID, dwkfflags: u32, pszitem: ::windows_sys::core::PCWSTR, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHCreateItemWithParent(pidlparent: *const Common::ITEMIDLIST, psfparent: IShellFolder, pidl: *const Common::ITEMIDLIST, riid: *const ::windows_sys::core::GUID, ppvitem: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateMemStream(pinit: *const u8, cbinit: u32) -> super::super::System::Com::IStream;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`, `\"Win32_System_Threading\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
    pub fn SHCreateProcessAsUserW(pscpi: *mut SHCREATEPROCESSINFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn SHCreatePropSheetExtArray(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, max_iface: u32) -> HPSXA;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateQueryCancelAutoPlayMoniker(ppmoniker: *mut super::super::System::Com::IMoniker) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(feature = "Win32_System_Ole")]
    pub fn SHCreateShellFolderView(pcsfv: *const SFV_CREATE, ppsv: *mut IShellView) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
    pub fn SHCreateShellFolderViewEx(pcsfv: *const CSFV, ppsv: *mut IShellView) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHCreateShellItem(pidlparent: *const Common::ITEMIDLIST, psfparent: IShellFolder, pidl: *const Common::ITEMIDLIST, ppsi: *mut IShellItem) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHCreateShellItemArray(pidlparent: *const Common::ITEMIDLIST, psf: IShellFolder, cidl: u32, ppidl: *const *const Common::ITEMIDLIST, ppsiitemarray: *mut IShellItemArray) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateShellItemArrayFromDataObject(pdo: super::super::System::Com::IDataObject, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHCreateShellItemArrayFromIDLists(cidl: u32, rgpidl: *const *const Common::ITEMIDLIST, ppsiitemarray: *mut IShellItemArray) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateShellItemArrayFromShellItem(psi: IShellItem, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn SHCreateShellPalette(hdc: super::super::Graphics::Gdi::HDC) -> super::super::Graphics::Gdi::HPALETTE;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateStdEnumFmtEtc(cfmt: u32, afmt: *const super::super::System::Com::FORMATETC, ppenumformatetc: *mut super::super::System::Com::IEnumFORMATETC) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateStreamOnFileA(pszfile: ::windows_sys::core::PCSTR, grfmode: u32, ppstm: *mut super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn SHCreateStreamOnFileEx(pszfile: ::windows_sys::core::PCWSTR, grfmode: u32, dwattributes: u32, fcreate: super::super::Foundation::BOOL, pstmtemplate: super::super::System::Com::IStream, ppstm: *mut super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHCreateStreamOnFileW(pszfile: ::windows_sys::core::PCWSTR, grfmode: u32, ppstm: *mut super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Threading\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Threading"))]
    pub fn SHCreateThread(pfnthreadproc: super::super::System::Threading::LPTHREAD_START_ROUTINE, pdata: *const ::core::ffi::c_void, flags: u32, pfncallback: super::super::System::Threading::LPTHREAD_START_ROUTINE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHCreateThreadRef(pcref: *mut i32, ppunk: *mut ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Threading\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Threading"))]
    pub fn SHCreateThreadWithHandle(pfnthreadproc: super::super::System::Threading::LPTHREAD_START_ROUTINE, pdata: *const ::core::ffi::c_void, flags: u32, pfncallback: super::super::System::Threading::LPTHREAD_START_ROUTINE, phandle: *mut super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn SHDefExtractIconA(psziconfile: ::windows_sys::core::PCSTR, iindex: i32, uflags: u32, phiconlarge: *mut super::WindowsAndMessaging::HICON, phiconsmall: *mut super::WindowsAndMessaging::HICON, niconsize: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn SHDefExtractIconW(psziconfile: ::windows_sys::core::PCWSTR, iindex: i32, uflags: u32, phiconlarge: *mut super::WindowsAndMessaging::HICON, phiconsmall: *mut super::WindowsAndMessaging::HICON, niconsize: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHDeleteEmptyKeyA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHDeleteEmptyKeyW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHDeleteKeyA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHDeleteKeyW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHDeleteValueA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHDeleteValueW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHDestroyPropSheetExtArray(hpsxa: HPSXA);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn SHDoDragDrop(hwnd: super::super::Foundation::HWND, pdata: super::super::System::Com::IDataObject, pdsrc: super::super::System::Ole::IDropSource, dweffect: u32, pdweffect: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHEmptyRecycleBinA(hwnd: super::super::Foundation::HWND, pszrootpath: ::windows_sys::core::PCSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHEmptyRecycleBinW(hwnd: super::super::Foundation::HWND, pszrootpath: ::windows_sys::core::PCWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHEnumKeyExA(hkey: super::super::System::Registry::HKEY, dwindex: u32, pszname: ::windows_sys::core::PSTR, pcchname: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHEnumKeyExW(hkey: super::super::System::Registry::HKEY, dwindex: u32, pszname: ::windows_sys::core::PWSTR, pcchname: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHEnumValueA(hkey: super::super::System::Registry::HKEY, dwindex: u32, pszvaluename: ::windows_sys::core::PSTR, pcchvaluename: *mut u32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHEnumValueW(hkey: super::super::System::Registry::HKEY, dwindex: u32, pszvaluename: ::windows_sys::core::PWSTR, pcchvaluename: *mut u32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn SHEnumerateUnreadMailAccountsW(hkeyuser: super::super::System::Registry::HKEY, dwindex: u32, pszmailaddress: ::windows_sys::core::PWSTR, cchmailaddress: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHEvaluateSystemCommandTemplate(pszcmdtemplate: ::windows_sys::core::PCWSTR, ppszapplication: *mut ::windows_sys::core::PWSTR, ppszcommandline: *mut ::windows_sys::core::PWSTR, ppszparameters: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFileOperationA(lpfileop: *mut SHFILEOPSTRUCTA) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFileOperationW(lpfileop: *mut SHFILEOPSTRUCTW) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHFindFiles(pidlfolder: *const Common::ITEMIDLIST, pidlsavefile: *const Common::ITEMIDLIST) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SHFind_InitMenuPopup(hmenu: super::WindowsAndMessaging::HMENU, hwndowner: super::super::Foundation::HWND, idcmdfirst: u32, idcmdlast: u32) -> IContextMenu;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHFlushSFCache();
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFormatDateTimeA(pft: *const super::super::Foundation::FILETIME, pdwflags: *mut u32, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFormatDateTimeW(pft: *const super::super::Foundation::FILETIME, pdwflags: *mut u32, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFormatDrive(hwnd: super::super::Foundation::HWND, drive: u32, fmtid: SHFMT_ID, options: SHFMT_OPT) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHFree(pv: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFreeNameMappings(hnamemappings: super::super::Foundation::HANDLE);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHFreeShared(hdata: super::super::Foundation::HANDLE, dwprocessid: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHGetAttributesFromDataObject(pdo: super::super::System::Com::IDataObject, dwattributemask: u32, pdwattributes: *mut u32, pcitems: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetDataFromIDListA(psf: IShellFolder, pidl: *const Common::ITEMIDLIST, nformat: SHGDFIL_FORMAT, pv: *mut ::core::ffi::c_void, cb: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetDataFromIDListW(psf: IShellFolder, pidl: *const Common::ITEMIDLIST, nformat: SHGDFIL_FORMAT, pv: *mut ::core::ffi::c_void, cb: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetDesktopFolder(ppshf: *mut IShellFolder) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetDiskFreeSpaceExA(pszdirectoryname: ::windows_sys::core::PCSTR, pulfreebytesavailabletocaller: *mut u64, pultotalnumberofbytes: *mut u64, pultotalnumberoffreebytes: *mut u64) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetDiskFreeSpaceExW(pszdirectoryname: ::windows_sys::core::PCWSTR, pulfreebytesavailabletocaller: *mut u64, pultotalnumberofbytes: *mut u64, pultotalnumberoffreebytes: *mut u64) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetDriveMedia(pszdrive: ::windows_sys::core::PCWSTR, pdwmediacontent: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Storage_FileSystem\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Storage_FileSystem", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SHGetFileInfoA(pszpath: ::windows_sys::core::PCSTR, dwfileattributes: super::super::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES, psfi: *mut SHFILEINFOA, cbfileinfo: u32, uflags: SHGFI_FLAGS) -> usize;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Storage_FileSystem\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Storage_FileSystem", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SHGetFileInfoW(pszpath: ::windows_sys::core::PCWSTR, dwfileattributes: super::super::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES, psfi: *mut SHFILEINFOW, cbfileinfo: u32, uflags: SHGFI_FLAGS) -> usize;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHGetFolderLocation(hwnd: super::super::Foundation::HWND, csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, ppidl: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetFolderPathA(hwnd: super::super::Foundation::HWND, csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, pszpath: ::windows_sys::core::PSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetFolderPathAndSubDirA(hwnd: super::super::Foundation::HWND, csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, pszsubdir: ::windows_sys::core::PCSTR, pszpath: ::windows_sys::core::PSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetFolderPathAndSubDirW(hwnd: super::super::Foundation::HWND, csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, pszsubdir: ::windows_sys::core::PCWSTR, pszpath: ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetFolderPathW(hwnd: super::super::Foundation::HWND, csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, pszpath: ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetIDListFromObject(punk: ::windows_sys::core::IUnknown, ppidl: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetIconOverlayIndexA(psziconpath: ::windows_sys::core::PCSTR, iiconindex: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetIconOverlayIndexW(psziconpath: ::windows_sys::core::PCWSTR, iiconindex: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetImageList(iimagelist: i32, riid: *const ::windows_sys::core::GUID, ppvobj: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetInstanceExplorer(ppunk: *mut ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetInverseCMAP(pbmap: *mut u8, cbmap: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHGetItemFromDataObject(pdtobj: super::super::System::Com::IDataObject, dwflags: DATAOBJ_GET_ITEM_FLAGS, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetItemFromObject(punk: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHGetKnownFolderIDList(rfid: *const ::windows_sys::core::GUID, dwflags: u32, htoken: super::super::Foundation::HANDLE, ppidl: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetKnownFolderItem(rfid: *const ::windows_sys::core::GUID, flags: KNOWN_FOLDER_FLAG, htoken: super::super::Foundation::HANDLE, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetKnownFolderPath(rfid: *const ::windows_sys::core::GUID, dwflags: u32, htoken: super::super::Foundation::HANDLE, ppszpath: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetLocalizedName(pszpath: ::windows_sys::core::PCWSTR, pszresmodule: ::windows_sys::core::PWSTR, cch: u32, pidsres: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHGetMalloc(ppmalloc: *mut super::super::System::Com::IMalloc) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetNameFromIDList(pidl: *const Common::ITEMIDLIST, sigdnname: SIGDN, ppszname: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetNewLinkInfoA(pszlinkto: ::windows_sys::core::PCSTR, pszdir: ::windows_sys::core::PCSTR, pszname: ::windows_sys::core::PSTR, pfmustcopy: *mut super::super::Foundation::BOOL, uflags: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetNewLinkInfoW(pszlinkto: ::windows_sys::core::PCWSTR, pszdir: ::windows_sys::core::PCWSTR, pszname: ::windows_sys::core::PWSTR, pfmustcopy: *mut super::super::Foundation::BOOL, uflags: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHGetPathFromIDListA(pidl: *const Common::ITEMIDLIST, pszpath: ::windows_sys::core::PSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHGetPathFromIDListEx(pidl: *const Common::ITEMIDLIST, pszpath: ::windows_sys::core::PWSTR, cchpath: u32, uopts: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHGetPathFromIDListW(pidl: *const Common::ITEMIDLIST, pszpath: ::windows_sys::core::PWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetRealIDL(psf: IShellFolder, pidlsimple: *const Common::ITEMIDLIST, ppidlreal: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetSetFolderCustomSettings(pfcs: *mut SHFOLDERCUSTOMSETTINGS, pszpath: ::windows_sys::core::PCWSTR, dwreadwrite: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetSetSettings(lpss: *mut SHELLSTATEA, dwmask: SSF_MASK, bset: super::super::Foundation::BOOL);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetSettings(psfs: *mut SHELLFLAGSTATE, dwmask: u32);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SHGetSpecialFolderLocation(hwnd: super::super::Foundation::HWND, csidl: i32, ppidl: *mut *mut Common::ITEMIDLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetSpecialFolderPathA(hwnd: super::super::Foundation::HWND, pszpath: ::windows_sys::core::PSTR, csidl: i32, fcreate: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetSpecialFolderPathW(hwnd: super::super::Foundation::HWND, pszpath: ::windows_sys::core::PWSTR, csidl: i32, fcreate: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn SHGetStockIconInfo(siid: SHSTOCKICONID, uflags: u32, psii: *mut SHSTOCKICONINFO) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn SHGetTemporaryPropertyForItem(psi: IShellItem, propkey: *const PropertiesSystem::PROPERTYKEY, ppropvar: *mut super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGetThreadRef(ppunk: *mut ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHGetUnreadMailCountW(hkeyuser: super::super::System::Registry::HKEY, pszmailaddress: ::windows_sys::core::PCWSTR, pdwcount: *mut u32, pfiletime: *mut super::super::Foundation::FILETIME, pszshellexecutecommand: ::windows_sys::core::PWSTR, cchshellexecutecommand: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHGetValueA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHGetValueW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetViewStatePropertyBag(pidl: *const Common::ITEMIDLIST, pszbagname: ::windows_sys::core::PCWSTR, dwflags: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGlobalCounterDecrement(id: SHGLOBALCOUNTER) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGlobalCounterGetValue(id: SHGLOBALCOUNTER) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHGlobalCounterIncrement(id: SHGLOBALCOUNTER) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHHandleUpdateImage(pidlextra: *const Common::ITEMIDLIST) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHILCreateFromPath(pszpath: ::windows_sys::core::PCWSTR, ppidl: *mut *mut Common::ITEMIDLIST, rgfinout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHInvokePrinterCommandA(hwnd: super::super::Foundation::HWND, uaction: u32, lpbuf1: ::windows_sys::core::PCSTR, lpbuf2: ::windows_sys::core::PCSTR, fmodal: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHInvokePrinterCommandW(hwnd: super::super::Foundation::HWND, uaction: u32, lpbuf1: ::windows_sys::core::PCWSTR, lpbuf2: ::windows_sys::core::PCWSTR, fmodal: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHIsFileAvailableOffline(pwszpath: ::windows_sys::core::PCWSTR, pdwstatus: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHIsLowMemoryMachine(dwtype: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHLimitInputEdit(hwndedit: super::super::Foundation::HWND, psf: IShellFolder) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHLoadInProc(rclsid: *const ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHLoadIndirectString(pszsource: ::windows_sys::core::PCWSTR, pszoutbuf: ::windows_sys::core::PWSTR, cchoutbuf: u32, ppvreserved: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHLoadNonloadedIconOverlayIdentifiers() -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHLockShared(hdata: super::super::Foundation::HANDLE, dwprocessid: u32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHMapPIDLToSystemImageListIndex(pshf: IShellFolder, pidl: *const Common::ITEMIDLIST, piindexsel: *mut i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHMessageBoxCheckA(hwnd: super::super::Foundation::HWND, psztext: ::windows_sys::core::PCSTR, pszcaption: ::windows_sys::core::PCSTR, utype: u32, idefault: i32, pszregval: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHMessageBoxCheckW(hwnd: super::super::Foundation::HWND, psztext: ::windows_sys::core::PCWSTR, pszcaption: ::windows_sys::core::PCWSTR, utype: u32, idefault: i32, pszregval: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHMultiFileProperties(pdtobj: super::super::System::Com::IDataObject, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHObjectProperties(hwnd: super::super::Foundation::HWND, shopobjecttype: SHOP_TYPE, pszobjectname: ::windows_sys::core::PCWSTR, pszpropertypage: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHOpenFolderAndSelectItems(pidlfolder: *const Common::ITEMIDLIST, cidl: u32, apidl: *const *const Common::ITEMIDLIST, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_System_Registry"))]
    pub fn SHOpenPropSheetW(pszcaption: ::windows_sys::core::PCWSTR, ahkeys: *const super::super::System::Registry::HKEY, ckeys: u32, pclsiddefault: *const ::windows_sys::core::GUID, pdtobj: super::super::System::Com::IDataObject, psb: IShellBrowser, pstartpage: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_System_Registry"))]
    pub fn SHOpenRegStream2A(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, grfmode: u32) -> super::super::System::Com::IStream;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_System_Registry"))]
    pub fn SHOpenRegStream2W(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, grfmode: u32) -> super::super::System::Com::IStream;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_System_Registry"))]
    pub fn SHOpenRegStreamA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, grfmode: u32) -> super::super::System::Com::IStream;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_System_Registry"))]
    pub fn SHOpenRegStreamW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, grfmode: u32) -> super::super::System::Com::IStream;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHOpenWithDialog(hwndparent: super::super::Foundation::HWND, poainfo: *const OPENASINFO) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_System_Com", feature = "Win32_UI_Shell_Common"))]
    pub fn SHParseDisplayName(pszname: ::windows_sys::core::PCWSTR, pbc: super::super::System::Com::IBindCtx, ppidl: *mut *mut Common::ITEMIDLIST, sfgaoin: u32, psfgaoout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHPathPrepareForWriteA(hwnd: super::super::Foundation::HWND, punkenablemodless: ::windows_sys::core::IUnknown, pszpath: ::windows_sys::core::PCSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHPathPrepareForWriteW(hwnd: super::super::Foundation::HWND, punkenablemodless: ::windows_sys::core::IUnknown, pszpath: ::windows_sys::core::PCWSTR, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHQueryInfoKeyA(hkey: super::super::System::Registry::HKEY, pcsubkeys: *mut u32, pcchmaxsubkeylen: *mut u32, pcvalues: *mut u32, pcchmaxvaluenamelen: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHQueryInfoKeyW(hkey: super::super::System::Registry::HKEY, pcsubkeys: *mut u32, pcchmaxsubkeylen: *mut u32, pcvalues: *mut u32, pcchmaxvaluenamelen: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHQueryRecycleBinA(pszrootpath: ::windows_sys::core::PCSTR, pshqueryrbinfo: *mut SHQUERYRBINFO) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHQueryRecycleBinW(pszrootpath: ::windows_sys::core::PCWSTR, pshqueryrbinfo: *mut SHQUERYRBINFO) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHQueryUserNotificationState(pquns: *mut QUERY_USER_NOTIFICATION_STATE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHQueryValueExA(hkey: super::super::System::Registry::HKEY, pszvalue: ::windows_sys::core::PCSTR, pdwreserved: *mut u32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHQueryValueExW(hkey: super::super::System::Registry::HKEY, pszvalue: ::windows_sys::core::PCWSTR, pdwreserved: *mut u32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegCloseUSKey(huskey: isize) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegCreateUSKeyA(pszpath: ::windows_sys::core::PCSTR, samdesired: u32, hrelativeuskey: isize, phnewuskey: *mut isize, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegCreateUSKeyW(pwzpath: ::windows_sys::core::PCWSTR, samdesired: u32, hrelativeuskey: isize, phnewuskey: *mut isize, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegDeleteEmptyUSKeyA(huskey: isize, pszsubkey: ::windows_sys::core::PCSTR, delregflags: SHREGDEL_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegDeleteEmptyUSKeyW(huskey: isize, pwzsubkey: ::windows_sys::core::PCWSTR, delregflags: SHREGDEL_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegDeleteUSValueA(huskey: isize, pszvalue: ::windows_sys::core::PCSTR, delregflags: SHREGDEL_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegDeleteUSValueW(huskey: isize, pwzvalue: ::windows_sys::core::PCWSTR, delregflags: SHREGDEL_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegDuplicateHKey(hkey: super::super::System::Registry::HKEY) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegEnumUSKeyA(huskey: isize, dwindex: u32, pszname: ::windows_sys::core::PSTR, pcchname: *mut u32, enumregflags: SHREGENUM_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegEnumUSKeyW(huskey: isize, dwindex: u32, pwzname: ::windows_sys::core::PWSTR, pcchname: *mut u32, enumregflags: SHREGENUM_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegEnumUSValueA(huskey: isize, dwindex: u32, pszvaluename: ::windows_sys::core::PSTR, pcchvaluename: *mut u32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32, enumregflags: SHREGENUM_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegEnumUSValueW(huskey: isize, dwindex: u32, pszvaluename: ::windows_sys::core::PWSTR, pcchvaluename: *mut u32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32, enumregflags: SHREGENUM_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegGetBoolUSValueA(pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, fignorehkcu: super::super::Foundation::BOOL, fdefault: super::super::Foundation::BOOL) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegGetBoolUSValueW(pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, fignorehkcu: super::super::Foundation::BOOL, fdefault: super::super::Foundation::BOOL) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegGetIntW(hk: super::super::System::Registry::HKEY, pwzkey: ::windows_sys::core::PCWSTR, idefault: i32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegGetPathA(hkey: super::super::System::Registry::HKEY, pcszsubkey: ::windows_sys::core::PCSTR, pcszvalue: ::windows_sys::core::PCSTR, pszpath: ::windows_sys::core::PSTR, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegGetPathW(hkey: super::super::System::Registry::HKEY, pcszsubkey: ::windows_sys::core::PCWSTR, pcszvalue: ::windows_sys::core::PCWSTR, pszpath: ::windows_sys::core::PWSTR, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegGetUSValueA(pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32, fignorehkcu: super::super::Foundation::BOOL, pvdefaultdata: *const ::core::ffi::c_void, dwdefaultdatasize: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegGetUSValueW(pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32, fignorehkcu: super::super::Foundation::BOOL, pvdefaultdata: *const ::core::ffi::c_void, dwdefaultdatasize: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegGetValueA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, srrfflags: i32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegGetValueFromHKCUHKLM(pwszkey: ::windows_sys::core::PCWSTR, pwszvalue: ::windows_sys::core::PCWSTR, srrfflags: i32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegGetValueW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, srrfflags: i32, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegOpenUSKeyA(pszpath: ::windows_sys::core::PCSTR, samdesired: u32, hrelativeuskey: isize, phnewuskey: *mut isize, fignorehkcu: super::super::Foundation::BOOL) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegOpenUSKeyW(pwzpath: ::windows_sys::core::PCWSTR, samdesired: u32, hrelativeuskey: isize, phnewuskey: *mut isize, fignorehkcu: super::super::Foundation::BOOL) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegQueryInfoUSKeyA(huskey: isize, pcsubkeys: *mut u32, pcchmaxsubkeylen: *mut u32, pcvalues: *mut u32, pcchmaxvaluenamelen: *mut u32, enumregflags: SHREGENUM_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegQueryInfoUSKeyW(huskey: isize, pcsubkeys: *mut u32, pcchmaxsubkeylen: *mut u32, pcvalues: *mut u32, pcchmaxvaluenamelen: *mut u32, enumregflags: SHREGENUM_FLAGS) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegQueryUSValueA(huskey: isize, pszvalue: ::windows_sys::core::PCSTR, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32, fignorehkcu: super::super::Foundation::BOOL, pvdefaultdata: *const ::core::ffi::c_void, dwdefaultdatasize: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegQueryUSValueW(huskey: isize, pszvalue: ::windows_sys::core::PCWSTR, pdwtype: *mut u32, pvdata: *mut ::core::ffi::c_void, pcbdata: *mut u32, fignorehkcu: super::super::Foundation::BOOL, pvdefaultdata: *const ::core::ffi::c_void, dwdefaultdatasize: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegSetPathA(hkey: super::super::System::Registry::HKEY, pcszsubkey: ::windows_sys::core::PCSTR, pcszvalue: ::windows_sys::core::PCSTR, pcszpath: ::windows_sys::core::PCSTR, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn SHRegSetPathW(hkey: super::super::System::Registry::HKEY, pcszsubkey: ::windows_sys::core::PCWSTR, pcszvalue: ::windows_sys::core::PCWSTR, pcszpath: ::windows_sys::core::PCWSTR, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegSetUSValueA(pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, dwtype: u32, pvdata: *const ::core::ffi::c_void, cbdata: u32, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegSetUSValueW(pwzsubkey: ::windows_sys::core::PCWSTR, pwzvalue: ::windows_sys::core::PCWSTR, dwtype: u32, pvdata: *const ::core::ffi::c_void, cbdata: u32, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegWriteUSValueA(huskey: isize, pszvalue: ::windows_sys::core::PCSTR, dwtype: u32, pvdata: *const ::core::ffi::c_void, cbdata: u32, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHRegWriteUSValueW(huskey: isize, pwzvalue: ::windows_sys::core::PCWSTR, dwtype: u32, pvdata: *const ::core::ffi::c_void, cbdata: u32, dwflags: u32) -> super::super::Foundation::WIN32_ERROR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHReleaseThreadRef() -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHRemoveLocalizedName(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Controls\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
    pub fn SHReplaceFromPropSheetExtArray(hpsxa: HPSXA, upageid: u32, lpfnreplacewith: super::Controls::LPFNSVADDPROPSHEETPAGE, lparam: super::super::Foundation::LPARAM) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHResolveLibrary(psilibrary: IShellItem) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHRestricted(rest: RESTRICTIONS) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHSendMessageBroadcastA(umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHSendMessageBroadcastW(umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHSetDefaultProperties(hwnd: super::super::Foundation::HWND, psi: IShellItem, dwfileopflags: u32, pfops: IFileOperationProgressSink) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHSetFolderPathA(csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, pszpath: ::windows_sys::core::PCSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHSetFolderPathW(csidl: i32, htoken: super::super::Foundation::HANDLE, dwflags: u32, pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHSetInstanceExplorer(punk: ::windows_sys::core::IUnknown);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHSetKnownFolderPath(rfid: *const ::windows_sys::core::GUID, dwflags: u32, htoken: super::super::Foundation::HANDLE, pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHSetLocalizedName(pszpath: ::windows_sys::core::PCWSTR, pszresmodule: ::windows_sys::core::PCWSTR, idsres: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_PropertiesSystem"))]
    pub fn SHSetTemporaryPropertyForItem(psi: IShellItem, propkey: *const PropertiesSystem::PROPERTYKEY, propvar: *const super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHSetThreadRef(punk: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHSetUnreadMailCountW(pszmailaddress: ::windows_sys::core::PCWSTR, dwcount: u32, pszshellexecutecommand: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn SHSetValueA(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCSTR, pszvalue: ::windows_sys::core::PCSTR, dwtype: u32, pvdata: *const ::core::ffi::c_void, cbdata: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(feature = "Win32_System_Registry")]
    pub fn SHSetValueW(hkey: super::super::System::Registry::HKEY, pszsubkey: ::windows_sys::core::PCWSTR, pszvalue: ::windows_sys::core::PCWSTR, dwtype: u32, pvdata: *const ::core::ffi::c_void, cbdata: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHShellFolderView_Message(hwndmain: super::super::Foundation::HWND, umsg: u32, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHShowManageLibraryUI(psilibrary: IShellItem, hwndowner: super::super::Foundation::HWND, psztitle: ::windows_sys::core::PCWSTR, pszinstruction: ::windows_sys::core::PCWSTR, lmdoptions: LIBRARYMANAGEDIALOGOPTIONS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHSimpleIDListFromPath(pszpath: ::windows_sys::core::PCWSTR) -> *mut Common::ITEMIDLIST;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn SHSkipJunction(pbc: super::super::System::Com::IBindCtx, pclsid: *const ::windows_sys::core::GUID) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHStartNetConnectionDialogW(hwnd: super::super::Foundation::HWND, pszremotename: ::windows_sys::core::PCWSTR, dwtype: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHStrDupA(psz: ::windows_sys::core::PCSTR, ppwsz: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHStrDupW(psz: ::windows_sys::core::PCWSTR, ppwsz: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHStripMneumonicA(pszmenu: ::windows_sys::core::PSTR) -> super::super::Foundation::CHAR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHStripMneumonicW(pszmenu: ::windows_sys::core::PWSTR) -> u16;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHTestTokenMembership(htoken: super::super::Foundation::HANDLE, ulrid: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHUnicodeToAnsi(pwszsrc: ::windows_sys::core::PCWSTR, pszdst: ::windows_sys::core::PSTR, cchbuf: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHUnicodeToUnicode(pwzsrc: ::windows_sys::core::PCWSTR, pwzdst: ::windows_sys::core::PWSTR, cwchbuf: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHUnlockShared(pvdata: *const ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHUpdateImageA(pszhashitem: ::windows_sys::core::PCSTR, iindex: i32, uflags: u32, iimageindex: i32);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SHUpdateImageW(pszhashitem: ::windows_sys::core::PCWSTR, iindex: i32, uflags: u32, iimageindex: i32);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHValidateUNC(hwndowner: super::super::Foundation::HWND, pszfile: ::windows_sys::core::PWSTR, fconnect: VALIDATEUNC_OPTION) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn SetCurrentProcessExplicitAppUserModelID(appid: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SetMenuContextHelpId(param0: super::WindowsAndMessaging::HMENU, param1: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetWindowContextHelpId(param0: super::super::Foundation::HWND, param1: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetWindowSubclass(hwnd: super::super::Foundation::HWND, pfnsubclass: SUBCLASSPROC, uidsubclass: usize, dwrefdata: usize) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ShellAboutA(hwnd: super::super::Foundation::HWND, szapp: ::windows_sys::core::PCSTR, szotherstuff: ::windows_sys::core::PCSTR, hicon: super::WindowsAndMessaging::HICON) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ShellAboutW(hwnd: super::super::Foundation::HWND, szapp: ::windows_sys::core::PCWSTR, szotherstuff: ::windows_sys::core::PCWSTR, hicon: super::WindowsAndMessaging::HICON) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ShellExecuteA(hwnd: super::super::Foundation::HWND, lpoperation: ::windows_sys::core::PCSTR, lpfile: ::windows_sys::core::PCSTR, lpparameters: ::windows_sys::core::PCSTR, lpdirectory: ::windows_sys::core::PCSTR, nshowcmd: i32) -> super::super::Foundation::HINSTANCE;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn ShellExecuteExA(pexecinfo: *mut SHELLEXECUTEINFOA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
    pub fn ShellExecuteExW(pexecinfo: *mut SHELLEXECUTEINFOW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ShellExecuteW(hwnd: super::super::Foundation::HWND, lpoperation: ::windows_sys::core::PCWSTR, lpfile: ::windows_sys::core::PCWSTR, lpparameters: ::windows_sys::core::PCWSTR, lpdirectory: ::windows_sys::core::PCWSTR, nshowcmd: i32) -> super::super::Foundation::HINSTANCE;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ShellMessageBoxA(happinst: super::super::Foundation::HINSTANCE, hwnd: super::super::Foundation::HWND, lpctext: ::windows_sys::core::PCSTR, lpctitle: ::windows_sys::core::PCSTR, fustyle: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ShellMessageBoxW(happinst: super::super::Foundation::HINSTANCE, hwnd: super::super::Foundation::HWND, lpctext: ::windows_sys::core::PCWSTR, lpctitle: ::windows_sys::core::PCWSTR, fustyle: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn Shell_GetCachedImageIndex(pwsziconpath: ::windows_sys::core::PCWSTR, iiconindex: i32, uiconflags: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn Shell_GetCachedImageIndexA(psziconpath: ::windows_sys::core::PCSTR, iiconindex: i32, uiconflags: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn Shell_GetCachedImageIndexW(psziconpath: ::windows_sys::core::PCWSTR, iiconindex: i32, uiconflags: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Controls\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
    pub fn Shell_GetImageLists(phiml: *mut super::Controls::HIMAGELIST, phimlsmall: *mut super::Controls::HIMAGELIST) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn Shell_MergeMenus(hmdst: super::WindowsAndMessaging::HMENU, hmsrc: super::WindowsAndMessaging::HMENU, uinsert: u32, uidadjust: u32, uidadjustmax: u32, uflags: MM_FLAGS) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn Shell_NotifyIconA(dwmessage: NOTIFY_ICON_MESSAGE, lpdata: *const NOTIFYICONDATAA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn Shell_NotifyIconGetRect(identifier: *const NOTIFYICONIDENTIFIER, iconlocation: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn Shell_NotifyIconW(dwmessage: NOTIFY_ICON_MESSAGE, lpdata: *const NOTIFYICONDATAW) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn SignalFileOpen(pidl: *const Common::ITEMIDLIST) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_Urlmon\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_Urlmon"))]
    pub fn SoftwareUpdateMessageBox(hwnd: super::super::Foundation::HWND, pszdistunit: ::windows_sys::core::PCWSTR, dwflags: u32, psdi: *mut super::super::System::Com::Urlmon::SOFTDISTINFO) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn StgMakeUniqueName(pstgparent: super::super::System::Com::StructuredStorage::IStorage, pszfilespec: ::windows_sys::core::PCWSTR, grfmode: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCSpnA(pszstr: ::windows_sys::core::PCSTR, pszset: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCSpnIA(pszstr: ::windows_sys::core::PCSTR, pszset: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCSpnIW(pszstr: ::windows_sys::core::PCWSTR, pszset: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCSpnW(pszstr: ::windows_sys::core::PCWSTR, pszset: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCatBuffA(pszdest: ::windows_sys::core::PSTR, pszsrc: ::windows_sys::core::PCSTR, cchdestbuffsize: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCatBuffW(pszdest: ::windows_sys::core::PWSTR, pszsrc: ::windows_sys::core::PCWSTR, cchdestbuffsize: i32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCatChainW(pszdst: ::windows_sys::core::PWSTR, cchdst: u32, ichat: u32, pszsrc: ::windows_sys::core::PCWSTR) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCatW(psz1: ::windows_sys::core::PWSTR, psz2: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrChrA(pszstart: ::windows_sys::core::PCSTR, wmatch: u16) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrChrIA(pszstart: ::windows_sys::core::PCSTR, wmatch: u16) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrChrIW(pszstart: ::windows_sys::core::PCWSTR, wmatch: u16) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrChrNIW(pszstart: ::windows_sys::core::PCWSTR, wmatch: u16, cchmax: u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrChrNW(pszstart: ::windows_sys::core::PCWSTR, wmatch: u16, cchmax: u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrChrW(pszstart: ::windows_sys::core::PCWSTR, wmatch: u16) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpCA(pszstr1: ::windows_sys::core::PCSTR, pszstr2: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpCW(pszstr1: ::windows_sys::core::PCWSTR, pszstr2: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpICA(pszstr1: ::windows_sys::core::PCSTR, pszstr2: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpICW(pszstr1: ::windows_sys::core::PCWSTR, pszstr2: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpIW(psz1: ::windows_sys::core::PCWSTR, psz2: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpLogicalW(psz1: ::windows_sys::core::PCWSTR, psz2: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNA(psz1: ::windows_sys::core::PCSTR, psz2: ::windows_sys::core::PCSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNCA(pszstr1: ::windows_sys::core::PCSTR, pszstr2: ::windows_sys::core::PCSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNCW(pszstr1: ::windows_sys::core::PCWSTR, pszstr2: ::windows_sys::core::PCWSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNIA(psz1: ::windows_sys::core::PCSTR, psz2: ::windows_sys::core::PCSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNICA(pszstr1: ::windows_sys::core::PCSTR, pszstr2: ::windows_sys::core::PCSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNICW(pszstr1: ::windows_sys::core::PCWSTR, pszstr2: ::windows_sys::core::PCWSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNIW(psz1: ::windows_sys::core::PCWSTR, psz2: ::windows_sys::core::PCWSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpNW(psz1: ::windows_sys::core::PCWSTR, psz2: ::windows_sys::core::PCWSTR, nchar: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCmpW(psz1: ::windows_sys::core::PCWSTR, psz2: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCpyNW(pszdst: ::windows_sys::core::PWSTR, pszsrc: ::windows_sys::core::PCWSTR, cchmax: i32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrCpyW(psz1: ::windows_sys::core::PWSTR, psz2: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrDupA(pszsrch: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrDupW(pszsrch: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFormatByteSize64A(qdw: i64, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFormatByteSizeA(dw: u32, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFormatByteSizeEx(ull: u64, flags: SFBS_FLAGS, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFormatByteSizeW(qdw: i64, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFormatKBSizeA(qdw: i64, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFormatKBSizeW(qdw: i64, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFromTimeIntervalA(pszout: ::windows_sys::core::PSTR, cchmax: u32, dwtimems: u32, digits: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrFromTimeIntervalW(pszout: ::windows_sys::core::PWSTR, cchmax: u32, dwtimems: u32, digits: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrIsIntlEqualA(fcasesens: super::super::Foundation::BOOL, pszstring1: ::windows_sys::core::PCSTR, pszstring2: ::windows_sys::core::PCSTR, nchar: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrIsIntlEqualW(fcasesens: super::super::Foundation::BOOL, pszstring1: ::windows_sys::core::PCWSTR, pszstring2: ::windows_sys::core::PCWSTR, nchar: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrNCatA(psz1: ::windows_sys::core::PSTR, psz2: ::windows_sys::core::PCSTR, cchmax: i32) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrNCatW(psz1: ::windows_sys::core::PWSTR, psz2: ::windows_sys::core::PCWSTR, cchmax: i32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrPBrkA(psz: ::windows_sys::core::PCSTR, pszset: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrPBrkW(psz: ::windows_sys::core::PCWSTR, pszset: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrRChrA(pszstart: ::windows_sys::core::PCSTR, pszend: ::windows_sys::core::PCSTR, wmatch: u16) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrRChrIA(pszstart: ::windows_sys::core::PCSTR, pszend: ::windows_sys::core::PCSTR, wmatch: u16) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrRChrIW(pszstart: ::windows_sys::core::PCWSTR, pszend: ::windows_sys::core::PCWSTR, wmatch: u16) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrRChrW(pszstart: ::windows_sys::core::PCWSTR, pszend: ::windows_sys::core::PCWSTR, wmatch: u16) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrRStrIA(pszsource: ::windows_sys::core::PCSTR, pszlast: ::windows_sys::core::PCSTR, pszsrch: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrRStrIW(pszsource: ::windows_sys::core::PCWSTR, pszlast: ::windows_sys::core::PCWSTR, pszsrch: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
    pub fn StrRetToBSTR(pstr: *mut Common::STRRET, pidl: *const Common::ITEMIDLIST, pbstr: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn StrRetToBufA(pstr: *mut Common::STRRET, pidl: *const Common::ITEMIDLIST, pszbuf: ::windows_sys::core::PSTR, cchbuf: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn StrRetToBufW(pstr: *mut Common::STRRET, pidl: *const Common::ITEMIDLIST, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn StrRetToStrA(pstr: *mut Common::STRRET, pidl: *const Common::ITEMIDLIST, ppsz: *mut ::windows_sys::core::PSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn StrRetToStrW(pstr: *mut Common::STRRET, pidl: *const Common::ITEMIDLIST, ppsz: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrSpnA(psz: ::windows_sys::core::PCSTR, pszset: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrSpnW(psz: ::windows_sys::core::PCWSTR, pszset: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrStrA(pszfirst: ::windows_sys::core::PCSTR, pszsrch: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrStrIA(pszfirst: ::windows_sys::core::PCSTR, pszsrch: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrStrIW(pszfirst: ::windows_sys::core::PCWSTR, pszsrch: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrStrNIW(pszfirst: ::windows_sys::core::PCWSTR, pszsrch: ::windows_sys::core::PCWSTR, cchmax: u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrStrNW(pszfirst: ::windows_sys::core::PCWSTR, pszsrch: ::windows_sys::core::PCWSTR, cchmax: u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrStrW(pszfirst: ::windows_sys::core::PCWSTR, pszsrch: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrToInt64ExA(pszstring: ::windows_sys::core::PCSTR, dwflags: i32, pllret: *mut i64) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrToInt64ExW(pszstring: ::windows_sys::core::PCWSTR, dwflags: i32, pllret: *mut i64) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrToIntA(pszsrc: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrToIntExA(pszstring: ::windows_sys::core::PCSTR, dwflags: i32, piret: *mut i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrToIntExW(pszstring: ::windows_sys::core::PCWSTR, dwflags: i32, piret: *mut i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn StrToIntW(pszsrc: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrTrimA(psz: ::windows_sys::core::PSTR, psztrimchars: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn StrTrimW(psz: ::windows_sys::core::PWSTR, psztrimchars: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UnloadUserProfile(htoken: super::super::Foundation::HANDLE, hprofile: super::super::Foundation::HANDLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UnregisterAppConstrainedChangeNotification(registration: *mut _APPCONSTRAIN_REGISTRATION);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UnregisterAppStateChangeNotification(registration: *mut _APPSTATE_REGISTRATION);
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UnregisterScaleChangeEvent(dwcookie: usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlApplySchemeA(pszin: ::windows_sys::core::PCSTR, pszout: ::windows_sys::core::PSTR, pcchout: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlApplySchemeW(pszin: ::windows_sys::core::PCWSTR, pszout: ::windows_sys::core::PWSTR, pcchout: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlCanonicalizeA(pszurl: ::windows_sys::core::PCSTR, pszcanonicalized: ::windows_sys::core::PSTR, pcchcanonicalized: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlCanonicalizeW(pszurl: ::windows_sys::core::PCWSTR, pszcanonicalized: ::windows_sys::core::PWSTR, pcchcanonicalized: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlCombineA(pszbase: ::windows_sys::core::PCSTR, pszrelative: ::windows_sys::core::PCSTR, pszcombined: ::windows_sys::core::PSTR, pcchcombined: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlCombineW(pszbase: ::windows_sys::core::PCWSTR, pszrelative: ::windows_sys::core::PCWSTR, pszcombined: ::windows_sys::core::PWSTR, pcchcombined: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlCompareA(psz1: ::windows_sys::core::PCSTR, psz2: ::windows_sys::core::PCSTR, fignoreslash: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlCompareW(psz1: ::windows_sys::core::PCWSTR, psz2: ::windows_sys::core::PCWSTR, fignoreslash: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlCreateFromPathA(pszpath: ::windows_sys::core::PCSTR, pszurl: ::windows_sys::core::PSTR, pcchurl: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlCreateFromPathW(pszpath: ::windows_sys::core::PCWSTR, pszurl: ::windows_sys::core::PWSTR, pcchurl: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlEscapeA(pszurl: ::windows_sys::core::PCSTR, pszescaped: ::windows_sys::core::PSTR, pcchescaped: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlEscapeW(pszurl: ::windows_sys::core::PCWSTR, pszescaped: ::windows_sys::core::PWSTR, pcchescaped: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlFixupW(pcszurl: ::windows_sys::core::PCWSTR, psztranslatedurl: ::windows_sys::core::PWSTR, cchmax: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlGetLocationA(pszurl: ::windows_sys::core::PCSTR) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlGetLocationW(pszurl: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlGetPartA(pszin: ::windows_sys::core::PCSTR, pszout: ::windows_sys::core::PSTR, pcchout: *mut u32, dwpart: u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlGetPartW(pszin: ::windows_sys::core::PCWSTR, pszout: ::windows_sys::core::PWSTR, pcchout: *mut u32, dwpart: u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlHashA(pszurl: ::windows_sys::core::PCSTR, pbhash: *mut u8, cbhash: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlHashW(pszurl: ::windows_sys::core::PCWSTR, pbhash: *mut u8, cbhash: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlIsA(pszurl: ::windows_sys::core::PCSTR, urlis: URLIS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlIsNoHistoryA(pszurl: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlIsNoHistoryW(pszurl: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlIsOpaqueA(pszurl: ::windows_sys::core::PCSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlIsOpaqueW(pszurl: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UrlIsW(pszurl: ::windows_sys::core::PCWSTR, urlis: URLIS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlUnescapeA(pszurl: ::windows_sys::core::PSTR, pszunescaped: ::windows_sys::core::PSTR, pcchunescaped: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn UrlUnescapeW(pszurl: ::windows_sys::core::PWSTR, pszunescaped: ::windows_sys::core::PWSTR, pcchunescaped: *mut u32, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn WhichPlatform() -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn Win32DeleteFile(pszpath: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WinHelpA(hwndmain: super::super::Foundation::HWND, lpszhelp: ::windows_sys::core::PCSTR, ucommand: u32, dwdata: usize) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WinHelpW(hwndmain: super::super::Foundation::HWND, lpszhelp: ::windows_sys::core::PCWSTR, ucommand: u32, dwdata: usize) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WriteCabinetState(pcs: *const CABINETSTATE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn wnsprintfA(pszdest: ::windows_sys::core::PSTR, cchdest: i32, pszfmt: ::windows_sys::core::PCSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn wnsprintfW(pszdest: ::windows_sys::core::PWSTR, cchdest: i32, pszfmt: ::windows_sys::core::PCWSTR) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn wvnsprintfA(pszdest: ::windows_sys::core::PSTR, cchdest: i32, pszfmt: ::windows_sys::core::PCSTR, arglist: *const i8) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
    pub fn wvnsprintfW(pszdest: ::windows_sys::core::PWSTR, cchdest: i32, pszfmt: ::windows_sys::core::PCWSTR, arglist: *const i8) -> i32;
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct AASHELLMENUFILENAME {
    pub cbTotal: i16,
    pub rgbReserved: [u8; 12],
    pub szFileName: [u16; 1],
}
impl ::core::marker::Copy for AASHELLMENUFILENAME {}
impl ::core::clone::Clone for AASHELLMENUFILENAME {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct AASHELLMENUITEM {
    pub lpReserved1: *mut ::core::ffi::c_void,
    pub iReserved: i32,
    pub uiReserved: u32,
    pub lpName: *mut AASHELLMENUFILENAME,
    pub psz: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for AASHELLMENUITEM {}
impl ::core::clone::Clone for AASHELLMENUITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABE_BOTTOM: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABE_LEFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABE_RIGHT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABE_TOP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_ACTIVATE: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_GETAUTOHIDEBAR: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_GETAUTOHIDEBAREX: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_GETSTATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_GETTASKBARPOS: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_NEW: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_QUERYPOS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_REMOVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_SETAUTOHIDEBAR: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_SETAUTOHIDEBAREX: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_SETPOS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_SETSTATE: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABM_WINDOWPOSCHANGED: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABN_FULLSCREENAPP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABN_POSCHANGED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABN_STATECHANGE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABN_WINDOWARRANGE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABS_ALWAYSONTOP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ABS_AUTOHIDE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACDD_VISIBLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ACENUMOPTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACEO_NONE: ACENUMOPTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACEO_MOSTRECENTFIRST: ACENUMOPTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACEO_FIRSTUNUSED: ACENUMOPTION = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ACTIVATEOPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AO_NONE: ACTIVATEOPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AO_DESIGNMODE: ACTIVATEOPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AO_NOERRORUI: ACTIVATEOPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AO_NOSPLASHSCREEN: ACTIVATEOPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AO_PRELAUNCH: ACTIVATEOPTIONS = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ADDURL_SILENT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ADJACENT_DISPLAY_EDGES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ADE_NONE: ADJACENT_DISPLAY_EDGES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ADE_LEFT: ADJACENT_DISPLAY_EDGES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ADE_RIGHT: ADJACENT_DISPLAY_EDGES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_APPLY_BUFFERED_REFRESH: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_APPLY_DYNAMICREFRESH: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_APPLY_FORCE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_APPLY_HTMLGEN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_APPLY_REFRESH: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_APPLY_SAVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_GETWP_BMP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_GETWP_IMAGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AD_GETWP_LAST_APPLIED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type AHE_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHE_DESKTOP: AHE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHE_IMMERSIVE: AHE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type AHTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_UNDEFINED: AHTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_USER_APPLICATION: AHTYPE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_ANY_APPLICATION: AHTYPE = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_MACHINEDEFAULT: AHTYPE = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_PROGID: AHTYPE = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_APPLICATION: AHTYPE = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_CLASS_APPLICATION: AHTYPE = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AHTYPE_ANY_PROGID: AHTYPE = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPACTIONFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_INSTALL: APPACTIONFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_UNINSTALL: APPACTIONFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_MODIFY: APPACTIONFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_REPAIR: APPACTIONFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_UPGRADE: APPACTIONFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_CANGETSIZE: APPACTIONFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_MODIFYREMOVE: APPACTIONFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_ADDLATER: APPACTIONFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPACTION_UNSCHEDULE: APPACTIONFLAGS = 512i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct APPBARDATA {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uCallbackMessage: u32,
    pub uEdge: u32,
    pub rc: super::super::Foundation::RECT,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for APPBARDATA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for APPBARDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct APPBARDATA {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uCallbackMessage: u32,
    pub uEdge: u32,
    pub rc: super::super::Foundation::RECT,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for APPBARDATA {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for APPBARDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct APPCATEGORYINFO {
    pub Locale: u32,
    pub pszDescription: ::windows_sys::core::PWSTR,
    pub AppCategoryId: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for APPCATEGORYINFO {}
impl ::core::clone::Clone for APPCATEGORYINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct APPCATEGORYINFOLIST {
    pub cCategory: u32,
    pub pCategoryInfo: *mut APPCATEGORYINFO,
}
impl ::core::marker::Copy for APPCATEGORYINFOLIST {}
impl ::core::clone::Clone for APPCATEGORYINFOLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPDOCLISTTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ADLT_RECENT: APPDOCLISTTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ADLT_FREQUENT: APPDOCLISTTYPE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct APPINFODATA {
    pub cbSize: u32,
    pub dwMask: u32,
    pub pszDisplayName: ::windows_sys::core::PWSTR,
    pub pszVersion: ::windows_sys::core::PWSTR,
    pub pszPublisher: ::windows_sys::core::PWSTR,
    pub pszProductID: ::windows_sys::core::PWSTR,
    pub pszRegisteredOwner: ::windows_sys::core::PWSTR,
    pub pszRegisteredCompany: ::windows_sys::core::PWSTR,
    pub pszLanguage: ::windows_sys::core::PWSTR,
    pub pszSupportUrl: ::windows_sys::core::PWSTR,
    pub pszSupportTelephone: ::windows_sys::core::PWSTR,
    pub pszHelpLink: ::windows_sys::core::PWSTR,
    pub pszInstallLocation: ::windows_sys::core::PWSTR,
    pub pszInstallSource: ::windows_sys::core::PWSTR,
    pub pszInstallDate: ::windows_sys::core::PWSTR,
    pub pszContact: ::windows_sys::core::PWSTR,
    pub pszComments: ::windows_sys::core::PWSTR,
    pub pszImage: ::windows_sys::core::PWSTR,
    pub pszReadmeUrl: ::windows_sys::core::PWSTR,
    pub pszUpdateInfoUrl: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for APPINFODATA {}
impl ::core::clone::Clone for APPINFODATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPINFODATAFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_DISPLAYNAME: APPINFODATAFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_VERSION: APPINFODATAFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_PUBLISHER: APPINFODATAFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_PRODUCTID: APPINFODATAFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_REGISTEREDOWNER: APPINFODATAFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_REGISTEREDCOMPANY: APPINFODATAFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_LANGUAGE: APPINFODATAFLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_SUPPORTURL: APPINFODATAFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_SUPPORTTELEPHONE: APPINFODATAFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_HELPLINK: APPINFODATAFLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_INSTALLLOCATION: APPINFODATAFLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_INSTALLSOURCE: APPINFODATAFLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_INSTALLDATE: APPINFODATAFLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_CONTACT: APPINFODATAFLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_COMMENTS: APPINFODATAFLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_IMAGE: APPINFODATAFLAGS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_READMEURL: APPINFODATAFLAGS = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AIM_UPDATEINFOURL: APPINFODATAFLAGS = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type APPLET_PROC = ::core::option::Option<unsafe extern "system" fn(hwndcpl: super::super::Foundation::HWND, msg: u32, lparam1: super::super::Foundation::LPARAM, lparam2: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPLICATION_VIEW_MIN_WIDTH = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVMW_DEFAULT: APPLICATION_VIEW_MIN_WIDTH = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVMW_320: APPLICATION_VIEW_MIN_WIDTH = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVMW_500: APPLICATION_VIEW_MIN_WIDTH = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPLICATION_VIEW_ORIENTATION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVO_LANDSCAPE: APPLICATION_VIEW_ORIENTATION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVO_PORTRAIT: APPLICATION_VIEW_ORIENTATION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPLICATION_VIEW_SIZE_PREFERENCE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_DEFAULT: APPLICATION_VIEW_SIZE_PREFERENCE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_USE_LESS: APPLICATION_VIEW_SIZE_PREFERENCE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_USE_HALF: APPLICATION_VIEW_SIZE_PREFERENCE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_USE_MORE: APPLICATION_VIEW_SIZE_PREFERENCE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_USE_MINIMUM: APPLICATION_VIEW_SIZE_PREFERENCE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_USE_NONE: APPLICATION_VIEW_SIZE_PREFERENCE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVSP_CUSTOM: APPLICATION_VIEW_SIZE_PREFERENCE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type APPLICATION_VIEW_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVS_FULLSCREEN_LANDSCAPE: APPLICATION_VIEW_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVS_FILLED: APPLICATION_VIEW_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVS_SNAPPED: APPLICATION_VIEW_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AVS_FULLSCREEN_PORTRAIT: APPLICATION_VIEW_STATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const APPNAMEBUFFERLEN: u32 = 40u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_AUDIOCD: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_AUTOPLAYMUSIC: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_AUTOPLAYPIX: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_AUTOPLAYVIDEO: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_AUTORUNINF: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_BLANKBD: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_BLANKCD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_BLANKDVD: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_BLURAY: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_CAMERASTORAGE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_CUSTOMEVENT: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_DVDAUDIO: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_DVDMOVIE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_MASK: u32 = 131070u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_PHASE_FINAL: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_PHASE_MASK: u32 = 1879048192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_PHASE_PRESNIFF: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_PHASE_SNIFFING: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_PHASE_UNKNOWN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_SVCD: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_UNKNOWNCONTENT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ARCONTENT_VCD: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCCLASS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_SHELL_KEY: ASSOCCLASS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_PROGID_KEY: ASSOCCLASS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_PROGID_STR: ASSOCCLASS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_CLSID_KEY: ASSOCCLASS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_CLSID_STR: ASSOCCLASS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_APP_KEY: ASSOCCLASS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_APP_STR: ASSOCCLASS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_SYSTEM_STR: ASSOCCLASS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_FOLDER: ASSOCCLASS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_STAR: ASSOCCLASS = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_FIXED_PROGID_STR: ASSOCCLASS = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCCLASS_PROTOCOL_STR: ASSOCCLASS = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCDATA = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_MSIDESCRIPTOR: ASSOCDATA = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_NOACTIVATEHANDLER: ASSOCDATA = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_UNUSED1: ASSOCDATA = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_HASPERUSERASSOC: ASSOCDATA = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_EDITFLAGS: ASSOCDATA = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_VALUE: ASSOCDATA = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCDATA_MAX: ASSOCDATA = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCENUM = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCENUM_NONE: ASSOCENUM = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_APP_TO_APP: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_IGNOREBASECLASS: i32 = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_BYEXENAME: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_DEFAULTTOFOLDER: i32 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_DEFAULTTOSTAR: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_FIXED_PROGID: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_FOR_FILE: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_IGNOREUNKNOWN: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_INIT_NOREMAPCLSID: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_IS_FULL_URI: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_IS_PROTOCOL: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_NOFIXUPS: i32 = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_NONE: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_NOTRUNCATE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_NOUSERSETTINGS: i32 = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_OPEN_BYEXENAME: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_PER_MACHINE_ONLY: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_REMAPRUNDLL: i32 = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCF_VERIFY: i32 = 64i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_System_Registry")]
pub struct ASSOCIATIONELEMENT {
    pub ac: ASSOCCLASS,
    pub hkClass: super::super::System::Registry::HKEY,
    pub pszClass: ::windows_sys::core::PCWSTR,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_System_Registry")]
impl ::core::marker::Copy for ASSOCIATIONELEMENT {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_System_Registry")]
impl ::core::clone::Clone for ASSOCIATIONELEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Registry\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_System_Registry")]
pub struct ASSOCIATIONELEMENT {
    pub ac: ASSOCCLASS,
    pub hkClass: super::super::System::Registry::HKEY,
    pub pszClass: ::windows_sys::core::PCWSTR,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_System_Registry")]
impl ::core::marker::Copy for ASSOCIATIONELEMENT {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_System_Registry")]
impl ::core::clone::Clone for ASSOCIATIONELEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCIATIONLEVEL = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AL_MACHINE: ASSOCIATIONLEVEL = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AL_EFFECTIVE: ASSOCIATIONLEVEL = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AL_USER: ASSOCIATIONLEVEL = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCIATIONTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AT_FILEEXTENSION: ASSOCIATIONTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AT_URLPROTOCOL: ASSOCIATIONTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AT_STARTMENUCLIENT: ASSOCIATIONTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AT_MIMETYPE: ASSOCIATIONTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCKEY = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCKEY_SHELLEXECCLASS: ASSOCKEY = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCKEY_APP: ASSOCKEY = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCKEY_CLASS: ASSOCKEY = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCKEY_BASECLASS: ASSOCKEY = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCKEY_MAX: ASSOCKEY = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOCSTR = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_COMMAND: ASSOCSTR = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_EXECUTABLE: ASSOCSTR = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_FRIENDLYDOCNAME: ASSOCSTR = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_FRIENDLYAPPNAME: ASSOCSTR = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_NOOPEN: ASSOCSTR = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_SHELLNEWVALUE: ASSOCSTR = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DDECOMMAND: ASSOCSTR = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DDEIFEXEC: ASSOCSTR = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DDEAPPLICATION: ASSOCSTR = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DDETOPIC: ASSOCSTR = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_INFOTIP: ASSOCSTR = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_QUICKTIP: ASSOCSTR = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_TILEINFO: ASSOCSTR = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_CONTENTTYPE: ASSOCSTR = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DEFAULTICON: ASSOCSTR = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_SHELLEXTENSION: ASSOCSTR = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DROPTARGET: ASSOCSTR = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_DELEGATEEXECUTE: ASSOCSTR = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_SUPPORTED_URI_PROTOCOLS: ASSOCSTR = 19i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_PROGID: ASSOCSTR = 20i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_APPID: ASSOCSTR = 21i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_APPPUBLISHER: ASSOCSTR = 22i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_APPICONREFERENCE: ASSOCSTR = 23i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOCSTR_MAX: ASSOCSTR = 24i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ASSOC_FILTER = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOC_FILTER_NONE: ASSOC_FILTER = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ASSOC_FILTER_RECOMMENDED: ASSOC_FILTER = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ATTACHMENT_ACTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_ACTION_CANCEL: ATTACHMENT_ACTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_ACTION_SAVE: ATTACHMENT_ACTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_ACTION_EXEC: ATTACHMENT_ACTION = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ATTACHMENT_PROMPT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_PROMPT_NONE: ATTACHMENT_PROMPT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_PROMPT_SAVE: ATTACHMENT_PROMPT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_PROMPT_EXEC: ATTACHMENT_PROMPT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ATTACHMENT_PROMPT_EXEC_OR_SAVE: ATTACHMENT_PROMPT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type AUTOCOMPLETELISTOPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_NONE: AUTOCOMPLETELISTOPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_CURRENTDIR: AUTOCOMPLETELISTOPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_MYCOMPUTER: AUTOCOMPLETELISTOPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_DESKTOP: AUTOCOMPLETELISTOPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_FAVORITES: AUTOCOMPLETELISTOPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_FILESYSONLY: AUTOCOMPLETELISTOPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_FILESYSDIRS: AUTOCOMPLETELISTOPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACLO_VIRTUALNAMESPACE: AUTOCOMPLETELISTOPTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type AUTOCOMPLETEOPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_NONE: AUTOCOMPLETEOPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_AUTOSUGGEST: AUTOCOMPLETEOPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_AUTOAPPEND: AUTOCOMPLETEOPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_SEARCH: AUTOCOMPLETEOPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_FILTERPREFIXES: AUTOCOMPLETEOPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_USETAB: AUTOCOMPLETEOPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_UPDOWNKEYDROPSLIST: AUTOCOMPLETEOPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_RTLREADING: AUTOCOMPLETEOPTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_WORD_FILTER: AUTOCOMPLETEOPTIONS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ACO_NOPREFIXFILTERING: AUTOCOMPLETEOPTIONS = 256i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct AUTO_SCROLL_DATA {
    pub iNextSample: i32,
    pub dwLastScroll: u32,
    pub bFull: super::super::Foundation::BOOL,
    pub pts: [super::super::Foundation::POINT; 3],
    pub dwTimes: [u32; 3],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for AUTO_SCROLL_DATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for AUTO_SCROLL_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
pub const AccessibilityDockingService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 701373766, data2: 46209, data3: 19104, data4: [160, 138, 211, 235, 200, 172, 164, 2] };
pub const AlphabeticalCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1009145030, data2: 29554, data3: 20331, data4: [179, 16, 85, 214, 18, 143, 73, 210] };
pub const AppShellVerbHandler: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1322493721, data2: 52904, data3: 19417, data4: [145, 13, 226, 82, 249, 151, 175, 194] };
pub const AppStartupLink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 658421223, data2: 34992, data3: 18499, data4: [191, 239, 226, 200, 29, 67, 170, 229] };
pub const AppVisibility: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2120213465, data2: 39007, data3: 18696, data4: [145, 249, 238, 25, 249, 253, 21, 20] };
pub const ApplicationActivationManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1169822333, data2: 4264, data3: 18154, data4: [138, 183, 86, 234, 144, 120, 148, 60] };
pub const ApplicationAssociationRegistration: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1494354375, data2: 30331, data3: 17074, data4: [159, 186, 68, 238, 70, 21, 242, 199] };
pub const ApplicationAssociationRegistrationUI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 426250349, data2: 62389, data3: 17615, data4: [137, 14, 17, 111, 203, 158, 206, 241] };
pub const ApplicationDesignModeSettings: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2508877749, data2: 56498, data3: 20399, data4: [170, 253, 127, 176, 84, 173, 26, 59] };
pub const ApplicationDestinations: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2260811779, data2: 19819, data3: 20211, data4: [167, 180, 5, 6, 102, 59, 46, 104] };
pub const ApplicationDocumentLists: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2260648482, data2: 12530, data3: 18400, data4: [159, 37, 96, 209, 28, 215, 92, 40] };
pub const AttachmentServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1093000598, data2: 57402, data3: 16643, data4: [143, 112, 224, 89, 125, 128, 59, 156] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(feature = "Win32_UI_Shell_Common")]
pub struct BANDINFOSFB {
    pub dwMask: u32,
    pub dwStateMask: u32,
    pub dwState: u32,
    pub crBkgnd: u32,
    pub crBtnLt: u32,
    pub crBtnDk: u32,
    pub wViewMode: u16,
    pub wAlign: u16,
    pub psf: IShellFolder,
    pub pidl: *mut Common::ITEMIDLIST,
}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::marker::Copy for BANDINFOSFB {}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::clone::Clone for BANDINFOSFB {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type BANDSITECID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSID_BANDADDED: BANDSITECID = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSID_BANDREMOVED: BANDSITECID = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct BANDSITEINFO {
    pub dwMask: u32,
    pub dwState: u32,
    pub dwStyle: u32,
}
impl ::core::marker::Copy for BANDSITEINFO {}
impl ::core::clone::Clone for BANDSITEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct BANNER_NOTIFICATION {
    pub event: BANNER_NOTIFICATION_EVENT,
    pub providerIdentity: ::windows_sys::core::PCWSTR,
    pub contentId: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for BANNER_NOTIFICATION {}
impl ::core::clone::Clone for BANNER_NOTIFICATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type BANNER_NOTIFICATION_EVENT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNE_Rendered: BANNER_NOTIFICATION_EVENT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNE_Hovered: BANNER_NOTIFICATION_EVENT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNE_Closed: BANNER_NOTIFICATION_EVENT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNE_Dismissed: BANNER_NOTIFICATION_EVENT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNE_Button1Clicked: BANNER_NOTIFICATION_EVENT = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNE_Button2Clicked: BANNER_NOTIFICATION_EVENT = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
pub struct BASEBROWSERDATALH {
    pub _hwnd: super::super::Foundation::HWND,
    pub _ptl: ITravelLog,
    pub _phlf: IHlinkFrame,
    pub _pautoWB2: IWebBrowser2,
    pub _pautoEDS: IExpDispSupport,
    pub _pautoSS: IShellService,
    pub _eSecureLockIcon: i32,
    pub _bitfield: u32,
    pub _uActivateState: u32,
    pub _pidlViewState: *mut Common::ITEMIDLIST,
    pub _pctView: super::super::System::Ole::IOleCommandTarget,
    pub _pidlCur: *mut Common::ITEMIDLIST,
    pub _psv: IShellView,
    pub _psf: IShellFolder,
    pub _hwndView: super::super::Foundation::HWND,
    pub _pszTitleCur: ::windows_sys::core::PWSTR,
    pub _pidlPending: *mut Common::ITEMIDLIST,
    pub _psvPending: IShellView,
    pub _psfPending: IShellFolder,
    pub _hwndViewPending: super::super::Foundation::HWND,
    pub _pszTitlePending: ::windows_sys::core::PWSTR,
    pub _fIsViewMSHTML: super::super::Foundation::BOOL,
    pub _fPrivacyImpacted: super::super::Foundation::BOOL,
    pub _clsidView: ::windows_sys::core::GUID,
    pub _clsidViewPending: ::windows_sys::core::GUID,
    pub _hwndFrame: super::super::Foundation::HWND,
    pub _lPhishingFilterStatus: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for BASEBROWSERDATALH {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for BASEBROWSERDATALH {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
pub struct BASEBROWSERDATAXP {
    pub _hwnd: super::super::Foundation::HWND,
    pub _ptl: ITravelLog,
    pub _phlf: IHlinkFrame,
    pub _pautoWB2: IWebBrowser2,
    pub _pautoEDS: IExpDispSupportXP,
    pub _pautoSS: IShellService,
    pub _eSecureLockIcon: i32,
    pub _bitfield: u32,
    pub _uActivateState: u32,
    pub _pidlViewState: *mut Common::ITEMIDLIST,
    pub _pctView: super::super::System::Ole::IOleCommandTarget,
    pub _pidlCur: *mut Common::ITEMIDLIST,
    pub _psv: IShellView,
    pub _psf: IShellFolder,
    pub _hwndView: super::super::Foundation::HWND,
    pub _pszTitleCur: ::windows_sys::core::PWSTR,
    pub _pidlPending: *mut Common::ITEMIDLIST,
    pub _psvPending: IShellView,
    pub _psfPending: IShellFolder,
    pub _hwndViewPending: super::super::Foundation::HWND,
    pub _pszTitlePending: ::windows_sys::core::PWSTR,
    pub _fIsViewMSHTML: super::super::Foundation::BOOL,
    pub _fPrivacyImpacted: super::super::Foundation::BOOL,
    pub _clsidView: ::windows_sys::core::GUID,
    pub _clsidViewPending: ::windows_sys::core::GUID,
    pub _hwndFrame: super::super::Foundation::HWND,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for BASEBROWSERDATAXP {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for BASEBROWSERDATAXP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type BFFCALLBACK = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, umsg: u32, lparam: super::super::Foundation::LPARAM, lpdata: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_ENABLEOK: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_INITIALIZED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_IUNKNOWN: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SELCHANGED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETEXPANDED: u32 = 1130u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETOKTEXT: u32 = 1129u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETSELECTION: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETSELECTIONA: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETSELECTIONW: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETSTATUSTEXT: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETSTATUSTEXTA: u32 = 1124u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_SETSTATUSTEXTW: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_VALIDATEFAILED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_VALIDATEFAILEDA: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFFM_VALIDATEFAILEDW: u32 = 4u32;
pub const BHID_AssociationArray: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3198807831, data2: 33521, data3: 20320, data4: [146, 132, 79, 141, 183, 92, 59, 233] };
pub const BHID_DataObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3099639199, data2: 60708, data3: 17756, data4: [131, 230, 213, 57, 12, 79, 232, 196] };
pub const BHID_EnumAssocHandlers: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3098217372, data2: 49900, data3: 20346, data4: [145, 141, 49, 73, 0, 230, 40, 10] };
pub const BHID_EnumItems: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2499151129, data2: 10320, data3: 18724, data4: [170, 90, 209, 94, 132, 134, 128, 57] };
pub const BHID_FilePlaceholder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2256002283, data2: 43744, data3: 16389, data4: [141, 61, 84, 127, 168, 82, 248, 37] };
pub const BHID_Filter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 953190264, data2: 62807, data3: 18064, data4: [158, 191, 186, 84, 112, 106, 216, 247] };
pub const BHID_LinkTargetItem: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 964813352, data2: 62809, data3: 4563, data4: [142, 58, 0, 192, 79, 104, 55, 213] };
pub const BHID_PropertyStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 59040164, data2: 5411, data3: 17308, data4: [164, 200, 171, 145, 16, 82, 245, 134] };
pub const BHID_RandomAccessStream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4050635067, data2: 30638, data3: 19710, data4: [189, 167, 168, 102, 238, 166, 135, 141] };
pub const BHID_SFObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 964813348, data2: 62809, data3: 4563, data4: [142, 58, 0, 192, 79, 104, 55, 213] };
pub const BHID_SFUIObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 964813349, data2: 62809, data3: 4563, data4: [142, 58, 0, 192, 79, 104, 55, 213] };
pub const BHID_SFViewObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 964813350, data2: 62809, data3: 4563, data4: [142, 58, 0, 192, 79, 104, 55, 213] };
pub const BHID_Storage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 964813351, data2: 62809, data3: 4563, data4: [142, 58, 0, 192, 79, 104, 55, 213] };
pub const BHID_StorageEnum: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1176610019, data2: 61654, data3: 18291, data4: [138, 156, 70, 231, 123, 23, 72, 64] };
pub const BHID_StorageItem: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1078862089, data2: 30674, data3: 18073, data4: [165, 160, 79, 223, 16, 219, 152, 55] };
pub const BHID_Stream: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 485209003, data2: 31760, data3: 18842, data4: [164, 23, 146, 202, 22, 196, 203, 131] };
pub const BHID_ThumbnailHandler: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2066638090, data2: 36384, data3: 20298, data4: [176, 158, 101, 151, 175, 199, 47, 176] };
pub const BHID_Transfer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3588441761, data2: 63315, data3: 18738, data4: [180, 3, 69, 116, 128, 14, 36, 152] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_BROWSEFILEJUNCTIONS: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_BROWSEFORCOMPUTER: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_BROWSEFORPRINTER: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_BROWSEINCLUDEFILES: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_BROWSEINCLUDEURLS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_DONTGOBELOWDOMAIN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_EDITBOX: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_NEWDIALOGSTYLE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_NONEWFOLDERBUTTON: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_NOTRANSLATETARGETS: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_RETURNFSANCESTORS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_RETURNONLYFSDIRS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_SHAREABLE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_STATUSTEXT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_UAHINT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_VALIDATE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIND_INTERRUPTABLE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BMICON_LARGE: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BMICON_SMALL: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type BNSTATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNS_NORMAL: BNSTATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNS_BEGIN_NAVIGATE: BNSTATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BNS_NAVIGATE: BNSTATE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
pub struct BROWSEINFOA {
    pub hwndOwner: super::super::Foundation::HWND,
    pub pidlRoot: *mut Common::ITEMIDLIST,
    pub pszDisplayName: ::windows_sys::core::PSTR,
    pub lpszTitle: ::windows_sys::core::PCSTR,
    pub ulFlags: u32,
    pub lpfn: BFFCALLBACK,
    pub lParam: super::super::Foundation::LPARAM,
    pub iImage: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for BROWSEINFOA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for BROWSEINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
pub struct BROWSEINFOW {
    pub hwndOwner: super::super::Foundation::HWND,
    pub pidlRoot: *mut Common::ITEMIDLIST,
    pub pszDisplayName: ::windows_sys::core::PWSTR,
    pub lpszTitle: ::windows_sys::core::PCWSTR,
    pub ulFlags: u32,
    pub lpfn: BFFCALLBACK,
    pub lParam: super::super::Foundation::LPARAM,
    pub iImage: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for BROWSEINFOW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for BROWSEINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_CANMAXIMIZE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_DELEGATEDNAVIGATION: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_DONTSHOWNAVCANCELPAGE: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_FEEDNAVIGATION: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_FEEDSUBSCRIBED: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_HTMLNAVCANCELED: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_MERGEDMENUS: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_NAVNOHISTORY: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_NOLOCALFILEWARNING: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_REGISTERASDROPTARGET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_RESIZABLE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_SETNAVIGATABLECODEPAGE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_THEATERMODE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_TOPBROWSER: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_TRUSTEDFORACTIVEX: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSF_UISETBYAUTOMATION: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIM_STATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIM_STYLE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_ALWAYSGRIPPER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_AUTOGRIPPER: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_FIXEDORDER: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_LEFTALIGN: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_LOCKED: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_NOCAPTION: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_NOCONTEXTMENU: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_NODROPTARGET: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_NOGRIPPER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_PREFERNOLINEBREAK: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_PRESERVEORDERDURINGLAYOUT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSIS_SINGLECLICK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSSF_NOTITLE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSSF_UNDELETEABLE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BSSF_VISIBLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BUFFLEN: u32 = 255u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type BrowserNavConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navOpenInNewWindow: BrowserNavConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navNoHistory: BrowserNavConstants = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navNoReadFromCache: BrowserNavConstants = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navNoWriteToCache: BrowserNavConstants = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navAllowAutosearch: BrowserNavConstants = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navBrowserBar: BrowserNavConstants = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navHyperlink: BrowserNavConstants = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navEnforceRestricted: BrowserNavConstants = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navNewWindowsManaged: BrowserNavConstants = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navUntrustedForDownload: BrowserNavConstants = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navTrustedForActiveX: BrowserNavConstants = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navOpenInNewTab: BrowserNavConstants = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navOpenInBackgroundTab: BrowserNavConstants = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navKeepWordWheelText: BrowserNavConstants = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navVirtualTab: BrowserNavConstants = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navBlockRedirectsXDomain: BrowserNavConstants = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navOpenNewForegroundTab: BrowserNavConstants = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navTravelLogScreenshot: BrowserNavConstants = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navDeferUnload: BrowserNavConstants = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navSpeculative: BrowserNavConstants = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navSuggestNewWindow: BrowserNavConstants = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navSuggestNewTab: BrowserNavConstants = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved1: BrowserNavConstants = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navHomepageNavigate: BrowserNavConstants = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navRefresh: BrowserNavConstants = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navHostNavigation: BrowserNavConstants = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved2: BrowserNavConstants = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved3: BrowserNavConstants = 134217728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved4: BrowserNavConstants = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved5: BrowserNavConstants = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved6: BrowserNavConstants = 1073741824i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const navReserved7: BrowserNavConstants = -2147483648i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CABINETSTATE {
    pub cLength: u16,
    pub nVersion: u16,
    pub _bitfield: i32,
    pub fMenuEnumFilter: u32,
}
impl ::core::marker::Copy for CABINETSTATE {}
impl ::core::clone::Clone for CABINETSTATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CABINETSTATE_VERSION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CAMERAROLL_E_NO_DOWNSAMPLING_REQUIRED: ::windows_sys::core::HRESULT = -2144927456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CATEGORYINFO_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_NORMAL: CATEGORYINFO_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_COLLAPSED: CATEGORYINFO_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_HIDDEN: CATEGORYINFO_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_EXPANDED: CATEGORYINFO_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_NOHEADER: CATEGORYINFO_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_NOTCOLLAPSIBLE: CATEGORYINFO_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_NOHEADERCOUNT: CATEGORYINFO_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_SUBSETTED: CATEGORYINFO_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_SEPARATE_IMAGES: CATEGORYINFO_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATINFO_SHOWEMPTY: CATEGORYINFO_FLAGS = 256i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CATEGORY_INFO {
    pub cif: CATEGORYINFO_FLAGS,
    pub wszName: [u16; 260],
}
impl ::core::marker::Copy for CATEGORY_INFO {}
impl ::core::clone::Clone for CATEGORY_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
pub const CATID_BrowsableShellExt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136336, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CATID_BrowseInPlace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136337, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CATID_CommBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136340, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CATID_DeskBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136338, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CATID_FilePlaceholderMergeHandler: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1050450513, data2: 54442, data3: 18544, data4: [180, 124, 116, 36, 180, 145, 241, 204] };
pub const CATID_InfoBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136339, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CATID_LocationFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2522631505, data2: 35702, data3: 20055, data4: [128, 183, 86, 77, 46, 164, 181, 94] };
pub const CATID_LocationProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 456959092, data2: 9748, data3: 16715, data4: [184, 19, 26, 206, 202, 62, 61, 216] };
pub const CATID_SearchableApplication: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 913058090, data2: 55731, data3: 19903, data4: [187, 112, 230, 46, 195, 208, 187, 191] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CATSORT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATSORT_DEFAULT: CATSORT_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CATSORT_NAME: CATSORT_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_ADDSHIELD: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_ALLOWPREVIEWPANE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_ISFILESAVE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_ISFOLDERPICKER: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_NOINCLUDEITEM: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_NOSELECTVERB: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2GVF_SHOWALLFILES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2N_CONTEXTMENU_DONE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDB2N_CONTEXTMENU_START: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBOSC_KILLFOCUS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBOSC_RENAME: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBOSC_SELCHANGE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBOSC_SETFOCUS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBOSC_STATECHANGE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CDBURNINGEXTENSIONRET = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBE_RET_DEFAULT: CDBURNINGEXTENSIONRET = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBE_RET_DONTRUNOTHEREXTS: CDBURNINGEXTENSIONRET = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBE_RET_STOPWIZARD: CDBURNINGEXTENSIONRET = 2i32;
pub const CDBurn: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4226517509, data2: 48878, data3: 17474, data4: [128, 78, 64, 157, 108, 69, 21, 233] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CDCONTROLSTATEF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDCS_INACTIVE: CDCONTROLSTATEF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDCS_ENABLED: CDCONTROLSTATEF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDCS_VISIBLE: CDCONTROLSTATEF = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDCS_ENABLEDVISIBLE: CDCONTROLSTATEF = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_AUTOPLAY_SHELLIDLISTS: &str = "Autoplay Enumerated IDList Array";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_DROPDESCRIPTION: &str = "DropDescription";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILECONTENTS: &str = "FileContents";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILEDESCRIPTOR: &str = "FileGroupDescriptorW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILEDESCRIPTORA: &str = "FileGroupDescriptor";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILEDESCRIPTORW: &str = "FileGroupDescriptorW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILENAME: &str = "FileNameW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILENAMEA: &str = "FileName";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILENAMEMAP: &str = "FileNameMapW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILENAMEMAPA: &str = "FileNameMap";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILENAMEMAPW: &str = "FileNameMapW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILENAMEW: &str = "FileNameW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_FILE_ATTRIBUTES_ARRAY: &str = "File Attributes Array";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_INDRAGLOOP: &str = "InShellDragLoop";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_INETURL: &str = "UniformResourceLocatorW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_INETURLA: &str = "UniformResourceLocator";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_INETURLW: &str = "UniformResourceLocatorW";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_INVOKECOMMAND_DROPPARAM: &str = "InvokeCommand DropParam";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_LOGICALPERFORMEDDROPEFFECT: &str = "Logical Performed DropEffect";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_MOUNTEDVOLUME: &str = "MountedVolume";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_NETRESOURCES: &str = "Net Resource";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_PASTESUCCEEDED: &str = "Paste Succeeded";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_PERFORMEDDROPEFFECT: &str = "Performed DropEffect";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_PERSISTEDDATAOBJECT: &str = "PersistedDataObject";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_PREFERREDDROPEFFECT: &str = "Preferred DropEffect";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_PRINTERGROUP: &str = "PrinterFriendlyName";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_SHELLDROPHANDLER: &str = "DropHandlerCLSID";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_SHELLIDLIST: &str = "Shell IDList Array";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_SHELLIDLISTOFFSET: &str = "Shell Object Offsets";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_SHELLURL: &str = "UniformResourceLocator";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_TARGETCLSID: &str = "TargetCLSID";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_UNTRUSTEDDRAGDROP: &str = "UntrustedDragDrop";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CFSTR_ZONEIDENTIFIER: &str = "ZoneIdentifier";
pub const CGID_DefView: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1257275152, data2: 53809, data3: 4560, data4: [185, 66, 0, 160, 201, 3, 18, 225] };
pub const CGID_Explorer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136400, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CGID_ExplorerBarDoc: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136403, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CGID_MENUDESKBAR: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1553926674, data2: 38302, data3: 4560, data4: [163, 164, 0, 160, 201, 8, 38, 54] };
pub const CGID_ShellDocView: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136401, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CGID_ShellServiceObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136402, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const CGID_ShortCut: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2477164368, data2: 38170, data3: 4561, data4: [148, 111, 0, 0, 0, 0, 0, 0] };
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CIDA {
    pub cidl: u32,
    pub aoffset: [u32; 1],
}
impl ::core::marker::Copy for CIDA {}
impl ::core::clone::Clone for CIDA {
    fn clone(&self) -> Self {
        *self
    }
}
pub type CIE4ConnectionPoint = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CLOSEPROPS_DISCARD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CLOSEPROPS_NONE: u32 = 0u32;
pub const CLSID_ACLCustomMRU: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1765137299, data2: 8680, data3: 19660, data4: [190, 185, 159, 227, 199, 122, 41, 122] };
pub const CLSID_ACLHistory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 12265316, data2: 27255, data3: 4560, data4: [165, 53, 0, 192, 79, 215, 208, 98] };
pub const CLSID_ACLMRU: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1733731905, data2: 56945, data3: 4560, data4: [131, 27, 0, 170, 0, 91, 67, 131] };
pub const CLSID_ACLMulti: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 12265317, data2: 27255, data3: 4560, data4: [165, 53, 0, 192, 79, 215, 208, 98] };
pub const CLSID_ACListISF: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 62928625, data2: 41350, data3: 4560, data4: [130, 74, 0, 170, 0, 91, 67, 131] };
pub const CLSID_ActiveDesktop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1963230976, data2: 61215, data3: 4560, data4: [152, 136, 0, 96, 151, 222, 172, 249] };
pub const CLSID_AutoComplete: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 12265315, data2: 27255, data3: 4560, data4: [165, 53, 0, 192, 79, 215, 208, 98] };
pub const CLSID_CAnchorBrowsePropertyPage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 810611643, data2: 39093, data3: 4559, data4: [187, 130, 0, 170, 0, 189, 206, 11] };
pub const CLSID_CDocBrowsePropertyPage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 810611636, data2: 39093, data3: 4559, data4: [187, 130, 0, 170, 0, 189, 206, 11] };
pub const CLSID_CFSIconOverlayManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1672814465, data2: 51304, data3: 4560, data4: [153, 156, 0, 192, 79, 214, 85, 225] };
pub const CLSID_CImageBrowsePropertyPage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 810611635, data2: 39093, data3: 4559, data4: [187, 130, 0, 170, 0, 189, 206, 11] };
pub const CLSID_CURLSearchHook: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3485445632, data2: 6054, data3: 4560, data4: [153, 203, 0, 192, 79, 214, 68, 151] };
pub const CLSID_CUrlHistory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1010256448, data2: 47844, data3: 4559, data4: [191, 125, 0, 170, 0, 105, 70, 238] };
pub const CLSID_ControlPanel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 569122848, data2: 15082, data3: 4201, data4: [162, 221, 8, 0, 43, 48, 48, 157] };
pub const CLSID_DarwinAppPublisher: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3486304160, data2: 41602, data3: 4561, data4: [144, 130, 0, 96, 8, 5, 147, 130] };
pub const CLSID_DocHostUIHandler: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1884809554, data2: 48411, data3: 4561, data4: [137, 25, 0, 192, 79, 194, 200, 54] };
pub const CLSID_DragDropHelper: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1180116874, data2: 16667, data3: 4562, data4: [131, 154, 0, 192, 79, 217, 24, 208] };
pub const CLSID_FileTypes: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2962351424, data2: 33763, data3: 4559, data4: [167, 19, 0, 32, 175, 215, 151, 98] };
pub const CLSID_FolderItemsMultiLevel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1405569062, data2: 43929, data3: 19763, data4: [172, 164, 49, 23, 245, 29, 55, 136] };
pub const CLSID_FolderShortcut: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 184209105, data2: 59432, data3: 4561, data4: [145, 135, 181, 50, 241, 233, 87, 93] };
pub const CLSID_HWShellExecute: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4290274655, data2: 33209, data3: 20430, data4: [184, 156, 154, 107, 167, 109, 19, 231] };
pub const CLSID_ISFBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3626754736, data2: 22372, data3: 4560, data4: [169, 110, 0, 192, 79, 215, 5, 162] };
pub const CLSID_Internet: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2266780544, data2: 17056, data3: 4201, data4: [162, 234, 8, 0, 43, 48, 48, 157] };
pub const CLSID_InternetButtons: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 511273344, data2: 40133, data3: 4561, data4: [168, 63, 0, 192, 79, 201, 157, 97] };
pub const CLSID_InternetShortcut: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4226956096, data2: 58352, data3: 4123, data4: [132, 136, 0, 170, 0, 62, 86, 248] };
pub const CLSID_LinkColumnProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 619794178, data2: 31516, data3: 4561, data4: [131, 143, 0, 0, 248, 4, 97, 207] };
pub const CLSID_MSOButtons: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 395261112, data2: 41602, data3: 4562, data4: [134, 197, 0, 192, 79, 142, 234, 153] };
pub const CLSID_MenuBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1531817510, data2: 47111, data3: 4560, data4: [152, 21, 0, 192, 79, 217, 25, 114] };
pub const CLSID_MenuBandSite: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3778999524, data2: 54002, data3: 4560, data4: [152, 22, 0, 192, 79, 217, 25, 114] };
pub const CLSID_MenuToolbarBase: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1085892112, data2: 46370, data3: 4561, data4: [179, 180, 0, 170, 0, 110, 253, 231] };
pub const CLSID_MyComputer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 550522848, data2: 15082, data3: 4201, data4: [162, 216, 8, 0, 43, 48, 48, 157] };
pub const CLSID_MyDocuments: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1158516666, data2: 44325, data3: 4560, data4: [152, 168, 8, 0, 54, 27, 17, 3] };
pub const CLSID_NetworkDomain: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1189111424, data2: 19440, data3: 4561, data4: [131, 238, 0, 160, 201, 13, 200, 73] };
pub const CLSID_NetworkServer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3226741392, data2: 19440, data3: 4561, data4: [131, 238, 0, 160, 201, 13, 200, 73] };
pub const CLSID_NetworkShare: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1420252352, data2: 19440, data3: 4561, data4: [131, 238, 0, 160, 201, 13, 200, 73] };
pub const CLSID_NewMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3647578880, data2: 59391, data3: 4560, data4: [169, 59, 0, 160, 201, 15, 39, 25] };
pub const CLSID_Printers: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 573022848, data2: 15082, data3: 4201, data4: [162, 222, 8, 0, 43, 48, 48, 157] };
pub const CLSID_ProgressDialog: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4164434002, data2: 64723, data3: 4561, data4: [166, 185, 0, 96, 151, 223, 91, 212] };
pub const CLSID_QueryAssociations: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2691708157, data2: 27818, data3: 18772, data4: [172, 63, 151, 162, 114, 22, 249, 138] };
pub const CLSID_QuickLinks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 240959265, data2: 53599, data3: 4560, data4: [131, 1, 0, 170, 0, 91, 67, 131] };
pub const CLSID_RecycleBin: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1684009024, data2: 20609, data3: 4123, data4: [159, 8, 0, 170, 0, 47, 149, 78] };
pub const CLSID_ShellFldSetExt: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1834161088, data2: 35938, data3: 4561, data4: [178, 205, 0, 96, 151, 223, 140, 17] };
pub const CLSID_ShellThumbnailDiskCache: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 515755904, data2: 41472, data3: 4560, data4: [163, 164, 0, 192, 79, 215, 6, 236] };
pub const CLSID_ToolbarExtButtons: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 753186264, data2: 41615, data3: 4562, data4: [134, 197, 0, 192, 79, 142, 234, 153] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDID_INTSHORTCUTCREATE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_NEWFOLDER: &str = "NewFolder";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_NEWFOLDERA: &str = "NewFolder";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_NEWFOLDERW: &str = "NewFolder";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_VIEWDETAILS: &str = "ViewDetails";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_VIEWDETAILSA: &str = "ViewDetails";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_VIEWDETAILSW: &str = "ViewDetails";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_VIEWLIST: &str = "ViewList";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_VIEWLISTA: &str = "ViewList";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMDSTR_VIEWLISTW: &str = "ViewList";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_ASYNCVERBSTATE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_CANRENAME: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_DEFAULTONLY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_DISABLEDVERBS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_DONOTPICKDEFAULT: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_EXPLORE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_EXTENDEDVERBS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_INCLUDESTATIC: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_ITEMMENU: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_NODEFAULT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_NOVERBS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_OPTIMIZEFORINVOKE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_RESERVED: u32 = 4294901760u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_SYNCCASCADEMENU: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMF_VERBSONLY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMIC_MASK_CONTROL_DOWN: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMIC_MASK_PTINVOKE: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CMIC_MASK_SHIFT_DOWN: u32 = 268435456u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct CMINVOKECOMMANDINFO {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerb: ::windows_sys::core::PCSTR,
    pub lpParameters: ::windows_sys::core::PCSTR,
    pub lpDirectory: ::windows_sys::core::PCSTR,
    pub nShow: i32,
    pub dwHotKey: u32,
    pub hIcon: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for CMINVOKECOMMANDINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for CMINVOKECOMMANDINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct CMINVOKECOMMANDINFOEX {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerb: ::windows_sys::core::PCSTR,
    pub lpParameters: ::windows_sys::core::PCSTR,
    pub lpDirectory: ::windows_sys::core::PCSTR,
    pub nShow: i32,
    pub dwHotKey: u32,
    pub hIcon: super::super::Foundation::HANDLE,
    pub lpTitle: ::windows_sys::core::PCSTR,
    pub lpVerbW: ::windows_sys::core::PCWSTR,
    pub lpParametersW: ::windows_sys::core::PCWSTR,
    pub lpDirectoryW: ::windows_sys::core::PCWSTR,
    pub lpTitleW: ::windows_sys::core::PCWSTR,
    pub ptInvoke: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for CMINVOKECOMMANDINFOEX {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for CMINVOKECOMMANDINFOEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct CMINVOKECOMMANDINFOEX_REMOTE {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerbString: ::windows_sys::core::PCSTR,
    pub lpParameters: ::windows_sys::core::PCSTR,
    pub lpDirectory: ::windows_sys::core::PCSTR,
    pub nShow: i32,
    pub dwHotKey: u32,
    pub lpTitle: ::windows_sys::core::PCSTR,
    pub lpVerbWString: ::windows_sys::core::PCWSTR,
    pub lpParametersW: ::windows_sys::core::PCWSTR,
    pub lpDirectoryW: ::windows_sys::core::PCWSTR,
    pub lpTitleW: ::windows_sys::core::PCWSTR,
    pub ptInvoke: super::super::Foundation::POINT,
    pub lpVerbInt: u32,
    pub lpVerbWInt: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for CMINVOKECOMMANDINFOEX_REMOTE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for CMINVOKECOMMANDINFOEX_REMOTE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CM_COLUMNINFO {
    pub cbSize: u32,
    pub dwMask: u32,
    pub dwState: u32,
    pub uWidth: u32,
    pub uDefaultWidth: u32,
    pub uIdealWidth: u32,
    pub wszName: [u16; 80],
}
impl ::core::marker::Copy for CM_COLUMNINFO {}
impl ::core::clone::Clone for CM_COLUMNINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CM_ENUM_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_ENUM_ALL: CM_ENUM_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_ENUM_VISIBLE: CM_ENUM_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CM_MASK = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_MASK_WIDTH: CM_MASK = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_MASK_DEFAULTWIDTH: CM_MASK = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_MASK_IDEALWIDTH: CM_MASK = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_MASK_NAME: CM_MASK = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_MASK_STATE: CM_MASK = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CM_SET_WIDTH_VALUE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_WIDTH_USEDEFAULT: CM_SET_WIDTH_VALUE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_WIDTH_AUTOSIZE: CM_SET_WIDTH_VALUE = -2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CM_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_STATE_NONE: CM_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_STATE_VISIBLE: CM_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_STATE_FIXEDWIDTH: CM_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_STATE_NOSORTBYFOLDERNESS: CM_STATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CM_STATE_ALWAYSVISIBLE: CM_STATE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMPONENT_DEFAULT_LEFT: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMPONENT_DEFAULT_TOP: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMPONENT_TOP: u32 = 1073741823u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_CHECKED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_CURITEMSTATE: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_DIRTY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_FRIENDLYNAME: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_NOSCROLL: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_ORIGINAL_CSI: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_POS_LEFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_POS_TOP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_POS_ZINDEX: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_RESTORED_CSI: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_SIZE_HEIGHT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_SIZE_WIDTH: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_SOURCE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_SUBSCRIBEDURL: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_ELEM_TYPE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_TYPE_CFHTML: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_TYPE_CONTROL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_TYPE_HTMLDOC: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_TYPE_MAX: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_TYPE_PICTURE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COMP_TYPE_WEBSITE: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CONFIRM_CONFLICT_ITEM {
    pub pShellItem: IShellItem2,
    pub pszOriginalName: ::windows_sys::core::PWSTR,
    pub pszAlternateName: ::windows_sys::core::PWSTR,
    pub pszLocationShort: ::windows_sys::core::PWSTR,
    pub pszLocationFull: ::windows_sys::core::PWSTR,
    pub nType: SYNCMGR_CONFLICT_ITEM_TYPE,
}
impl ::core::marker::Copy for CONFIRM_CONFLICT_ITEM {}
impl ::core::clone::Clone for CONFIRM_CONFLICT_ITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CONFIRM_CONFLICT_RESULT_INFO {
    pub pszNewName: ::windows_sys::core::PWSTR,
    pub iItemIndex: u32,
}
impl ::core::marker::Copy for CONFIRM_CONFLICT_RESULT_INFO {}
impl ::core::clone::Clone for CONFIRM_CONFLICT_RESULT_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CONFLICT_RESOLUTION_CLSID_KEY: &str = "ConflictResolutionCLSID";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ACCESSDENIED_READONLY: ::windows_sys::core::HRESULT = -2144927681i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ACCESS_DENIED_DEST: ::windows_sys::core::HRESULT = -2144927710i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ACCESS_DENIED_SRC: ::windows_sys::core::HRESULT = -2144927711i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ALREADY_EXISTS_FOLDER: ::windows_sys::core::HRESULT = -2144927700i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ALREADY_EXISTS_NORMAL: ::windows_sys::core::HRESULT = -2144927703i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ALREADY_EXISTS_READONLY: ::windows_sys::core::HRESULT = -2144927702i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ALREADY_EXISTS_SYSTEM: ::windows_sys::core::HRESULT = -2144927701i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_BLOCKED_BY_DLP_POLICY: ::windows_sys::core::HRESULT = -2144927666i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_BLOCKED_BY_EDP_FOR_REMOVABLE_DRIVE: ::windows_sys::core::HRESULT = -2144927670i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_BLOCKED_BY_EDP_POLICY: ::windows_sys::core::HRESULT = -2144927672i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_CANCELLED: ::windows_sys::core::HRESULT = -2144927743i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_CANNOT_MOVE_FROM_RECYCLE_BIN: ::windows_sys::core::HRESULT = -2144927677i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_CANNOT_MOVE_SHARED_FOLDER: ::windows_sys::core::HRESULT = -2144927676i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_CANT_REACH_SOURCE: ::windows_sys::core::HRESULT = -2144927691i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_IS_RO_CD: ::windows_sys::core::HRESULT = -2144927729i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_IS_RO_DVD: ::windows_sys::core::HRESULT = -2144927726i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_IS_RW_CD: ::windows_sys::core::HRESULT = -2144927728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_IS_RW_DVD: ::windows_sys::core::HRESULT = -2144927725i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_IS_R_CD: ::windows_sys::core::HRESULT = -2144927727i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_IS_R_DVD: ::windows_sys::core::HRESULT = -2144927724i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_SAME_TREE: ::windows_sys::core::HRESULT = -2144927734i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DEST_SUBTREE: ::windows_sys::core::HRESULT = -2144927735i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DIFF_DIR: ::windows_sys::core::HRESULT = -2144927740i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DIR_NOT_EMPTY: ::windows_sys::core::HRESULT = -2144927683i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DISK_FULL: ::windows_sys::core::HRESULT = -2144927694i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_DISK_FULL_CLEAN: ::windows_sys::core::HRESULT = -2144927693i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_EA_LOSS: ::windows_sys::core::HRESULT = -2144927698i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_EA_NOT_SUPPORTED: ::windows_sys::core::HRESULT = -2144927692i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ENCRYPTION_LOSS: ::windows_sys::core::HRESULT = -2144927695i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_FAT_MAX_IN_ROOT: ::windows_sys::core::HRESULT = -2144927682i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_FILE_IS_FLD_DEST: ::windows_sys::core::HRESULT = -2144927732i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_FILE_TOO_LARGE: ::windows_sys::core::HRESULT = -2144927731i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_FLD_IS_FILE_DEST: ::windows_sys::core::HRESULT = -2144927733i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_INTERNET_ITEM_STORAGE_PROVIDER_ERROR: ::windows_sys::core::HRESULT = -2144927675i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_INTERNET_ITEM_STORAGE_PROVIDER_PAUSED: ::windows_sys::core::HRESULT = -2144927674i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_INTERNET_ITEM_UNAVAILABLE: ::windows_sys::core::HRESULT = -2144927678i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_INVALID_FILES_DEST: ::windows_sys::core::HRESULT = -2144927716i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_INVALID_FILES_SRC: ::windows_sys::core::HRESULT = -2144927717i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_MANY_SRC_1_DEST: ::windows_sys::core::HRESULT = -2144927739i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_NET_DISCONNECT_DEST: ::windows_sys::core::HRESULT = -2144927706i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_NET_DISCONNECT_SRC: ::windows_sys::core::HRESULT = -2144927707i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_NEWFILE_NAME_TOO_LONG: ::windows_sys::core::HRESULT = -2144927685i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_NEWFOLDER_NAME_TOO_LONG: ::windows_sys::core::HRESULT = -2144927684i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_PATH_NOT_FOUND_DEST: ::windows_sys::core::HRESULT = -2144927708i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_PATH_NOT_FOUND_SRC: ::windows_sys::core::HRESULT = -2144927709i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_PATH_TOO_DEEP_DEST: ::windows_sys::core::HRESULT = -2144927714i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_PATH_TOO_DEEP_SRC: ::windows_sys::core::HRESULT = -2144927715i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_PROPERTIES_LOSS: ::windows_sys::core::HRESULT = -2144927696i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_PROPERTY_LOSS: ::windows_sys::core::HRESULT = -2144927697i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RECYCLE_BIN_NOT_FOUND: ::windows_sys::core::HRESULT = -2144927686i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RECYCLE_FORCE_NUKE: ::windows_sys::core::HRESULT = -2144927690i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RECYCLE_PATH_TOO_LONG: ::windows_sys::core::HRESULT = -2144927688i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RECYCLE_SIZE_TOO_BIG: ::windows_sys::core::HRESULT = -2144927689i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RECYCLE_UNKNOWN_ERROR: ::windows_sys::core::HRESULT = -2144927691i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_REDIRECTED_TO_WEBPAGE: ::windows_sys::core::HRESULT = -2144927680i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_REMOVABLE_FULL: ::windows_sys::core::HRESULT = -2144927730i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_REQUIRES_EDP_CONSENT: ::windows_sys::core::HRESULT = -2144927673i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_REQUIRES_EDP_CONSENT_FOR_REMOVABLE_DRIVE: ::windows_sys::core::HRESULT = -2144927671i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_REQUIRES_ELEVATION: ::windows_sys::core::HRESULT = -2144927742i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RMS_BLOCKED_BY_EDP_FOR_REMOVABLE_DRIVE: ::windows_sys::core::HRESULT = -2144927668i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_RMS_REQUIRES_EDP_CONSENT_FOR_REMOVABLE_DRIVE: ::windows_sys::core::HRESULT = -2144927669i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ROOT_DIR_DEST: ::windows_sys::core::HRESULT = -2144927712i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_ROOT_DIR_SRC: ::windows_sys::core::HRESULT = -2144927713i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SAME_FILE: ::windows_sys::core::HRESULT = -2144927741i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SERVER_BAD_FILE_TYPE: ::windows_sys::core::HRESULT = -2144927679i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SHARING_VIOLATION_DEST: ::windows_sys::core::HRESULT = -2144927704i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SHARING_VIOLATION_SRC: ::windows_sys::core::HRESULT = -2144927705i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SILENT_FAIL_BY_DLP_POLICY: ::windows_sys::core::HRESULT = -2144927665i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SRC_IS_RO_CD: ::windows_sys::core::HRESULT = -2144927723i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SRC_IS_RO_DVD: ::windows_sys::core::HRESULT = -2144927720i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SRC_IS_RW_CD: ::windows_sys::core::HRESULT = -2144927722i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SRC_IS_RW_DVD: ::windows_sys::core::HRESULT = -2144927719i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SRC_IS_R_CD: ::windows_sys::core::HRESULT = -2144927721i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_SRC_IS_R_DVD: ::windows_sys::core::HRESULT = -2144927718i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_STREAM_LOSS: ::windows_sys::core::HRESULT = -2144927699i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_USER_CANCELLED: ::windows_sys::core::HRESULT = -2144927744i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_E_WARNED_BY_DLP_POLICY: ::windows_sys::core::HRESULT = -2144927667i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_ALREADY_DONE: ::windows_sys::core::HRESULT = 2555914i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_CLOSE_PROGRAM: ::windows_sys::core::HRESULT = 2555917i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_COLLISIONRESOLVED: ::windows_sys::core::HRESULT = 2555918i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_DONT_PROCESS_CHILDREN: ::windows_sys::core::HRESULT = 2555912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_KEEP_BOTH: ::windows_sys::core::HRESULT = 2555916i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_MERGE: ::windows_sys::core::HRESULT = 2555910i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_NOT_HANDLED: ::windows_sys::core::HRESULT = 2555907i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_PENDING: ::windows_sys::core::HRESULT = 2555915i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_PROGRESS_PAUSE: ::windows_sys::core::HRESULT = 2555919i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_USER_IGNORED: ::windows_sys::core::HRESULT = 2555909i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_USER_RETRY: ::windows_sys::core::HRESULT = 2555908i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const COPYENGINE_S_YES: ::windows_sys::core::HRESULT = 2555905i32;
pub const CPFG_CREDENTIAL_PROVIDER_LABEL: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 678150131, data2: 47828, data3: 17295, data4: [176, 7, 121, 183, 38, 124, 61, 72] };
pub const CPFG_CREDENTIAL_PROVIDER_LOGO: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 763590517, data2: 63181, data3: 17998, data4: [167, 69, 72, 47, 208, 180, 116, 147] };
pub const CPFG_LOGON_PASSWORD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1617054970, data2: 42103, data3: 18353, data4: [138, 142, 58, 74, 25, 152, 24, 39] };
pub const CPFG_LOGON_USERNAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3658857448, data2: 38221, data3: 20435, data4: [176, 244, 31, 181, 185, 11, 23, 75] };
pub const CPFG_SMARTCARD_PIN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1340417595, data2: 37249, data3: 18113, data4: [176, 164, 157, 237, 212, 219, 125, 234] };
pub const CPFG_SMARTCARD_USERNAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1042206569, data2: 22156, data3: 19862, data4: [157, 89, 70, 68, 65, 116, 226, 214] };
pub const CPFG_STANDALONE_SUBMIT_BUTTON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 192613080, data2: 52278, data3: 19801, data4: [128, 43, 130, 247, 20, 250, 112, 34] };
pub const CPFG_STYLE_LINK_AS_BUTTON: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 143631624, data2: 38054, data3: 17456, data4: [164, 203, 111, 198, 227, 192, 185, 226] };
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CPLINFO {
    pub idIcon: i32,
    pub idName: i32,
    pub idInfo: i32,
    pub lData: isize,
}
impl ::core::marker::Copy for CPLINFO {}
impl ::core::clone::Clone for CPLINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPLPAGE_DISPLAY_BACKGROUND: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPLPAGE_KEYBOARD_SPEED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPLPAGE_MOUSE_BUTTONS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPLPAGE_MOUSE_PTRMOTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPLPAGE_MOUSE_WHEEL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_DBLCLK: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_DYNAMIC_RES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_EXIT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_GETCOUNT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_INIT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_INQUIRE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_NEWINQUIRE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_SELECT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_SETUP: u32 = 200u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_STARTWPARMS: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_STARTWPARMSA: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_STARTWPARMSW: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPL_STOP: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CPVIEW = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPVIEW_CLASSIC: CPVIEW = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPVIEW_ALLITEMS: CPVIEW = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPVIEW_CATEGORY: CPVIEW = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPVIEW_HOME: CPVIEW = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_ACCOUNT_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPAO_NONE: CREDENTIAL_PROVIDER_ACCOUNT_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPAO_EMPTY_LOCAL: CREDENTIAL_PROVIDER_ACCOUNT_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPAO_EMPTY_CONNECTED: CREDENTIAL_PROVIDER_ACCOUNT_OPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPCFO_NONE: CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPCFO_ENABLE_PASSWORD_REVEAL: CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPCFO_IS_EMAIL_ADDRESS: CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPCFO_ENABLE_TOUCH_KEYBOARD_AUTO_INVOKE: CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPCFO_NUMBERS_ONLY: CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPCFO_SHOW_ENGLISH_KEYBOARD: CREDENTIAL_PROVIDER_CREDENTIAL_FIELD_OPTIONS = 16i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION {
    pub ulAuthenticationPackage: u32,
    pub clsidCredentialProvider: ::windows_sys::core::GUID,
    pub cbSerialization: u32,
    pub rgbSerialization: *mut u8,
}
impl ::core::marker::Copy for CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION {}
impl ::core::clone::Clone for CREDENTIAL_PROVIDER_CREDENTIAL_SERIALIZATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR {
    pub dwFieldID: u32,
    pub cpft: CREDENTIAL_PROVIDER_FIELD_TYPE,
    pub pszLabel: ::windows_sys::core::PWSTR,
    pub guidFieldType: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR {}
impl ::core::clone::Clone for CREDENTIAL_PROVIDER_FIELD_DESCRIPTOR {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFIS_NONE: CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFIS_READONLY: CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFIS_DISABLED: CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFIS_FOCUSED: CREDENTIAL_PROVIDER_FIELD_INTERACTIVE_STATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_FIELD_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFS_HIDDEN: CREDENTIAL_PROVIDER_FIELD_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFS_DISPLAY_IN_SELECTED_TILE: CREDENTIAL_PROVIDER_FIELD_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFS_DISPLAY_IN_DESELECTED_TILE: CREDENTIAL_PROVIDER_FIELD_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFS_DISPLAY_IN_BOTH: CREDENTIAL_PROVIDER_FIELD_STATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_FIELD_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_INVALID: CREDENTIAL_PROVIDER_FIELD_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_LARGE_TEXT: CREDENTIAL_PROVIDER_FIELD_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_SMALL_TEXT: CREDENTIAL_PROVIDER_FIELD_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_COMMAND_LINK: CREDENTIAL_PROVIDER_FIELD_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_EDIT_TEXT: CREDENTIAL_PROVIDER_FIELD_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_PASSWORD_TEXT: CREDENTIAL_PROVIDER_FIELD_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_TILE_IMAGE: CREDENTIAL_PROVIDER_FIELD_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_CHECKBOX: CREDENTIAL_PROVIDER_FIELD_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_COMBOBOX: CREDENTIAL_PROVIDER_FIELD_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPFT_SUBMIT_BUTTON: CREDENTIAL_PROVIDER_FIELD_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPGSR_NO_CREDENTIAL_NOT_FINISHED: CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPGSR_NO_CREDENTIAL_FINISHED: CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPGSR_RETURN_CREDENTIAL_FINISHED: CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPGSR_RETURN_NO_CREDENTIAL_FINISHED: CREDENTIAL_PROVIDER_GET_SERIALIZATION_RESPONSE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CREDENTIAL_PROVIDER_NO_DEFAULT: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_STATUS_ICON = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPSI_NONE: CREDENTIAL_PROVIDER_STATUS_ICON = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPSI_ERROR: CREDENTIAL_PROVIDER_STATUS_ICON = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPSI_WARNING: CREDENTIAL_PROVIDER_STATUS_ICON = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPSI_SUCCESS: CREDENTIAL_PROVIDER_STATUS_ICON = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CREDENTIAL_PROVIDER_USAGE_SCENARIO = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPUS_INVALID: CREDENTIAL_PROVIDER_USAGE_SCENARIO = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPUS_LOGON: CREDENTIAL_PROVIDER_USAGE_SCENARIO = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPUS_UNLOCK_WORKSTATION: CREDENTIAL_PROVIDER_USAGE_SCENARIO = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPUS_CHANGE_PASSWORD: CREDENTIAL_PROVIDER_USAGE_SCENARIO = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPUS_CREDUI: CREDENTIAL_PROVIDER_USAGE_SCENARIO = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CPUS_PLAP: CREDENTIAL_PROVIDER_USAGE_SCENARIO = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
pub struct CSFV {
    pub cbSize: u32,
    pub pshf: IShellFolder,
    pub psvOuter: IShellView,
    pub pidl: *mut Common::ITEMIDLIST,
    pub lEvents: i32,
    pub pfnCallback: LPFNVIEWCALLBACK,
    pub fvm: FOLDERVIEWMODE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for CSFV {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for CSFV {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_ADMINTOOLS: u32 = 48u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_ALTSTARTUP: u32 = 29u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_APPDATA: u32 = 26u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_BITBUCKET: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_CDBURN_AREA: u32 = 59u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_ADMINTOOLS: u32 = 47u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_ALTSTARTUP: u32 = 30u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_APPDATA: u32 = 35u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_DESKTOPDIRECTORY: u32 = 25u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_DOCUMENTS: u32 = 46u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_FAVORITES: u32 = 31u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_MUSIC: u32 = 53u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_OEM_LINKS: u32 = 58u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_PICTURES: u32 = 54u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_PROGRAMS: u32 = 23u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_STARTMENU: u32 = 22u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_STARTUP: u32 = 24u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_TEMPLATES: u32 = 45u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMMON_VIDEO: u32 = 55u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COMPUTERSNEARME: u32 = 61u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_CONNECTIONS: u32 = 49u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_CONTROLS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_COOKIES: u32 = 33u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_DESKTOP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_DESKTOPDIRECTORY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_DRIVES: u32 = 17u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FAVORITES: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_CREATE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_DONT_UNEXPAND: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_DONT_VERIFY: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_MASK: u32 = 65280u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_NO_ALIAS: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_PER_USER_INIT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FLAG_PFTI_TRACKTARGET: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_FONTS: u32 = 20u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_HISTORY: u32 = 34u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_INTERNET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_INTERNET_CACHE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_LOCAL_APPDATA: u32 = 28u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_MYDOCUMENTS: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_MYMUSIC: u32 = 13u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_MYPICTURES: u32 = 39u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_MYVIDEO: u32 = 14u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_NETHOOD: u32 = 19u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_NETWORK: u32 = 18u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PERSONAL: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PRINTERS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PRINTHOOD: u32 = 27u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PROFILE: u32 = 40u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PROGRAMS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PROGRAM_FILES: u32 = 38u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PROGRAM_FILESX86: u32 = 42u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PROGRAM_FILES_COMMON: u32 = 43u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_PROGRAM_FILES_COMMONX86: u32 = 44u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_RECENT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_RESOURCES: u32 = 56u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_RESOURCES_LOCALIZED: u32 = 57u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_SENDTO: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_STARTMENU: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_STARTUP: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_SYSTEM: u32 = 37u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_SYSTEMX86: u32 = 41u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_TEMPLATES: u32 = 21u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSIDL_WINDOWS: u32 = 36u32;
pub const CScriptErrorList: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4023390976, data2: 5647, data3: 4562, data4: [187, 46, 0, 128, 95, 247, 239, 202] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_COINIT: i32 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_COINIT_MTA: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_COINIT_STA: i32 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_FREELIBANDEXIT: i32 = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_INHERITWOW64: i32 = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_INSIST: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_KEYBOARD_LOCALE: i32 = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_NOADDREFLIB: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_OLEINITIALIZE: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_PROCESS_REF: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_REF_COUNTED: i32 = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_THREAD_REF: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_UNUSED: i32 = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_WAIT_ALLOWCOM: i32 = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CTF_WAIT_NO_REENTRANCY: i32 = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type CommandStateChangeConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSC_UPDATECOMMANDS: CommandStateChangeConstants = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSC_NAVIGATEFORWARD: CommandStateChangeConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CSC_NAVIGATEBACK: CommandStateChangeConstants = 2i32;
pub const ConflictFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 681146540, data2: 41217, data3: 17217, data4: [168, 23, 33, 235, 167, 253, 4, 109] };
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct DATABLOCK_HEADER {
    pub cbSize: u32,
    pub dwSignature: u32,
}
impl ::core::marker::Copy for DATABLOCK_HEADER {}
impl ::core::clone::Clone for DATABLOCK_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DATAOBJ_GET_ITEM_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DOGIF_DEFAULT: DATAOBJ_GET_ITEM_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DOGIF_TRAVERSE_LINK: DATAOBJ_GET_ITEM_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DOGIF_NO_HDROP: DATAOBJ_GET_ITEM_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DOGIF_NO_URL: DATAOBJ_GET_ITEM_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DOGIF_ONLY_IF_ONE: DATAOBJ_GET_ITEM_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBCID_CLSIDOFBAR: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBCID_EMPTY: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBCID_GETBAR: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBCID_ONDRAG: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBCID_RESIZE: i32 = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBCID_UPDATESIZE: i32 = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBC_GS_IDEAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBC_GS_SIZEDOWN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBC_HIDE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBC_SHOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBC_SHOWOBSCURE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIF_VIEWMODE_FLOATING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIF_VIEWMODE_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIF_VIEWMODE_TRANSPARENT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIF_VIEWMODE_VERTICAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_ADDTOFRONT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_ALWAYSGRIPPER: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_BKCOLOR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_BREAK: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_DEBOSSED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_FIXED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_FIXEDBMP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_NOGRIPPER: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_NOMARGINS: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_TOPALIGN: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_UNDELETEABLE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_USECHEVRON: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIMF_VARIABLEHEIGHT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_ACTUAL: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_BKCOLOR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_INTEGRAL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_MAXSIZE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_MINSIZE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_MODEFLAGS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBIM_TITLE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBPC_SELECTFIRST: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DEFAULTSAVEFOLDERTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSFT_DETECT: DEFAULTSAVEFOLDERTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSFT_PRIVATE: DEFAULTSAVEFOLDERTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSFT_PUBLIC: DEFAULTSAVEFOLDERTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DEFAULT_FOLDER_MENU_RESTRICTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_DEFAULT: DEFAULT_FOLDER_MENU_RESTRICTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_NO_STATIC_VERBS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_STATIC_VERBS_ONLY: DEFAULT_FOLDER_MENU_RESTRICTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_NO_RESOURCE_VERBS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_OPTIN_HANDLERS_ONLY: DEFAULT_FOLDER_MENU_RESTRICTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_RESOURCE_AND_FOLDER_VERBS_ONLY: DEFAULT_FOLDER_MENU_RESTRICTIONS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_USE_SPECIFIED_HANDLERS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_USE_SPECIFIED_VERBS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_NO_ASYNC_VERBS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_NO_NATIVECPU_VERBS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFMR_NO_NONWOW_VERBS: DEFAULT_FOLDER_MENU_RESTRICTIONS = 4096i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry", feature = "Win32_UI_Shell_Common"))]
pub struct DEFCONTEXTMENU {
    pub hwnd: super::super::Foundation::HWND,
    pub pcmcb: IContextMenuCB,
    pub pidlFolder: *mut Common::ITEMIDLIST,
    pub psf: IShellFolder,
    pub cidl: u32,
    pub apidl: *mut *mut Common::ITEMIDLIST,
    pub punkAssociationInfo: ::windows_sys::core::IUnknown,
    pub cKeys: u32,
    pub aKeys: *const super::super::System::Registry::HKEY,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for DEFCONTEXTMENU {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for DEFCONTEXTMENU {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DEF_SHARE_ID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DEFSHAREID_USERS: DEF_SHARE_ID = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DEFSHAREID_PUBLIC: DEF_SHARE_ID = 2i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct DELEGATEITEMID {
    pub cbSize: u16,
    pub wOuter: u16,
    pub cbInner: u16,
    pub rgb: [u8; 1],
}
impl ::core::marker::Copy for DELEGATEITEMID {}
impl ::core::clone::Clone for DELEGATEITEMID {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DESKBANDCID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_BANDINFOCHANGED: DESKBANDCID = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_SHOWONLY: DESKBANDCID = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_MAXIMIZEBAND: DESKBANDCID = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_PUSHCHEVRON: DESKBANDCID = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_DELAYINIT: DESKBANDCID = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_FINISHINIT: DESKBANDCID = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_SETWINDOWTHEME: DESKBANDCID = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DBID_PERMITAUTOHIDE: DESKBANDCID = 7i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DESKBANDINFO {
    pub dwMask: u32,
    pub ptMinSize: super::super::Foundation::POINTL,
    pub ptMaxSize: super::super::Foundation::POINTL,
    pub ptIntegral: super::super::Foundation::POINTL,
    pub ptActual: super::super::Foundation::POINTL,
    pub wszTitle: [u16; 256],
    pub dwModeFlags: u32,
    pub crBkgnd: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DESKBANDINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DESKBANDINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DESKTOP_SLIDESHOW_DIRECTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSD_FORWARD: DESKTOP_SLIDESHOW_DIRECTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSD_BACKWARD: DESKTOP_SLIDESHOW_DIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DESKTOP_SLIDESHOW_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSO_SHUFFLEIMAGES: DESKTOP_SLIDESHOW_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DESKTOP_SLIDESHOW_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSS_ENABLED: DESKTOP_SLIDESHOW_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSS_SLIDESHOW: DESKTOP_SLIDESHOW_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSS_DISABLED_BY_REMOTE_SESSION: DESKTOP_SLIDESHOW_STATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DESKTOP_WALLPAPER_POSITION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWPOS_CENTER: DESKTOP_WALLPAPER_POSITION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWPOS_TILE: DESKTOP_WALLPAPER_POSITION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWPOS_STRETCH: DESKTOP_WALLPAPER_POSITION = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWPOS_FIT: DESKTOP_WALLPAPER_POSITION = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWPOS_FILL: DESKTOP_WALLPAPER_POSITION = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWPOS_SPAN: DESKTOP_WALLPAPER_POSITION = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(feature = "Win32_UI_Shell_Common")]
pub struct DETAILSINFO {
    pub pidl: *mut Common::ITEMIDLIST,
    pub fmt: i32,
    pub cxChar: i32,
    pub str: Common::STRRET,
    pub iImage: i32,
}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::marker::Copy for DETAILSINFO {}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::clone::Clone for DETAILSINFO {
    fn clone(&self) -> Self {
        *self
    }
}
pub type DFConstraint = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DFMICS {
    pub cbSize: u32,
    pub fMask: u32,
    pub lParam: super::super::Foundation::LPARAM,
    pub idCmdFirst: u32,
    pub idDefMax: u32,
    pub pici: *mut CMINVOKECOMMANDINFO,
    pub punkSite: ::windows_sys::core::IUnknown,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DFMICS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DFMICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DFM_CMD = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_DELETE: DFM_CMD = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_MOVE: DFM_CMD = -2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_COPY: DFM_CMD = -3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_LINK: DFM_CMD = -4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_PROPERTIES: DFM_CMD = -5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_NEWFOLDER: DFM_CMD = -6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_PASTE: DFM_CMD = -7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_VIEWLIST: DFM_CMD = -8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_VIEWDETAILS: DFM_CMD = -9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_PASTELINK: DFM_CMD = -10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_PASTESPECIAL: DFM_CMD = -11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_MODALPROP: DFM_CMD = -12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_CMD_RENAME: DFM_CMD = -13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DFM_MESSAGE_ID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_MERGECONTEXTMENU: DFM_MESSAGE_ID = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_INVOKECOMMAND: DFM_MESSAGE_ID = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_GETHELPTEXT: DFM_MESSAGE_ID = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_WM_MEASUREITEM: DFM_MESSAGE_ID = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_WM_DRAWITEM: DFM_MESSAGE_ID = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_WM_INITMENUPOPUP: DFM_MESSAGE_ID = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_VALIDATECMD: DFM_MESSAGE_ID = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_MERGECONTEXTMENU_TOP: DFM_MESSAGE_ID = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_GETHELPTEXTW: DFM_MESSAGE_ID = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_INVOKECOMMANDEX: DFM_MESSAGE_ID = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_MAPCOMMANDNAME: DFM_MESSAGE_ID = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_GETDEFSTATICID: DFM_MESSAGE_ID = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_GETVERBW: DFM_MESSAGE_ID = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_GETVERBA: DFM_MESSAGE_ID = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_MERGECONTEXTMENU_BOTTOM: DFM_MESSAGE_ID = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DFM_MODIFYQCMFLAGS: DFM_MESSAGE_ID = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_BEGINDRAG: u32 = 204u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_CHECKSTATECHANGED: u32 = 209u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_COLUMNSCHANGED: u32 = 212u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_CONTENTSCHANGED: u32 = 207u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_CTRLMOUSEWHEEL: u32 = 213u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_DEFAULTVERBINVOKED: u32 = 203u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_ENTERPRESSED: u32 = 200u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_ENTERPRISEIDCHANGED: u32 = 224u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_EXPLORERWINDOWREADY: u32 = 221u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_FILELISTENUMDONE: u32 = 201u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_FILTERINVOKED: u32 = 218u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_FOCUSCHANGED: u32 = 208u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_FOLDERCHANGED: u32 = 217u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_DEFAULTCAT: u32 = 262u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_DIRTY: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_FORCEX86: u32 = 259u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_ONDOMAIN: u32 = 261u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_PUBCAT: u32 = 257u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_SHOWPOSTSETUP: u32 = 260u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_IADCCTL_SORT: u32 = 258u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_ICONSIZECHANGED: u32 = 215u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_INITIALENUMERATIONDONE: u32 = 223u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_NOITEMSTATE_CHANGED: u32 = 206u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_ORDERCHANGED: u32 = 210u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_ABORT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_COMPLETE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_ERROR: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_PROGRESSTEXT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_RESTORE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_START: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SEARCHCOMMAND_UPDATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SELECTEDITEMCHANGED: u32 = 220u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SELECTIONCHANGED: u32 = 200u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_SORTDONE: u32 = 214u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_UPDATEIMAGE: u32 = 222u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_VERBINVOKED: u32 = 202u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_VIEWMODECHANGED: u32 = 205u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_VIEWPAINTDONE: u32 = 211u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DISPID_WORDWHEELEDITED: u32 = 219u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DISPLAY_DEVICE_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DEVICE_PRIMARY: DISPLAY_DEVICE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DEVICE_IMMERSIVE: DISPLAY_DEVICE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DI_GETDRAGIMAGE: &str = "ShellGetDragImage";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLG_SCRNSAVECONFIGURE: u32 = 2003u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DLLGETVERSIONPROC = ::core::option::Option<unsafe extern "system" fn(param0: *mut DLLVERSIONINFO) -> ::windows_sys::core::HRESULT>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct DLLVERSIONINFO {
    pub cbSize: u32,
    pub dwMajorVersion: u32,
    pub dwMinorVersion: u32,
    pub dwBuildNumber: u32,
    pub dwPlatformID: u32,
}
impl ::core::marker::Copy for DLLVERSIONINFO {}
impl ::core::clone::Clone for DLLVERSIONINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct DLLVERSIONINFO2 {
    pub info1: DLLVERSIONINFO,
    pub dwFlags: u32,
    pub ullVersion: u64,
}
impl ::core::marker::Copy for DLLVERSIONINFO2 {}
impl ::core::clone::Clone for DLLVERSIONINFO2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLLVER_BUILD_MASK: u64 = 4294901760u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLLVER_MAJOR_MASK: u64 = 18446462598732840960u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLLVER_MINOR_MASK: u64 = 281470681743360u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLLVER_PLATFORM_NT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLLVER_PLATFORM_WINDOWS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DLLVER_QFE_MASK: u64 = 65535u64;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct DRAGINFOA {
    pub uSize: u32,
    pub pt: super::super::Foundation::POINT,
    pub fNC: super::super::Foundation::BOOL,
    pub lpFileList: ::windows_sys::core::PSTR,
    pub grfKeyState: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DRAGINFOA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DRAGINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct DRAGINFOA {
    pub uSize: u32,
    pub pt: super::super::Foundation::POINT,
    pub fNC: super::super::Foundation::BOOL,
    pub lpFileList: ::windows_sys::core::PSTR,
    pub grfKeyState: u32,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DRAGINFOA {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DRAGINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct DRAGINFOW {
    pub uSize: u32,
    pub pt: super::super::Foundation::POINT,
    pub fNC: super::super::Foundation::BOOL,
    pub lpFileList: ::windows_sys::core::PWSTR,
    pub grfKeyState: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DRAGINFOW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DRAGINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct DRAGINFOW {
    pub uSize: u32,
    pub pt: super::super::Foundation::POINT,
    pub fNC: super::super::Foundation::BOOL,
    pub lpFileList: ::windows_sys::core::PWSTR,
    pub grfKeyState: u32,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DRAGINFOW {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DRAGINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct DROPDESCRIPTION {
    pub r#type: DROPIMAGETYPE,
    pub szMessage: [u16; 260],
    pub szInsert: [u16; 260],
}
impl ::core::marker::Copy for DROPDESCRIPTION {}
impl ::core::clone::Clone for DROPDESCRIPTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DROPFILES {
    pub pFiles: u32,
    pub pt: super::super::Foundation::POINT,
    pub fNC: super::super::Foundation::BOOL,
    pub fWide: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DROPFILES {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DROPFILES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DROPIMAGETYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_INVALID: DROPIMAGETYPE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_NONE: DROPIMAGETYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_COPY: DROPIMAGETYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_MOVE: DROPIMAGETYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_LINK: DROPIMAGETYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_LABEL: DROPIMAGETYPE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_WARNING: DROPIMAGETYPE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DROPIMAGE_NOIMAGE: DROPIMAGETYPE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type DSH_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DSH_ALLOWDROPDESCRIPTIONTEXT: DSH_FLAGS = 1i32;
pub type DShellFolderViewEvents = *mut ::core::ffi::c_void;
pub type DShellNameSpaceEvents = *mut ::core::ffi::c_void;
pub type DShellWindowsEvents = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DVASPECT_COPY: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DVASPECT_LINK: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DVASPECT_SHORTNAME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWFAF_AUTOHIDE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWFAF_GROUP1: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWFAF_GROUP2: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWFAF_HIDDEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWFRF_DELETECONFIGDATA: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const DWFRF_NORMAL: u32 = 0u32;
pub type DWebBrowserEvents = *mut ::core::ffi::c_void;
pub type DWebBrowserEvents2 = *mut ::core::ffi::c_void;
pub const DefFolderMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3325264574, data2: 31027, data3: 18640, data4: [154, 200, 133, 251, 70, 190, 47, 221] };
pub const DesktopGadget: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2454506523, data2: 25954, data3: 19589, data4: [134, 87, 209, 119, 146, 82, 34, 182] };
pub const DesktopWallpaper: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3268358416, data2: 17934, data3: 20417, data4: [185, 208, 138, 28, 12, 156, 196, 189] };
pub const DestinationList: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2012286192, data2: 15797, data3: 18790, data4: [181, 32, 183, 197, 79, 211, 94, 214] };
pub const DocPropShellExtension: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2285073347, data2: 49033, data3: 4561, data4: [190, 53, 8, 0, 54, 177, 26, 3] };
pub const DriveSizeCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2486532947, data2: 51753, data3: 19320, data4: [131, 174, 232, 254, 116, 9, 19, 79] };
pub const DriveTypeCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2963862479, data2: 17203, data3: 19371, data4: [136, 115, 28, 203, 28, 173, 164, 139] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type EC_HOST_UI_MODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECHUIM_DESKTOP: EC_HOST_UI_MODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECHUIM_IMMERSIVE: EC_HOST_UI_MODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECHUIM_SYSTEM_LAUNCHER: EC_HOST_UI_MODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type EDGE_GESTURE_KIND = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EGK_TOUCH: EDGE_GESTURE_KIND = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EGK_KEYBOARD: EDGE_GESTURE_KIND = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EGK_MOUSE: EDGE_GESTURE_KIND = 2i32;
pub const EP_AdvQueryPane: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3035224971, data2: 13498, data3: 19513, data4: [181, 204, 22, 161, 189, 44, 65, 28] };
pub const EP_Commands: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3648280680, data2: 51807, data3: 19062, data4: [145, 205, 245, 161, 41, 251, 176, 118] };
pub const EP_Commands_Organize: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1927812864, data2: 58348, data3: 18016, data4: [191, 36, 60, 59, 123, 100, 136, 6] };
pub const EP_Commands_View: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 569885485, data2: 61098, data3: 17307, data4: [187, 81, 55, 185, 111, 214, 169, 67] };
pub const EP_DetailsPane: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1135344011, data2: 35256, data3: 18221, data4: [185, 206, 230, 155, 130, 41, 240, 25] };
pub const EP_NavPane: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3409013538, data2: 9719, data3: 17080, data4: [138, 9, 84, 13, 35, 164, 60, 47] };
pub const EP_PreviewPane: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2302436305, data2: 17864, data3: 19735, data4: [190, 25, 34, 59, 231, 27, 227, 101] };
pub const EP_QueryPane: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1706876495, data2: 20231, data3: 20263, data4: [131, 167, 26, 252, 164, 223, 125, 221] };
pub const EP_Ribbon: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3530892456, data2: 51698, data3: 18484, data4: [161, 6, 223, 136, 137, 253, 79, 55] };
pub const EP_StatusBar: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1711167182, data2: 23806, data3: 19396, data4: [173, 138, 122, 227, 254, 126, 143, 124] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXECUTE_E_LAUNCH_APPLICATION: ::windows_sys::core::HRESULT = -2144927487i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type EXPLORER_BROWSER_FILL_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBF_NONE: EXPLORER_BROWSER_FILL_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBF_SELECTFROMDATAOBJECT: EXPLORER_BROWSER_FILL_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBF_NODROPTARGET: EXPLORER_BROWSER_FILL_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type EXPLORER_BROWSER_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_NONE: EXPLORER_BROWSER_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_NAVIGATEONCE: EXPLORER_BROWSER_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_SHOWFRAMES: EXPLORER_BROWSER_OPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_ALWAYSNAVIGATE: EXPLORER_BROWSER_OPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_NOTRAVELLOG: EXPLORER_BROWSER_OPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_NOWRAPPERWINDOW: EXPLORER_BROWSER_OPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_HTMLSHAREPOINTVIEW: EXPLORER_BROWSER_OPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_NOBORDER: EXPLORER_BROWSER_OPTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EBO_NOPERSISTVIEWSTATE: EXPLORER_BROWSER_OPTIONS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXP_DARWIN_ID_SIG: u32 = 2684354566u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct EXP_DARWIN_LINK {
    pub dbh: DATABLOCK_HEADER,
    pub szDarwinID: [super::super::Foundation::CHAR; 260],
    pub szwDarwinID: [u16; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for EXP_DARWIN_LINK {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for EXP_DARWIN_LINK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct EXP_PROPERTYSTORAGE {
    pub cbSize: u32,
    pub dwSignature: u32,
    pub abPropertyStorage: [u8; 1],
}
impl ::core::marker::Copy for EXP_PROPERTYSTORAGE {}
impl ::core::clone::Clone for EXP_PROPERTYSTORAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXP_PROPERTYSTORAGE_SIG: u32 = 2684354569u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct EXP_SPECIAL_FOLDER {
    pub cbSize: u32,
    pub dwSignature: u32,
    pub idSpecialFolder: u32,
    pub cbOffset: u32,
}
impl ::core::marker::Copy for EXP_SPECIAL_FOLDER {}
impl ::core::clone::Clone for EXP_SPECIAL_FOLDER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXP_SPECIAL_FOLDER_SIG: u32 = 2684354565u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXP_SZ_ICON_SIG: u32 = 2684354567u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct EXP_SZ_LINK {
    pub cbSize: u32,
    pub dwSignature: u32,
    pub szTarget: [super::super::Foundation::CHAR; 260],
    pub swzTarget: [u16; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for EXP_SZ_LINK {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for EXP_SZ_LINK {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXP_SZ_LINK_SIG: u32 = 2684354561u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct EXTRASEARCH {
    pub guidSearch: ::windows_sys::core::GUID,
    pub wszFriendlyName: [u16; 80],
    pub wszUrl: [u16; 2084],
}
impl ::core::marker::Copy for EXTRASEARCH {}
impl ::core::clone::Clone for EXTRASEARCH {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_ACTIVATIONDENIED_SHELLERROR: ::windows_sys::core::HRESULT = -2144927439i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_ACTIVATIONDENIED_SHELLNOTREADY: ::windows_sys::core::HRESULT = -2144927436i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_ACTIVATIONDENIED_SHELLRESTART: ::windows_sys::core::HRESULT = -2144927438i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_ACTIVATIONDENIED_UNEXPECTED: ::windows_sys::core::HRESULT = -2144927437i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_ACTIVATIONDENIED_USERCLOSE: ::windows_sys::core::HRESULT = -2144927440i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_FILE_PLACEHOLDER_NOT_INITIALIZED: ::windows_sys::core::HRESULT = -2144927472i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_FILE_PLACEHOLDER_SERVER_TIMED_OUT: ::windows_sys::core::HRESULT = -2144927470i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_FILE_PLACEHOLDER_STORAGEPROVIDER_NOT_FOUND: ::windows_sys::core::HRESULT = -2144927469i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_FILE_PLACEHOLDER_VERSION_MISMATCH: ::windows_sys::core::HRESULT = -2144927471i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_FLAGS: ::windows_sys::core::HRESULT = -2147217408i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_IMAGEFEED_CHANGEDISABLED: ::windows_sys::core::HRESULT = -2144926960i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_NOTVALIDFORANIMATEDIMAGE: ::windows_sys::core::HRESULT = -2147221503i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_PREVIEWHANDLER_CORRUPT: ::windows_sys::core::HRESULT = -2042494972i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_PREVIEWHANDLER_DRM_FAIL: ::windows_sys::core::HRESULT = -2042494975i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_PREVIEWHANDLER_NOAUTH: ::windows_sys::core::HRESULT = -2042494974i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_PREVIEWHANDLER_NOTFOUND: ::windows_sys::core::HRESULT = -2042494973i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_SHELL_EXTENSION_BLOCKED: ::windows_sys::core::HRESULT = -2144926975i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_TILE_NOTIFICATIONS_PLATFORM_FAILURE: ::windows_sys::core::HRESULT = -2144927159i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_USERTILE_CHANGEDISABLED: ::windows_sys::core::HRESULT = -2144927215i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_USERTILE_FILESIZE: ::windows_sys::core::HRESULT = -2144927212i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_USERTILE_LARGEORDYNAMIC: ::windows_sys::core::HRESULT = -2144927214i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_USERTILE_UNSUPPORTEDFILETYPE: ::windows_sys::core::HRESULT = -2144927216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const E_USERTILE_VIDEOFRAMESIZE: ::windows_sys::core::HRESULT = -2144927213i32;
pub const EnumerableObjectCollection: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 758409409, data2: 13991, data3: 17334, data4: [172, 36, 211, 240, 47, 217, 96, 122] };
pub const ExecuteFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 299611260, data2: 42277, data3: 16395, data4: [158, 128, 165, 70, 21, 160, 144, 192] };
pub const ExecuteUnknown: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3830354984, data2: 48572, data3: 18823, data4: [160, 153, 64, 220, 143, 210, 85, 231] };
pub const ExplorerBrowser: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1912169349, data2: 56790, data3: 18643, data4: [160, 193, 174, 6, 232, 176, 85, 251] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_BROWSERFIRST: u32 = 40960u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_BROWSERLAST: u32 = 48896u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_GLOBALFIRST: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_GLOBALLAST: u32 = 40959u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_EDIT: u32 = 32832u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_EXPLORE: u32 = 33104u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_FAVORITES: u32 = 33136u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_FILE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_FIND: u32 = 33088u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_HELP: u32 = 33024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_TOOLS: u32 = 32960u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_TOOLS_SEP_GOTO: u32 = 32961u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_VIEW: u32 = 32896u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_MENU_VIEW_SEP_OPTIONS: u32 = 32897u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_SHVIEWFIRST: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_SHVIEWLAST: u32 = 32767u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_STATUS: u32 = 40961u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCIDM_TOOLBAR: u32 = 40960u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_CLSID: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_FLAGS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_ICONFILE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_INFOTIP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_LOGO: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_VIEWID: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCSM_WEBVIEWTEMPLATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCS_FLAG_DRAGDROP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCS_FORCEWRITE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCS_READ: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCT_ADDTOEND: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCT_CONFIGABLE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCT_MERGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCW_INTERNETBAR: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCW_PROGRESS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCW_STATUS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCW_TOOLBAR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FCW_TREE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FDAP = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDAP_BOTTOM: FDAP = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDAP_TOP: FDAP = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FDE_OVERWRITE_RESPONSE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDEOR_DEFAULT: FDE_OVERWRITE_RESPONSE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDEOR_ACCEPT: FDE_OVERWRITE_RESPONSE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDEOR_REFUSE: FDE_OVERWRITE_RESPONSE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FDE_SHAREVIOLATION_RESPONSE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDESVR_DEFAULT: FDE_SHAREVIOLATION_RESPONSE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDESVR_ACCEPT: FDE_SHAREVIOLATION_RESPONSE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDESVR_REFUSE: FDE_SHAREVIOLATION_RESPONSE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_LONGDATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_LONGTIME: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_LTRDATE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_NOAUTOREADINGORDER: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_RELATIVE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_RTLDATE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_SHORTDATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FDTF_SHORTTIME: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FD_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_CLSID: FD_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_SIZEPOINT: FD_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_ATTRIBUTES: FD_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_CREATETIME: FD_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_ACCESSTIME: FD_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_WRITESTIME: FD_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_FILESIZE: FD_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_PROGRESSUI: FD_FLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_LINKUI: FD_FLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FD_UNICODE: FD_FLAGS = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FFFP_MODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FFFP_EXACTMATCH: FFFP_MODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FFFP_NEARESTPARENTMATCH: FFFP_MODE = 1i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FILEDESCRIPTORA {
    pub dwFlags: u32,
    pub clsid: ::windows_sys::core::GUID,
    pub sizel: super::super::Foundation::SIZE,
    pub pointl: super::super::Foundation::POINTL,
    pub dwFileAttributes: u32,
    pub ftCreationTime: super::super::Foundation::FILETIME,
    pub ftLastAccessTime: super::super::Foundation::FILETIME,
    pub ftLastWriteTime: super::super::Foundation::FILETIME,
    pub nFileSizeHigh: u32,
    pub nFileSizeLow: u32,
    pub cFileName: [super::super::Foundation::CHAR; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FILEDESCRIPTORA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FILEDESCRIPTORA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FILEDESCRIPTORW {
    pub dwFlags: u32,
    pub clsid: ::windows_sys::core::GUID,
    pub sizel: super::super::Foundation::SIZE,
    pub pointl: super::super::Foundation::POINTL,
    pub dwFileAttributes: u32,
    pub ftCreationTime: super::super::Foundation::FILETIME,
    pub ftLastAccessTime: super::super::Foundation::FILETIME,
    pub ftLastWriteTime: super::super::Foundation::FILETIME,
    pub nFileSizeHigh: u32,
    pub nFileSizeLow: u32,
    pub cFileName: [u16; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FILEDESCRIPTORW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FILEDESCRIPTORW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FILEGROUPDESCRIPTORA {
    pub cItems: u32,
    pub fgd: [FILEDESCRIPTORA; 1],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FILEGROUPDESCRIPTORA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FILEGROUPDESCRIPTORA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct FILEGROUPDESCRIPTORW {
    pub cItems: u32,
    pub fgd: [FILEDESCRIPTORW; 1],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for FILEGROUPDESCRIPTORW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for FILEGROUPDESCRIPTORW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FILEOPENDIALOGOPTIONS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_OVERWRITEPROMPT: FILEOPENDIALOGOPTIONS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_STRICTFILETYPES: FILEOPENDIALOGOPTIONS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_NOCHANGEDIR: FILEOPENDIALOGOPTIONS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_PICKFOLDERS: FILEOPENDIALOGOPTIONS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_FORCEFILESYSTEM: FILEOPENDIALOGOPTIONS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_ALLNONSTORAGEITEMS: FILEOPENDIALOGOPTIONS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_NOVALIDATE: FILEOPENDIALOGOPTIONS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_ALLOWMULTISELECT: FILEOPENDIALOGOPTIONS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_PATHMUSTEXIST: FILEOPENDIALOGOPTIONS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_FILEMUSTEXIST: FILEOPENDIALOGOPTIONS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_CREATEPROMPT: FILEOPENDIALOGOPTIONS = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_SHAREAWARE: FILEOPENDIALOGOPTIONS = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_NOREADONLYRETURN: FILEOPENDIALOGOPTIONS = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_NOTESTFILECREATE: FILEOPENDIALOGOPTIONS = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_HIDEMRUPLACES: FILEOPENDIALOGOPTIONS = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_HIDEPINNEDPLACES: FILEOPENDIALOGOPTIONS = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_NODEREFERENCELINKS: FILEOPENDIALOGOPTIONS = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_OKBUTTONNEEDSINTERACTION: FILEOPENDIALOGOPTIONS = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_DONTADDTORECENT: FILEOPENDIALOGOPTIONS = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_FORCESHOWHIDDEN: FILEOPENDIALOGOPTIONS = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_DEFAULTNOMINIMODE: FILEOPENDIALOGOPTIONS = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_FORCEPREVIEWPANEON: FILEOPENDIALOGOPTIONS = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOS_SUPPORTSTREAMABLEITEMS: FILEOPENDIALOGOPTIONS = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FILETYPEATTRIBUTEFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_None: FILETYPEATTRIBUTEFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_Exclude: FILETYPEATTRIBUTEFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_Show: FILETYPEATTRIBUTEFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_HasExtension: FILETYPEATTRIBUTEFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEdit: FILETYPEATTRIBUTEFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoRemove: FILETYPEATTRIBUTEFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoNewVerb: FILETYPEATTRIBUTEFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditVerb: FILETYPEATTRIBUTEFLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoRemoveVerb: FILETYPEATTRIBUTEFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditDesc: FILETYPEATTRIBUTEFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditIcon: FILETYPEATTRIBUTEFLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditDflt: FILETYPEATTRIBUTEFLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditVerbCmd: FILETYPEATTRIBUTEFLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditVerbExe: FILETYPEATTRIBUTEFLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoDDE: FILETYPEATTRIBUTEFLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoEditMIME: FILETYPEATTRIBUTEFLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_OpenIsSafe: FILETYPEATTRIBUTEFLAGS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_AlwaysUnsafe: FILETYPEATTRIBUTEFLAGS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_NoRecentDocs: FILETYPEATTRIBUTEFLAGS = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_SafeForElevation: FILETYPEATTRIBUTEFLAGS = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FTA_AlwaysUseDirectInvoke: FILETYPEATTRIBUTEFLAGS = 4194304i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct FILE_ATTRIBUTES_ARRAY {
    pub cItems: u32,
    pub dwSumFileAttributes: u32,
    pub dwProductFileAttributes: u32,
    pub rgdwFileAttributes: [u32; 1],
}
impl ::core::marker::Copy for FILE_ATTRIBUTES_ARRAY {}
impl ::core::clone::Clone for FILE_ATTRIBUTES_ARRAY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FILE_OPERATION_FLAGS2 = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF2_NONE: FILE_OPERATION_FLAGS2 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF2_MERGEFOLDERSONCOLLISION: FILE_OPERATION_FLAGS2 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FILE_USAGE_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FUT_PLAYING: FILE_USAGE_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FUT_EDITING: FILE_USAGE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FUT_GENERIC: FILE_USAGE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FLYOUT_PLACEMENT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FP_DEFAULT: FLYOUT_PLACEMENT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FP_ABOVE: FLYOUT_PLACEMENT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FP_BELOW: FLYOUT_PLACEMENT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FP_LEFT: FLYOUT_PLACEMENT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FP_RIGHT: FLYOUT_PLACEMENT = 4i32;
pub const FMTID_Briefcase: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 848136993, data2: 30505, data3: 19452, data4: [149, 76, 144, 43, 50, 157, 86, 176] };
pub const FMTID_CustomImageProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2127399694, data2: 49462, data3: 19099, data4: [148, 17, 78, 189, 102, 115, 204, 195] };
pub const FMTID_DRM: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2930514404, data2: 35246, data3: 17672, data4: [185, 183, 187, 134, 122, 190, 226, 237] };
pub const FMTID_Displaced: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2601995059, data2: 16639, data3: 4562, data4: [162, 126, 0, 192, 79, 195, 8, 113] };
pub const FMTID_ImageProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 347610529, data2: 309, data3: 19761, data4: [150, 217, 108, 191, 201, 103, 26, 153] };
pub const FMTID_InternetSite: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136353, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const FMTID_Intshcut: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136352, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const FMTID_LibraryProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1568061055, data2: 39741, data3: 17595, data4: [182, 174, 37, 218, 79, 99, 138, 103] };
pub const FMTID_MUSIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1453537070, data2: 52892, data3: 4562, data4: [159, 14, 0, 96, 151, 198, 134, 246] };
pub const FMTID_Misc: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2601995060, data2: 16639, data3: 4562, data4: [162, 126, 0, 192, 79, 195, 8, 113] };
pub const FMTID_Query: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1231625360, data2: 32279, data3: 4122, data4: [169, 28, 8, 0, 43, 46, 205, 169] };
pub const FMTID_ShellDetails: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 677604006, data2: 38205, data3: 4562, data4: [181, 214, 0, 192, 79, 217, 24, 208] };
pub const FMTID_Storage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3072717104, data2: 18415, data3: 4122, data4: [165, 241, 2, 96, 140, 158, 235, 172] };
pub const FMTID_Volume: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2601995061, data2: 16639, data3: 4562, data4: [162, 126, 0, 192, 79, 195, 8, 113] };
pub const FMTID_WebView: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4062663808, data2: 63362, data3: 17041, data4: [189, 148, 241, 54, 147, 81, 58, 236] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_ADDUNDORECORD: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_COPYASDOWNLOAD: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_DONTDISPLAYDESTPATH: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_DONTDISPLAYLOCATIONS: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_DONTDISPLAYSOURCEPATH: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_EARLYFAILURE: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_KEEPNEWERFILE: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_MOVEACLSACROSSVOLUMES: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_NOCOPYHOOKS: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_NOMINIMIZEBOX: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_NOSKIPJUNCTIONS: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_PREFERHARDLINK: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_PRESERVEFILEEXTENSIONS: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_RECYCLEONDELETE: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_REQUIREELEVATION: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOFX_SHOWELEVATIONPROMPT: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_ALLOWUNDO: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_CONFIRMMOUSE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_FILESONLY: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_MULTIDESTFILES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NOCONFIRMATION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NOCONFIRMMKDIR: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NOCOPYSECURITYATTRIBS: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NOERRORUI: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NORECURSEREPARSE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NORECURSION: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_NO_CONNECTED_ELEMENTS: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_RENAMEONCOLLISION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_SILENT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_SIMPLEPROGRESS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_WANTMAPPINGHANDLE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FOF_WANTNUKEWARNING: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FOLDERFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NONE: FOLDERFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_AUTOARRANGE: FOLDERFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_ABBREVIATEDNAMES: FOLDERFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_SNAPTOGRID: FOLDERFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_OWNERDATA: FOLDERFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_BESTFITWINDOW: FOLDERFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_DESKTOP: FOLDERFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_SINGLESEL: FOLDERFLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOSUBFOLDERS: FOLDERFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_TRANSPARENT: FOLDERFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOCLIENTEDGE: FOLDERFLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOSCROLL: FOLDERFLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_ALIGNLEFT: FOLDERFLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOICONS: FOLDERFLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_SHOWSELALWAYS: FOLDERFLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOVISIBLE: FOLDERFLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_SINGLECLICKACTIVATE: FOLDERFLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOWEBVIEW: FOLDERFLAGS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_HIDEFILENAMES: FOLDERFLAGS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_CHECKSELECT: FOLDERFLAGS = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOENUMREFRESH: FOLDERFLAGS = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOGROUPING: FOLDERFLAGS = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_FULLROWSELECT: FOLDERFLAGS = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOFILTERS: FOLDERFLAGS = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOCOLUMNHEADER: FOLDERFLAGS = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOHEADERINALLVIEWS: FOLDERFLAGS = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_EXTENDEDTILES: FOLDERFLAGS = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_TRICHECKSELECT: FOLDERFLAGS = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_AUTOCHECKSELECT: FOLDERFLAGS = 134217728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_NOBROWSERVIEWSTATE: FOLDERFLAGS = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_SUBSETGROUPS: FOLDERFLAGS = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_USESEARCHFOLDER: FOLDERFLAGS = 1073741824i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FWF_ALLOWRTLREADING: FOLDERFLAGS = -2147483648i32;
pub const FOLDERID_AccountPictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 9216177, data2: 21940, data3: 19542, data4: [184, 168, 77, 228, 178, 153, 211, 190] };
pub const FOLDERID_AddNewPrograms: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3730954609, data2: 24252, data3: 20226, data4: [163, 169, 108, 130, 137, 94, 92, 4] };
pub const FOLDERID_AdminTools: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1917776240, data2: 42029, data3: 20463, data4: [159, 38, 182, 14, 132, 111, 186, 79] };
pub const FOLDERID_AllAppMods: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2060875929, data2: 26287, data3: 17338, data4: [145, 86, 106, 173, 66, 230, 197, 150] };
pub const FOLDERID_AppCaptures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3988848241, data2: 39128, data3: 20298, data4: [185, 32, 200, 220, 19, 60, 177, 101] };
pub const FOLDERID_AppDataDesktop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2999313017, data2: 31453, data3: 17311, data4: [178, 140, 196, 31, 225, 187, 246, 114] };
pub const FOLDERID_AppDataDocuments: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2078369296, data2: 8063, data3: 17580, data4: [191, 240, 131, 225, 95, 47, 252, 161] };
pub const FOLDERID_AppDataFavorites: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2096885692, data2: 56863, data3: 17834, data4: [184, 67, 165, 66, 172, 83, 108, 201] };
pub const FOLDERID_AppDataProgramData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1436369059, data2: 41014, data3: 16634, data4: [175, 97, 132, 203, 67, 10, 77, 52] };
pub const FOLDERID_AppUpdates: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2735066777, data2: 62759, data3: 18731, data4: [139, 26, 126, 118, 250, 152, 214, 228] };
pub const FOLDERID_ApplicationShortcuts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2744223617, data2: 58866, data3: 18576, data4: [179, 217, 167, 229, 67, 50, 50, 140] };
pub const FOLDERID_AppsFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 512184461, data2: 35266, data3: 17136, data4: [138, 126, 100, 90, 15, 80, 202, 88] };
pub const FOLDERID_CDBurning: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2656217872, data2: 63501, data3: 18911, data4: [172, 184, 67, 48, 245, 104, 120, 85] };
pub const FOLDERID_CameraRoll: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2875177083, data2: 31970, data3: 20355, data4: [145, 93, 85, 8, 70, 201, 83, 123] };
pub const FOLDERID_CameraRollLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 723574645, data2: 7898, data3: 16441, data4: [128, 151, 56, 121, 130, 39, 213, 183] };
pub const FOLDERID_ChangeRemovePrograms: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3748816556, data2: 37492, data3: 18535, data4: [141, 85, 59, 214, 97, 222, 135, 45] };
pub const FOLDERID_CommonAdminTools: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3493351037, data2: 47811, data3: 18327, data4: [143, 20, 203, 162, 41, 179, 146, 181] };
pub const FOLDERID_CommonOEMLinks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3250250448, data2: 4319, data3: 17204, data4: [190, 221, 122, 162, 11, 34, 122, 157] };
pub const FOLDERID_CommonPrograms: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 20567118, data2: 27390, data3: 18930, data4: [134, 144, 61, 175, 202, 230, 255, 184] };
pub const FOLDERID_CommonStartMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2752599833, data2: 54830, data3: 18717, data4: [170, 124, 231, 75, 139, 227, 176, 103] };
pub const FOLDERID_CommonStartMenuPlaces: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2755692447, data2: 34720, data3: 20349, data4: [183, 0, 2, 7, 185, 102, 25, 74] };
pub const FOLDERID_CommonStartup: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2191911477, data2: 55757, data3: 18373, data4: [150, 41, 225, 93, 47, 113, 78, 110] };
pub const FOLDERID_CommonTemplates: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3108124647, data2: 22444, data3: 17223, data4: [145, 81, 176, 140, 108, 50, 209, 247] };
pub const FOLDERID_ComputerFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 180388732, data2: 48120, data3: 17706, data4: [133, 13, 121, 208, 142, 102, 124, 167] };
pub const FOLDERID_ConflictFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1275001669, data2: 13437, data3: 16390, data4: [165, 190, 172, 12, 176, 86, 113, 146] };
pub const FOLDERID_ConnectionsFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1863113003, data2: 11927, data3: 17873, data4: [136, 255, 176, 209, 134, 184, 222, 221] };
pub const FOLDERID_Contacts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1450723412, data2: 50891, data3: 17963, data4: [129, 105, 136, 227, 80, 172, 184, 130] };
pub const FOLDERID_ControlPanelFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2192001771, data2: 44724, data3: 18012, data4: [160, 20, 208, 151, 238, 52, 109, 99] };
pub const FOLDERID_Cookies: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 722433629, data2: 49385, data3: 16753, data4: [144, 142, 8, 166, 17, 184, 79, 246] };
pub const FOLDERID_CurrentAppMods: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1035209504, data2: 10800, data3: 19902, data4: [145, 126, 119, 29, 210, 29, 208, 153] };
pub const FOLDERID_Desktop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3032468538, data2: 56108, data3: 16972, data4: [176, 41, 127, 233, 154, 135, 198, 65] };
pub const FOLDERID_DevelopmentFiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3689472142, data2: 12371, data3: 19388, data4: [177, 131, 42, 123, 43, 25, 30, 89] };
pub const FOLDERID_Device: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 472564188, data2: 17240, data3: 19308, data4: [151, 51, 175, 33, 21, 101, 118, 240] };
pub const FOLDERID_DeviceMetadataStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1558488553, data2: 58603, data3: 18333, data4: [184, 159, 19, 12, 2, 136, 97, 85] };
pub const FOLDERID_Documents: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4258503376, data2: 9103, data3: 18095, data4: [173, 180, 108, 133, 72, 3, 105, 199] };
pub const FOLDERID_DocumentsLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2064494973, data2: 40146, data3: 19091, data4: [151, 51, 70, 204, 137, 2, 46, 124] };
pub const FOLDERID_Downloads: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 927851152, data2: 4671, data3: 17765, data4: [145, 100, 57, 196, 146, 94, 70, 123] };
pub const FOLDERID_Favorites: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 393738081, data2: 26797, data3: 19850, data4: [135, 189, 48, 183, 89, 250, 51, 221] };
pub const FOLDERID_Fonts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4246899895, data2: 44561, data3: 19171, data4: [134, 76, 22, 243, 145, 10, 184, 254] };
pub const FOLDERID_GameTasks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 89108065, data2: 19928, data3: 18311, data4: [128, 182, 9, 2, 32, 196, 183, 0] };
pub const FOLDERID_Games: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3401919514, data2: 46397, data3: 20188, data4: [146, 215, 107, 46, 138, 193, 148, 52] };
pub const FOLDERID_History: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3655109179, data2: 46980, data3: 17198, data4: [167, 129, 90, 17, 48, 167, 89, 99] };
pub const FOLDERID_HomeGroup: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1381141099, data2: 47587, data3: 19165, data4: [182, 13, 88, 140, 45, 186, 132, 45] };
pub const FOLDERID_HomeGroupCurrentUser: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2608117411, data2: 3581, data3: 20241, data4: [158, 120, 95, 120, 0, 242, 231, 114] };
pub const FOLDERID_ImplicitAppShortcuts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3165988207, data2: 31222, data3: 19694, data4: [183, 37, 220, 52, 228, 2, 253, 70] };
pub const FOLDERID_InternetCache: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 891585000, data2: 13246, data3: 16977, data4: [186, 133, 96, 7, 202, 237, 207, 157] };
pub const FOLDERID_InternetFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1302296692, data2: 19980, data3: 18692, data4: [150, 123, 64, 176, 210, 12, 62, 75] };
pub const FOLDERID_Libraries: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 457090524, data2: 46471, data3: 18310, data4: [180, 239, 189, 29, 195, 50, 174, 174] };
pub const FOLDERID_Links: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3216627168, data2: 50857, data3: 16460, data4: [178, 178, 174, 109, 182, 175, 73, 104] };
pub const FOLDERID_LocalAppData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4055050117, data2: 28602, data3: 20431, data4: [157, 85, 123, 142, 127, 21, 112, 145] };
pub const FOLDERID_LocalAppDataLow: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2770379172, data2: 6016, data3: 20470, data4: [189, 24, 22, 115, 67, 197, 175, 22] };
pub const FOLDERID_LocalDocuments: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4096713427, data2: 37023, data3: 18695, data4: [136, 113, 76, 34, 252, 11, 247, 86] };
pub const FOLDERID_LocalDownloads: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2105798299, data2: 8772, data3: 20080, data4: [177, 245, 83, 147, 4, 42, 241, 228] };
pub const FOLDERID_LocalMusic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2697370265, data2: 8648, data3: 18033, data4: [135, 3, 121, 52, 22, 47, 207, 29] };
pub const FOLDERID_LocalPictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 232587613, data2: 45164, data3: 17877, data4: [140, 76, 245, 151, 19, 133, 70, 57] };
pub const FOLDERID_LocalStorage: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3018524883, data2: 41459, data3: 18795, data4: [134, 90, 66, 181, 54, 205, 160, 236] };
pub const FOLDERID_LocalVideos: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 891841128, data2: 15447, data3: 16801, data4: [187, 177, 14, 174, 115, 215, 108, 149] };
pub const FOLDERID_LocalizedResourcesDir: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 704657246, data2: 8780, data3: 18910, data4: [184, 209, 68, 13, 247, 239, 61, 220] };
pub const FOLDERID_Music: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1272501617, data2: 27929, data3: 18643, data4: [190, 151, 66, 34, 32, 8, 14, 67] };
pub const FOLDERID_MusicLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 554871562, data2: 51306, data3: 20478, data4: [163, 104, 13, 233, 110, 71, 1, 46] };
pub const FOLDERID_NetHood: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3316367187, data2: 57727, data3: 16673, data4: [137, 0, 134, 98, 111, 194, 201, 115] };
pub const FOLDERID_NetworkFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3523997380, data2: 23720, data3: 18693, data4: [174, 59, 191, 37, 30, 160, 155, 83] };
pub const FOLDERID_Objects3D: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 834723109, data2: 37945, data3: 20242, data4: [191, 65, 127, 244, 237, 163, 135, 34] };
pub const FOLDERID_OneDrive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2771106374, data2: 59873, data3: 17247, data4: [179, 217, 40, 218, 166, 72, 192, 246] };
pub const FOLDERID_OriginalImages: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 741785770, data2: 22546, data3: 19335, data4: [191, 208, 76, 208, 223, 177, 155, 57] };
pub const FOLDERID_PhotoAlbums: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1775423376, data2: 64563, data3: 20407, data4: [154, 12, 235, 176, 240, 252, 180, 60] };
pub const FOLDERID_Pictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 870482224, data2: 19998, data3: 18038, data4: [131, 90, 152, 57, 92, 59, 195, 187] };
pub const FOLDERID_PicturesLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2844831391, data2: 41019, data3: 20096, data4: [148, 188, 153, 18, 215, 80, 65, 4] };
pub const FOLDERID_Playlists: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3734159815, data2: 33663, data3: 20329, data4: [163, 187, 134, 230, 49, 32, 74, 35] };
pub const FOLDERID_PrintHood: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2457124237, data2: 53201, data3: 16835, data4: [179, 94, 177, 63, 85, 167, 88, 244] };
pub const FOLDERID_PrintersFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1996246573, data2: 54957, data3: 17689, data4: [166, 99, 55, 189, 86, 6, 129, 133] };
pub const FOLDERID_Profile: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1584170383, data2: 3618, data3: 18272, data4: [154, 254, 234, 51, 23, 182, 113, 115] };
pub const FOLDERID_ProgramData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1655397762, data2: 64961, data3: 19907, data4: [169, 221, 7, 13, 29, 73, 93, 151] };
pub const FOLDERID_ProgramFiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2422105014, data2: 49599, data3: 18766, data4: [178, 156, 101, 183, 50, 211, 210, 26] };
pub const FOLDERID_ProgramFilesCommon: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4159827205, data2: 40813, data3: 18338, data4: [170, 174, 41, 211, 23, 198, 240, 102] };
pub const FOLDERID_ProgramFilesCommonX64: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1667618215, data2: 3853, data3: 17893, data4: [135, 246, 13, 165, 107, 106, 79, 125] };
pub const FOLDERID_ProgramFilesCommonX86: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3734457636, data2: 55750, data3: 19774, data4: [191, 145, 244, 69, 81, 32, 185, 23] };
pub const FOLDERID_ProgramFilesX64: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1837142903, data2: 27376, data3: 17483, data4: [137, 87, 163, 119, 63, 2, 32, 14] };
pub const FOLDERID_ProgramFilesX86: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2086289647, data2: 41211, data3: 19452, data4: [135, 74, 192, 242, 224, 185, 250, 142] };
pub const FOLDERID_Programs: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2810142071, data2: 11819, data3: 17603, data4: [166, 162, 171, 166, 1, 5, 74, 81] };
pub const FOLDERID_Public: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3755964066, data2: 51242, data3: 19811, data4: [144, 106, 86, 68, 172, 69, 115, 133] };
pub const FOLDERID_PublicDesktop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3299488781, data2: 61967, data3: 18531, data4: [175, 239, 248, 126, 242, 230, 186, 37] };
pub const FOLDERID_PublicDocuments: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3980928175, data2: 56548, data3: 17832, data4: [129, 226, 252, 121, 101, 8, 54, 52] };
pub const FOLDERID_PublicDownloads: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1029983387, data2: 8120, data3: 20272, data4: [155, 69, 246, 112, 35, 95, 121, 192] };
pub const FOLDERID_PublicGameTasks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3737068854, data2: 57768, data3: 19545, data4: [182, 162, 65, 69, 134, 71, 106, 234] };
pub const FOLDERID_PublicLibraries: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1222309899, data2: 59087, data3: 20302, data4: [184, 0, 14, 105, 216, 78, 227, 132] };
pub const FOLDERID_PublicMusic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 840235701, data2: 38743, data3: 17048, data4: [187, 97, 146, 169, 222, 170, 68, 255] };
pub const FOLDERID_PublicPictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3068918662, data2: 26887, data3: 16700, data4: [154, 247, 79, 194, 171, 240, 124, 197] };
pub const FOLDERID_PublicRingtones: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3847596896, data2: 5435, data3: 19735, data4: [159, 4, 165, 254, 153, 252, 21, 236] };
pub const FOLDERID_PublicUserTiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 75673452, data2: 2289, data3: 19508, data4: [140, 144, 225, 126, 201, 139, 30, 23] };
pub const FOLDERID_PublicVideos: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 603985978, data2: 24965, data3: 18939, data4: [162, 216, 74, 57, 42, 96, 43, 163] };
pub const FOLDERID_QuickLaunch: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1386541089, data2: 31605, data3: 18601, data4: [159, 107, 75, 135, 162, 16, 188, 143] };
pub const FOLDERID_Recent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2924527745, data2: 60370, data3: 17290, data4: [134, 85, 138, 9, 46, 52, 152, 122] };
pub const FOLDERID_RecordedCalls: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 797655234, data2: 33773, data3: 18670, data4: [179, 131, 161, 241, 87, 236, 111, 154] };
pub const FOLDERID_RecordedTVLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 443538338, data2: 62509, data3: 17240, data4: [167, 152, 183, 77, 116, 89, 38, 197] };
pub const FOLDERID_RecycleBinFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3075686470, data2: 16075, data3: 19480, data4: [190, 78, 100, 205, 76, 183, 214, 172] };
pub const FOLDERID_ResourceDir: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2328955953, data2: 10971, data3: 17046, data4: [168, 247, 228, 112, 18, 50, 201, 114] };
pub const FOLDERID_RetailDemo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 315934366, data2: 9389, data3: 18723, data4: [190, 25, 49, 50, 28, 67, 167, 103] };
pub const FOLDERID_Ringtones: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3362784331, data2: 62622, data3: 16678, data4: [169, 195, 181, 42, 31, 244, 17, 232] };
pub const FOLDERID_RoamedTileImages: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2863191461, data2: 61910, data3: 16985, data4: [186, 168, 120, 231, 239, 96, 131, 94] };
pub const FOLDERID_RoamingAppData: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1052149211, data2: 26105, data3: 19702, data4: [160, 58, 227, 239, 101, 114, 159, 61] };
pub const FOLDERID_RoamingTiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 12385370, data2: 60820, data3: 20040, data4: [150, 161, 63, 98, 23, 242, 25, 144] };
pub const FOLDERID_SEARCH_CSC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3996312646, data2: 12746, data3: 19130, data4: [129, 79, 165, 235, 210, 253, 109, 94] };
pub const FOLDERID_SEARCH_MAPI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2565606936, data2: 8344, data3: 19780, data4: [134, 68, 102, 151, 147, 21, 162, 129] };
pub const FOLDERID_SampleMusic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2991638120, data2: 62845, data3: 20193, data4: [166, 60, 41, 14, 231, 209, 170, 31] };
pub const FOLDERID_SamplePictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3297772864, data2: 9081, data3: 19573, data4: [132, 75, 100, 230, 250, 248, 113, 107] };
pub const FOLDERID_SamplePlaylists: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 365586867, data2: 12526, data3: 18881, data4: [172, 225, 107, 94, 195, 114, 175, 181] };
pub const FOLDERID_SampleVideos: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2241768852, data2: 11909, data3: 18605, data4: [167, 26, 9, 105, 203, 86, 166, 205] };
pub const FOLDERID_SavedGames: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1281110783, data2: 48029, data3: 17328, data4: [181, 180, 45, 114, 229, 78, 170, 164] };
pub const FOLDERID_SavedPictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 991508610, data2: 54189, data3: 20139, data4: [150, 90, 105, 130, 157, 31, 181, 159] };
pub const FOLDERID_SavedPicturesLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3797637138, data2: 48776, data3: 19417, data4: [148, 176, 41, 35, 52, 119, 182, 195] };
pub const FOLDERID_SavedSearches: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2099067396, data2: 57019, data3: 16661, data4: [149, 207, 47, 41, 218, 41, 32, 218] };
pub const FOLDERID_Screenshots: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3082739329, data2: 57236, data3: 18050, data4: [167, 216, 87, 165, 38, 32, 184, 111] };
pub const FOLDERID_SearchHistory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 223100342, data2: 931, data3: 17967, data4: [160, 230, 8, 146, 76, 65, 181, 212] };
pub const FOLDERID_SearchHome: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 419641297, data2: 47306, data3: 16673, data4: [166, 57, 109, 71, 45, 22, 151, 42] };
pub const FOLDERID_SearchTemplates: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2120444926, data2: 57257, data3: 19806, data4: [180, 86, 215, 179, 152, 81, 216, 169] };
pub const FOLDERID_SendTo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2307064684, data2: 10176, data3: 16459, data4: [143, 8, 16, 45, 16, 220, 253, 116] };
pub const FOLDERID_SidebarDefaultParts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2067361364, data2: 40645, data3: 17152, data4: [190, 10, 36, 130, 235, 174, 26, 38] };
pub const FOLDERID_SidebarParts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2807903790, data2: 20732, data3: 20407, data4: [172, 44, 168, 190, 170, 49, 68, 147] };
pub const FOLDERID_SkyDrive: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2771106374, data2: 59873, data3: 17247, data4: [179, 217, 40, 218, 166, 72, 192, 246] };
pub const FOLDERID_SkyDriveCameraRoll: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1987995665, data2: 18891, data3: 17011, data4: [135, 194, 32, 243, 85, 225, 8, 91] };
pub const FOLDERID_SkyDriveDocuments: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 618176036, data2: 12057, data3: 17716, data4: [157, 222, 106, 102, 113, 251, 184, 254] };
pub const FOLDERID_SkyDriveMusic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3287434654, data2: 32982, data3: 17884, data4: [191, 239, 31, 118, 159, 43, 231, 48] };
pub const FOLDERID_SkyDrivePictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 865540533, data2: 35911, data3: 18580, data4: [148, 194, 216, 247, 122, 221, 68, 166] };
pub const FOLDERID_StartMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1650152387, data2: 43848, data3: 20161, data4: [186, 31, 161, 239, 65, 70, 252, 25] };
pub const FOLDERID_StartMenuAllPrograms: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4066575855, data2: 26952, data3: 16569, data4: [178, 85, 129, 69, 61, 9, 199, 133] };
pub const FOLDERID_Startup: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3111985339, data2: 62570, data3: 19607, data4: [186, 16, 94, 54, 8, 67, 8, 84] };
pub const FOLDERID_SyncManagerFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1130793976, data2: 49486, data3: 18866, data4: [151, 201, 116, 119, 132, 215, 132, 183] };
pub const FOLDERID_SyncResultsFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 681220675, data2: 48708, data3: 16471, data4: [164, 27, 88, 122, 118, 215, 231, 249] };
pub const FOLDERID_SyncSetupFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 253837624, data2: 45523, data3: 19088, data4: [187, 169, 39, 203, 192, 197, 56, 154] };
pub const FOLDERID_System: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 448876151, data2: 743, data3: 20061, data4: [183, 68, 46, 177, 174, 81, 152, 183] };
pub const FOLDERID_SystemX86: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3595710896, data2: 45809, data3: 18519, data4: [164, 206, 168, 231, 198, 234, 125, 39] };
pub const FOLDERID_Templates: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2788332520, data2: 26190, data3: 18651, data4: [160, 121, 223, 117, 158, 5, 9, 247] };
pub const FOLDERID_UserPinned: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2654573995, data2: 8092, data3: 20243, data4: [184, 39, 72, 178, 75, 108, 113, 116] };
pub const FOLDERID_UserProfiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 123916914, data2: 50442, data3: 19376, data4: [163, 130, 105, 125, 205, 114, 155, 128] };
pub const FOLDERID_UserProgramFiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1557638882, data2: 8729, data3: 19047, data4: [184, 93, 108, 156, 225, 86, 96, 203] };
pub const FOLDERID_UserProgramFilesCommon: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3166515287, data2: 51804, data3: 17954, data4: [180, 45, 188, 86, 219, 10, 229, 22] };
pub const FOLDERID_UsersFiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4090367868, data2: 18689, data3: 19148, data4: [134, 72, 213, 212, 75, 4, 239, 143] };
pub const FOLDERID_UsersLibraries: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2734838877, data2: 57087, data3: 17995, data4: [171, 232, 97, 200, 100, 141, 147, 155] };
pub const FOLDERID_Videos: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 412654365, data2: 39349, data3: 17755, data4: [132, 28, 171, 124, 116, 228, 221, 252] };
pub const FOLDERID_VideosLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1226740271, data2: 22083, data3: 19188, data4: [167, 235, 78, 122, 19, 141, 129, 116] };
pub const FOLDERID_Windows: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4086035460, data2: 7491, data3: 17138, data4: [147, 5, 103, 222, 11, 40, 252, 35] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FOLDERLOGICALVIEWMODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_UNSPECIFIED: FOLDERLOGICALVIEWMODE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_FIRST: FOLDERLOGICALVIEWMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_DETAILS: FOLDERLOGICALVIEWMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_TILES: FOLDERLOGICALVIEWMODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_ICONS: FOLDERLOGICALVIEWMODE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_LIST: FOLDERLOGICALVIEWMODE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_CONTENT: FOLDERLOGICALVIEWMODE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FLVM_LAST: FOLDERLOGICALVIEWMODE = 5i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct FOLDERSETDATA {
    pub _fs: FOLDERSETTINGS,
    pub _vidRestore: ::windows_sys::core::GUID,
    pub _dwViewPriority: u32,
}
impl ::core::marker::Copy for FOLDERSETDATA {}
impl ::core::clone::Clone for FOLDERSETDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct FOLDERSETTINGS {
    pub ViewMode: u32,
    pub fFlags: u32,
}
impl ::core::marker::Copy for FOLDERSETTINGS {}
impl ::core::clone::Clone for FOLDERSETTINGS {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FOLDERTYPEID_AccountPictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3676986767, data2: 1766, data3: 16391, data4: [171, 166, 175, 135, 125, 82, 110, 166] };
pub const FOLDERTYPEID_Communications: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2437373925, data2: 22635, data3: 20154, data4: [141, 117, 209, 116, 52, 184, 205, 246] };
pub const FOLDERTYPEID_CompressedFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2149662338, data2: 48381, data3: 19535, data4: [136, 23, 187, 39, 96, 18, 103, 169] };
pub const FOLDERTYPEID_Contacts: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3727388908, data2: 39927, data3: 19091, data4: [189, 61, 36, 63, 120, 129, 212, 146] };
pub const FOLDERTYPEID_ControlPanelCategory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3729720928, data2: 64016, data3: 19343, data4: [164, 148, 6, 139, 32, 178, 35, 7] };
pub const FOLDERTYPEID_ControlPanelClassic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 204969203, data2: 46405, data3: 17322, data4: [163, 41, 195, 116, 48, 197, 141, 42] };
pub const FOLDERTYPEID_Documents: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2101991206, data2: 15393, data3: 20229, data4: [153, 170, 253, 194, 201, 71, 70, 86] };
pub const FOLDERTYPEID_Downloads: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2287605870, data2: 42048, data3: 19162, data4: [129, 43, 219, 135, 27, 148, 34, 89] };
pub const FOLDERTYPEID_Games: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3062477008, data2: 30419, data3: 19643, data4: [135, 247, 88, 93, 14, 12, 224, 112] };
pub const FOLDERTYPEID_Generic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1548691637, data2: 63593, data3: 20100, data4: [142, 96, 241, 29, 185, 124, 92, 199] };
pub const FOLDERTYPEID_GenericLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1598991258, data2: 26675, data3: 20321, data4: [137, 157, 49, 207, 70, 151, 157, 73] };
pub const FOLDERTYPEID_GenericSearchResults: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2145262110, data2: 35633, data3: 18853, data4: [147, 184, 107, 225, 76, 250, 73, 67] };
pub const FOLDERTYPEID_Invalid: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1468037272, data2: 35919, data3: 17506, data4: [187, 99, 113, 4, 35, 128, 177, 9] };
pub const FOLDERTYPEID_Music: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2497109452, data2: 19048, data3: 16757, data4: [163, 116, 189, 88, 74, 81, 11, 120] };
pub const FOLDERTYPEID_NetworkExplorer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 634135595, data2: 39548, data3: 20305, data4: [128, 224, 122, 41, 40, 254, 190, 66] };
pub const FOLDERTYPEID_OpenSearch: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2410649129, data2: 6528, data3: 18175, data4: [128, 35, 157, 206, 171, 156, 62, 227] };
pub const FOLDERTYPEID_OtherUsers: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3006790912, data2: 40405, data3: 17973, data4: [166, 212, 218, 51, 253, 16, 43, 122] };
pub const FOLDERTYPEID_Pictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3010006616, data2: 59745, data3: 16955, data4: [182, 135, 56, 110, 191, 216, 50, 57] };
pub const FOLDERTYPEID_Printers: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 746307270, data2: 51268, data3: 18954, data4: [145, 250, 206, 246, 245, 156, 253, 161] };
pub const FOLDERTYPEID_PublishedItems: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2133810070, data2: 65396, data3: 16858, data4: [175, 216, 28, 120, 165, 243, 174, 162] };
pub const FOLDERTYPEID_RecordedTV: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1431806607, data2: 23974, data3: 20355, data4: [136, 9, 194, 201, 138, 17, 166, 250] };
pub const FOLDERTYPEID_RecycleBin: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3604602884, data2: 52615, data3: 17451, data4: [157, 87, 94, 10, 235, 79, 111, 114] };
pub const FOLDERTYPEID_SavedGames: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3493212935, data2: 10443, data3: 16646, data4: [159, 35, 41, 86, 227, 229, 224, 231] };
pub const FOLDERTYPEID_SearchConnector: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2552702446, data2: 28487, data3: 18334, data4: [180, 71, 129, 43, 250, 125, 46, 143] };
pub const FOLDERTYPEID_SearchHome: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2202896964, data2: 2420, data3: 20182, data4: [134, 110, 242, 3, 216, 11, 56, 16] };
pub const FOLDERTYPEID_Searches: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 185311971, data2: 16479, data3: 16734, data4: [166, 238, 202, 214, 37, 32, 120, 83] };
pub const FOLDERTYPEID_SoftwareExplorer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3597941019, data2: 21209, data3: 19975, data4: [131, 78, 103, 201, 134, 16, 243, 157] };
pub const FOLDERTYPEID_StartMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4018648267, data2: 62158, data3: 18309, data4: [134, 88, 76, 166, 198, 62, 56, 198] };
pub const FOLDERTYPEID_StorageProviderDocuments: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3714170214, data2: 28904, data3: 18653, data4: [150, 85, 101, 197, 225, 170, 194, 209] };
pub const FOLDERTYPEID_StorageProviderGeneric: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1325525957, data2: 9093, data3: 16882, data4: [162, 142, 44, 92, 145, 251, 86, 224] };
pub const FOLDERTYPEID_StorageProviderMusic: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1731120510, data2: 44804, data3: 17305, data4: [135, 92, 2, 144, 132, 91, 98, 71] };
pub const FOLDERTYPEID_StorageProviderPictures: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1909867177, data2: 62129, data3: 17101, data4: [173, 146, 235, 147, 0, 199, 204, 10] };
pub const FOLDERTYPEID_StorageProviderVideos: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1361661345, data2: 55217, data3: 18523, data4: [158, 154, 23, 207, 254, 51, 225, 135] };
pub const FOLDERTYPEID_UserFiles: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3440363163, data2: 29154, data3: 18149, data4: [150, 144, 91, 205, 159, 87, 170, 179] };
pub const FOLDERTYPEID_UsersLibraries: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3302592265, data2: 24868, data3: 20448, data4: [153, 66, 130, 100, 22, 8, 45, 169] };
pub const FOLDERTYPEID_Videos: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1604936711, data2: 32375, data3: 18492, data4: [172, 147, 105, 29, 5, 133, 13, 232] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FOLDERVIEWMODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_AUTO: FOLDERVIEWMODE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_FIRST: FOLDERVIEWMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_ICON: FOLDERVIEWMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_SMALLICON: FOLDERVIEWMODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_LIST: FOLDERVIEWMODE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_DETAILS: FOLDERVIEWMODE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_THUMBNAIL: FOLDERVIEWMODE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_TILE: FOLDERVIEWMODE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_THUMBSTRIP: FOLDERVIEWMODE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_CONTENT: FOLDERVIEWMODE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVM_LAST: FOLDERVIEWMODE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FOLDERVIEWOPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_DEFAULT: FOLDERVIEWOPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_VISTALAYOUT: FOLDERVIEWOPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_CUSTOMPOSITION: FOLDERVIEWOPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_CUSTOMORDERING: FOLDERVIEWOPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_SUPPORTHYPERLINKS: FOLDERVIEWOPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_NOANIMATIONS: FOLDERVIEWOPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVO_NOSCROLLTIPS: FOLDERVIEWOPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FOLDER_ENUM_MODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FEM_VIEWRESULT: FOLDER_ENUM_MODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FEM_NAVIGATION: FOLDER_ENUM_MODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FO_COPY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FO_DELETE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FO_MOVE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FO_RENAME: u32 = 4u32;
pub const FSCopyHandler: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3516348426, data2: 2681, data3: 19912, data4: [160, 51, 237, 136, 44, 47, 161, 75] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVSIF_CANVIEWIT: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVSIF_NEWFAILED: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVSIF_NEWFILE: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVSIF_PINNED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVSIF_RECT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type FVTEXTTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const FVST_EMPTYTEXT: FVTEXTTYPE = 0i32;
pub const FileOpenDialog: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3692845724, data2: 59530, data3: 19934, data4: [165, 161, 96, 248, 42, 32, 174, 247] };
pub const FileOperation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 986731893, data2: 34903, data3: 18512, data4: [146, 119, 17, 184, 91, 219, 142, 9] };
pub const FileSaveDialog: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3233080051, data2: 47649, data3: 18291, data4: [141, 186, 51, 94, 201, 70, 235, 139] };
pub const FileSearchBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3303944691, data2: 18280, data3: 4562, data4: [190, 92, 0, 160, 201, 168, 61, 161] };
pub type Folder = *mut ::core::ffi::c_void;
pub type Folder2 = *mut ::core::ffi::c_void;
pub type Folder3 = *mut ::core::ffi::c_void;
pub type FolderItem = *mut ::core::ffi::c_void;
pub type FolderItem2 = *mut ::core::ffi::c_void;
pub type FolderItemVerb = *mut ::core::ffi::c_void;
pub type FolderItemVerbs = *mut ::core::ffi::c_void;
pub type FolderItems = *mut ::core::ffi::c_void;
pub type FolderItems2 = *mut ::core::ffi::c_void;
pub type FolderItems3 = *mut ::core::ffi::c_void;
pub const FolderViewHost: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 548522787, data2: 26984, data3: 20153, data4: [183, 212, 166, 109, 0, 208, 124, 238] };
pub const FrameworkInputPane: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3574729379, data2: 18106, data3: 17605, data4: [130, 45, 202, 128, 146, 193, 252, 114] };
pub const FreeSpaceCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3042998163, data2: 9388, data3: 17607, data4: [130, 226, 131, 23, 38, 170, 108, 183] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GADOF_DIRTY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_HELPTEXT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_HELPTEXTA: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_HELPTEXTW: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_UNICODE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VALIDATE: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VALIDATEA: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VALIDATEW: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VERB: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VERBA: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VERBICONW: u32 = 20u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCS_VERBW: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCT_INVALID: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCT_LFNCHAR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCT_SEPARATOR: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCT_SHORTCHAR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GCT_WILD: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GETPROPS_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_ASYNC: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_CHECKSHIELD: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_DEFAULTICON: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_DONTCACHE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_FORCENOSHIELD: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_FORSHELL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_FORSHORTCUT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_NOTFILENAME: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_OPENICON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_PERCLASS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_PERINSTANCE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_SHIELD: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GIL_SIMULATEDOC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GPFIDL_ALTNAME: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GPFIDL_DEFAULT: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GPFIDL_UNCPRINTER: i32 = 2i32;
pub const GenericCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 634108310, data2: 37613, data3: 17790, data4: [178, 140, 71, 116, 8, 75, 213, 98] };
pub type HDROP = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct HELPINFO {
    pub cbSize: u32,
    pub iContextType: i32,
    pub iCtrlId: i32,
    pub hItemHandle: super::super::Foundation::HANDLE,
    pub dwContextId: usize,
    pub MousePos: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for HELPINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for HELPINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct HELPWININFOA {
    pub wStructSize: i32,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub wMax: i32,
    pub rgchMember: [super::super::Foundation::CHAR; 2],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for HELPWININFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for HELPWININFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct HELPWININFOW {
    pub wStructSize: i32,
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub wMax: i32,
    pub rgchMember: [u16; 2],
}
impl ::core::marker::Copy for HELPWININFOW {}
impl ::core::clone::Clone for HELPWININFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLBWIF_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLBWIF_HASFRAMEWNDINFO: HLBWIF_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLBWIF_HASDOCWNDINFO: HLBWIF_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLBWIF_FRAMEWNDMAXIMIZED: HLBWIF_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLBWIF_DOCWNDMAXIMIZED: HLBWIF_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLBWIF_HASWEBTOOLBARINFO: HLBWIF_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLBWIF_WEBTOOLBARHIDDEN: HLBWIF_FLAGS = 32u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct HLBWINFO {
    pub cbSize: u32,
    pub grfHLBWIF: u32,
    pub rcFramePos: super::super::Foundation::RECT,
    pub rcDocPos: super::super::Foundation::RECT,
    pub hltbinfo: HLTBINFO,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for HLBWINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for HLBWINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLFNAMEF = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLFNAMEF_DEFAULT: HLFNAMEF = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLFNAMEF_TRYCACHE: HLFNAMEF = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLFNAMEF_TRYPRETTYTARGET: HLFNAMEF = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLFNAMEF_TRYFULLTARGET: HLFNAMEF = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLFNAMEF_TRYWIN95SHORTCUT: HLFNAMEF = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLID_INFO = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLID_INVALID: HLID_INFO = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLID_PREVIOUS: HLID_INFO = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLID_NEXT: HLID_INFO = 4294967294u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLID_CURRENT: HLID_INFO = 4294967293u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLID_STACKBOTTOM: HLID_INFO = 4294967292u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLID_STACKTOP: HLID_INFO = 4294967291u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLINKGETREF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKGETREF_DEFAULT: HLINKGETREF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKGETREF_ABSOLUTE: HLINKGETREF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKGETREF_RELATIVE: HLINKGETREF = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLINKMISC = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKMISC_RELATIVE: HLINKMISC = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLINKSETF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKSETF_TARGET: HLINKSETF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKSETF_LOCATION: HLINKSETF = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLINKWHICHMK = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKWHICHMK_CONTAINER: HLINKWHICHMK = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINKWHICHMK_BASE: HLINKWHICHMK = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINK_E_FIRST: ::windows_sys::core::HRESULT = -2147221248i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINK_S_DONTHIDE: i32 = 262400i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLINK_S_FIRST: ::windows_sys::core::HRESULT = 262400i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct HLITEM {
    pub uHLID: u32,
    pub pwzFriendlyName: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for HLITEM {}
impl ::core::clone::Clone for HLITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLNF = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_INTERNALJUMP: HLNF = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_OPENINNEWWINDOW: HLNF = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_NAVIGATINGBACK: HLNF = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_NAVIGATINGFORWARD: HLNF = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_NAVIGATINGTOSTACKITEM: HLNF = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_CREATENOHISTORY: HLNF = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_ALLOW_AUTONAVIGATE: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_CALLERUNTRUSTED: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_DISABLEWINDOWRESTRICTIONS: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_EXTERNALNAVIGATE: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_NEWWINDOWSMANAGED: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_TRUSTEDFORACTIVEX: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_TRUSTFIRSTDOWNLOAD: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLNF_UNTRUSTEDFORDOWNLOAD: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLQF_INFO = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLQF_ISVALID: HLQF_INFO = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLQF_ISCURRENT: HLQF_INFO = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct HLTBINFO {
    pub uDockType: u32,
    pub rcTbPos: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for HLTBINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for HLTBINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HLTB_INFO = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTB_DOCKEDLEFT: HLTB_INFO = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTB_DOCKEDTOP: HLTB_INFO = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTB_DOCKEDRIGHT: HLTB_INFO = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTB_DOCKEDBOTTOM: HLTB_INFO = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTB_FLOATING: HLTB_INFO = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type HOMEGROUPSHARINGCHOICES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HGSC_NONE: HOMEGROUPSHARINGCHOICES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HGSC_MUSICLIBRARY: HOMEGROUPSHARINGCHOICES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HGSC_PICTURESLIBRARY: HOMEGROUPSHARINGCHOICES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HGSC_VIDEOSLIBRARY: HOMEGROUPSHARINGCHOICES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HGSC_DOCUMENTSLIBRARY: HOMEGROUPSHARINGCHOICES = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HGSC_PRINTERS: HOMEGROUPSHARINGCHOICES = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HOMEGROUP_SECURITY_GROUP: &str = "HomeUsers";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HOMEGROUP_SECURITY_GROUP_MULTI: &str = "HUG";
pub type HPSXA = isize;
pub const HideInputPaneAnimationCoordinator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 944194225, data2: 10871, data3: 19635, data4: [140, 248, 17, 54, 245, 225, 126, 89] };
pub const HomeGroup: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3732388356, data2: 15506, data3: 19729, data4: [161, 165, 66, 53, 42, 83, 224, 227] };
pub type IACList = *mut ::core::ffi::c_void;
pub type IACList2 = *mut ::core::ffi::c_void;
pub type IAccessibilityDockingService = *mut ::core::ffi::c_void;
pub type IAccessibilityDockingServiceCallback = *mut ::core::ffi::c_void;
pub type IAccessibleObject = *mut ::core::ffi::c_void;
pub type IActionProgress = *mut ::core::ffi::c_void;
pub type IActionProgressDialog = *mut ::core::ffi::c_void;
pub type IAppActivationUIInfo = *mut ::core::ffi::c_void;
pub type IAppPublisher = *mut ::core::ffi::c_void;
pub type IAppVisibility = *mut ::core::ffi::c_void;
pub type IAppVisibilityEvents = *mut ::core::ffi::c_void;
pub type IApplicationActivationManager = *mut ::core::ffi::c_void;
pub type IApplicationAssociationRegistration = *mut ::core::ffi::c_void;
pub type IApplicationAssociationRegistrationUI = *mut ::core::ffi::c_void;
pub type IApplicationDesignModeSettings = *mut ::core::ffi::c_void;
pub type IApplicationDesignModeSettings2 = *mut ::core::ffi::c_void;
pub type IApplicationDestinations = *mut ::core::ffi::c_void;
pub type IApplicationDocumentLists = *mut ::core::ffi::c_void;
pub type IAssocHandler = *mut ::core::ffi::c_void;
pub type IAssocHandlerInvoker = *mut ::core::ffi::c_void;
pub type IAttachmentExecute = *mut ::core::ffi::c_void;
pub type IAutoComplete = *mut ::core::ffi::c_void;
pub type IAutoComplete2 = *mut ::core::ffi::c_void;
pub type IAutoCompleteDropDown = *mut ::core::ffi::c_void;
pub type IBandHost = *mut ::core::ffi::c_void;
pub type IBandSite = *mut ::core::ffi::c_void;
pub type IBannerNotificationHandler = *mut ::core::ffi::c_void;
pub type IBanneredBar = *mut ::core::ffi::c_void;
pub type IBrowserFrameOptions = *mut ::core::ffi::c_void;
pub type IBrowserService = *mut ::core::ffi::c_void;
pub type IBrowserService2 = *mut ::core::ffi::c_void;
pub type IBrowserService3 = *mut ::core::ffi::c_void;
pub type IBrowserService4 = *mut ::core::ffi::c_void;
pub type ICDBurn = *mut ::core::ffi::c_void;
pub type ICDBurnExt = *mut ::core::ffi::c_void;
pub type ICategorizer = *mut ::core::ffi::c_void;
pub type ICategoryProvider = *mut ::core::ffi::c_void;
pub type IColumnManager = *mut ::core::ffi::c_void;
pub type IColumnProvider = *mut ::core::ffi::c_void;
pub type ICommDlgBrowser = *mut ::core::ffi::c_void;
pub type ICommDlgBrowser2 = *mut ::core::ffi::c_void;
pub type ICommDlgBrowser3 = *mut ::core::ffi::c_void;
pub type IComputerInfoChangeNotify = *mut ::core::ffi::c_void;
pub type IConnectableCredentialProviderCredential = *mut ::core::ffi::c_void;
pub type IContactManagerInterop = *mut ::core::ffi::c_void;
pub type IContextMenu = *mut ::core::ffi::c_void;
pub type IContextMenu2 = *mut ::core::ffi::c_void;
pub type IContextMenu3 = *mut ::core::ffi::c_void;
pub type IContextMenuCB = *mut ::core::ffi::c_void;
pub type IContextMenuSite = *mut ::core::ffi::c_void;
pub type ICopyHookA = *mut ::core::ffi::c_void;
pub type ICopyHookW = *mut ::core::ffi::c_void;
pub type ICreateProcessInputs = *mut ::core::ffi::c_void;
pub type ICreatingProcess = *mut ::core::ffi::c_void;
pub type ICredentialProvider = *mut ::core::ffi::c_void;
pub type ICredentialProviderCredential = *mut ::core::ffi::c_void;
pub type ICredentialProviderCredential2 = *mut ::core::ffi::c_void;
pub type ICredentialProviderCredentialEvents = *mut ::core::ffi::c_void;
pub type ICredentialProviderCredentialEvents2 = *mut ::core::ffi::c_void;
pub type ICredentialProviderCredentialWithFieldOptions = *mut ::core::ffi::c_void;
pub type ICredentialProviderEvents = *mut ::core::ffi::c_void;
pub type ICredentialProviderFilter = *mut ::core::ffi::c_void;
pub type ICredentialProviderSetUserArray = *mut ::core::ffi::c_void;
pub type ICredentialProviderUser = *mut ::core::ffi::c_void;
pub type ICredentialProviderUserArray = *mut ::core::ffi::c_void;
pub type ICurrentItem = *mut ::core::ffi::c_void;
pub type ICurrentWorkingDirectory = *mut ::core::ffi::c_void;
pub type ICustomDestinationList = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDC_OFFLINE_HAND: u32 = 103u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDC_PANTOOL_HAND_CLOSED: u32 = 105u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDC_PANTOOL_HAND_OPEN: u32 = 104u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDD_WIZEXTN_FIRST: u32 = 20480u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDD_WIZEXTN_LAST: u32 = 20736u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDO_SHGIOI_DEFAULT: u64 = 4294967292u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDO_SHGIOI_LINK: u32 = 268435454u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDO_SHGIOI_SHARE: u32 = 268435455u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDO_SHGIOI_SLOWFILE: u64 = 4294967293u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IDS_DESCRIPTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ID_APP: u32 = 100u32;
pub type IDataObjectAsyncCapability = *mut ::core::ffi::c_void;
pub type IDataObjectProvider = *mut ::core::ffi::c_void;
pub type IDataTransferManagerInterop = *mut ::core::ffi::c_void;
pub type IDefaultExtractIconInit = *mut ::core::ffi::c_void;
pub type IDefaultFolderMenuInitialize = *mut ::core::ffi::c_void;
pub type IDelegateFolder = *mut ::core::ffi::c_void;
pub type IDelegateItem = *mut ::core::ffi::c_void;
pub type IDeskBand = *mut ::core::ffi::c_void;
pub type IDeskBand2 = *mut ::core::ffi::c_void;
pub type IDeskBandInfo = *mut ::core::ffi::c_void;
pub type IDeskBar = *mut ::core::ffi::c_void;
pub type IDeskBarClient = *mut ::core::ffi::c_void;
pub type IDesktopGadget = *mut ::core::ffi::c_void;
pub type IDesktopWallpaper = *mut ::core::ffi::c_void;
pub type IDestinationStreamFactory = *mut ::core::ffi::c_void;
pub type IDisplayItem = *mut ::core::ffi::c_void;
pub type IDocViewSite = *mut ::core::ffi::c_void;
pub type IDockingWindow = *mut ::core::ffi::c_void;
pub type IDockingWindowFrame = *mut ::core::ffi::c_void;
pub type IDockingWindowSite = *mut ::core::ffi::c_void;
pub type IDragSourceHelper = *mut ::core::ffi::c_void;
pub type IDragSourceHelper2 = *mut ::core::ffi::c_void;
pub type IDropTargetHelper = *mut ::core::ffi::c_void;
pub type IDynamicHWHandler = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_ASPECT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_ASYNC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_CACHE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_GLEAM: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_NOBORDER: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_NOSTAMP: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_OFFLINE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_ORIGSIZE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_QUALITY: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_REFRESH: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIFLAG_SCREEN: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEIT_PRIORITY_NORMAL: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEI_PRIORITY_MAX: u32 = 2147483647u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEI_PRIORITY_MIN: u32 = 0u32;
pub const IENamespaceTreeControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2900700419, data2: 58829, data3: 19232, data4: [130, 255, 231, 27, 17, 190, 174, 29] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type IEPDNFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IEPDN_BINDINGUI: IEPDNFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type IESHORTCUTFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IESHORTCUT_NEWBROWSER: IESHORTCUTFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IESHORTCUT_OPENNEWTAB: IESHORTCUTFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IESHORTCUT_FORCENAVIGATE: IESHORTCUTFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IESHORTCUT_BACKGROUNDTAB: IESHORTCUTFLAGS = 8i32;
pub type IEnumACString = *mut ::core::ffi::c_void;
pub type IEnumAssocHandlers = *mut ::core::ffi::c_void;
pub type IEnumExplorerCommand = *mut ::core::ffi::c_void;
pub type IEnumExtraSearch = *mut ::core::ffi::c_void;
pub type IEnumFullIDList = *mut ::core::ffi::c_void;
pub type IEnumHLITEM = *mut ::core::ffi::c_void;
pub type IEnumIDList = *mut ::core::ffi::c_void;
pub type IEnumObjects = *mut ::core::ffi::c_void;
pub type IEnumPublishedApps = *mut ::core::ffi::c_void;
pub type IEnumReadyCallback = *mut ::core::ffi::c_void;
pub type IEnumResources = *mut ::core::ffi::c_void;
pub type IEnumShellItems = *mut ::core::ffi::c_void;
pub type IEnumSyncMgrConflict = *mut ::core::ffi::c_void;
pub type IEnumSyncMgrEvents = *mut ::core::ffi::c_void;
pub type IEnumSyncMgrSyncItems = *mut ::core::ffi::c_void;
pub type IEnumTravelLogEntry = *mut ::core::ffi::c_void;
pub type IEnumerableView = *mut ::core::ffi::c_void;
pub type IExecuteCommand = *mut ::core::ffi::c_void;
pub type IExecuteCommandApplicationHostEnvironment = *mut ::core::ffi::c_void;
pub type IExecuteCommandHost = *mut ::core::ffi::c_void;
pub type IExpDispSupport = *mut ::core::ffi::c_void;
pub type IExpDispSupportXP = *mut ::core::ffi::c_void;
pub type IExplorerBrowser = *mut ::core::ffi::c_void;
pub type IExplorerBrowserEvents = *mut ::core::ffi::c_void;
pub type IExplorerCommand = *mut ::core::ffi::c_void;
pub type IExplorerCommandProvider = *mut ::core::ffi::c_void;
pub type IExplorerCommandState = *mut ::core::ffi::c_void;
pub type IExplorerPaneVisibility = *mut ::core::ffi::c_void;
pub type IExtensionServices = *mut ::core::ffi::c_void;
pub type IExtractIconA = *mut ::core::ffi::c_void;
pub type IExtractIconW = *mut ::core::ffi::c_void;
pub type IExtractImage = *mut ::core::ffi::c_void;
pub type IExtractImage2 = *mut ::core::ffi::c_void;
pub type IFileDialog = *mut ::core::ffi::c_void;
pub type IFileDialog2 = *mut ::core::ffi::c_void;
pub type IFileDialogControlEvents = *mut ::core::ffi::c_void;
pub type IFileDialogCustomize = *mut ::core::ffi::c_void;
pub type IFileDialogEvents = *mut ::core::ffi::c_void;
pub type IFileIsInUse = *mut ::core::ffi::c_void;
pub type IFileOpenDialog = *mut ::core::ffi::c_void;
pub type IFileOperation = *mut ::core::ffi::c_void;
pub type IFileOperation2 = *mut ::core::ffi::c_void;
pub type IFileOperationProgressSink = *mut ::core::ffi::c_void;
pub type IFileSaveDialog = *mut ::core::ffi::c_void;
pub type IFileSearchBand = *mut ::core::ffi::c_void;
pub type IFileSyncMergeHandler = *mut ::core::ffi::c_void;
pub type IFileSystemBindData = *mut ::core::ffi::c_void;
pub type IFileSystemBindData2 = *mut ::core::ffi::c_void;
pub type IFolderBandPriv = *mut ::core::ffi::c_void;
pub type IFolderFilter = *mut ::core::ffi::c_void;
pub type IFolderFilterSite = *mut ::core::ffi::c_void;
pub type IFolderView = *mut ::core::ffi::c_void;
pub type IFolderView2 = *mut ::core::ffi::c_void;
pub type IFolderViewHost = *mut ::core::ffi::c_void;
pub type IFolderViewOC = *mut ::core::ffi::c_void;
pub type IFolderViewOptions = *mut ::core::ffi::c_void;
pub type IFolderViewSettings = *mut ::core::ffi::c_void;
pub type IFrameworkInputPane = *mut ::core::ffi::c_void;
pub type IFrameworkInputPaneHandler = *mut ::core::ffi::c_void;
pub type IGetServiceIds = *mut ::core::ffi::c_void;
pub type IHWEventHandler = *mut ::core::ffi::c_void;
pub type IHWEventHandler2 = *mut ::core::ffi::c_void;
pub type IHandlerActivationHost = *mut ::core::ffi::c_void;
pub type IHandlerInfo = *mut ::core::ffi::c_void;
pub type IHandlerInfo2 = *mut ::core::ffi::c_void;
pub type IHlink = *mut ::core::ffi::c_void;
pub type IHlinkBrowseContext = *mut ::core::ffi::c_void;
pub type IHlinkFrame = *mut ::core::ffi::c_void;
pub type IHlinkSite = *mut ::core::ffi::c_void;
pub type IHlinkTarget = *mut ::core::ffi::c_void;
pub type IHomeGroup = *mut ::core::ffi::c_void;
pub type IIOCancelInformation = *mut ::core::ffi::c_void;
pub type IIdentityName = *mut ::core::ffi::c_void;
pub type IImageRecompress = *mut ::core::ffi::c_void;
pub type IInitializeCommand = *mut ::core::ffi::c_void;
pub type IInitializeNetworkFolder = *mut ::core::ffi::c_void;
pub type IInitializeObject = *mut ::core::ffi::c_void;
pub type IInitializeWithBindCtx = *mut ::core::ffi::c_void;
pub type IInitializeWithItem = *mut ::core::ffi::c_void;
pub type IInitializeWithPropertyStore = *mut ::core::ffi::c_void;
pub type IInitializeWithWindow = *mut ::core::ffi::c_void;
pub type IInputObject = *mut ::core::ffi::c_void;
pub type IInputObject2 = *mut ::core::ffi::c_void;
pub type IInputObjectSite = *mut ::core::ffi::c_void;
pub type IInputPaneAnimationCoordinator = *mut ::core::ffi::c_void;
pub type IInputPanelConfiguration = *mut ::core::ffi::c_void;
pub type IInputPanelInvocationConfiguration = *mut ::core::ffi::c_void;
pub type IInsertItem = *mut ::core::ffi::c_void;
pub type IItemNameLimits = *mut ::core::ffi::c_void;
pub type IKnownFolder = *mut ::core::ffi::c_void;
pub type IKnownFolderManager = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ILMM_IE4: u32 = 0u32;
pub type ILaunchSourceAppUserModelId = *mut ::core::ffi::c_void;
pub type ILaunchSourceViewSizePreference = *mut ::core::ffi::c_void;
pub type ILaunchTargetMonitor = *mut ::core::ffi::c_void;
pub type ILaunchTargetViewSizePreference = *mut ::core::ffi::c_void;
pub type ILaunchUIContext = *mut ::core::ffi::c_void;
pub type ILaunchUIContextProvider = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IMM_ACC_DOCKING_E_DOCKOCCUPIED: ::windows_sys::core::HRESULT = -2144927183i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IMM_ACC_DOCKING_E_INSUFFICIENTHEIGHT: ::windows_sys::core::HRESULT = -2144927184i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IMSC_E_SHELL_COMPONENT_STARTUP_FAILURE: ::windows_sys::core::HRESULT = -2144927181i32;
pub type IMenuBand = *mut ::core::ffi::c_void;
pub type IMenuPopup = *mut ::core::ffi::c_void;
pub type IModalWindow = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const INTERNET_MAX_PATH_LENGTH: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const INTERNET_MAX_SCHEME_LENGTH: u32 = 32u32;
pub type INameSpaceTreeAccessible = *mut ::core::ffi::c_void;
pub type INameSpaceTreeControl = *mut ::core::ffi::c_void;
pub type INameSpaceTreeControl2 = *mut ::core::ffi::c_void;
pub type INameSpaceTreeControlCustomDraw = *mut ::core::ffi::c_void;
pub type INameSpaceTreeControlDropHandler = *mut ::core::ffi::c_void;
pub type INameSpaceTreeControlEvents = *mut ::core::ffi::c_void;
pub type INameSpaceTreeControlFolderCapabilities = *mut ::core::ffi::c_void;
pub type INamedPropertyBag = *mut ::core::ffi::c_void;
pub type INamespaceWalk = *mut ::core::ffi::c_void;
pub type INamespaceWalkCB = *mut ::core::ffi::c_void;
pub type INamespaceWalkCB2 = *mut ::core::ffi::c_void;
pub type INetworkFolderInternal = *mut ::core::ffi::c_void;
pub type INewMenuClient = *mut ::core::ffi::c_void;
pub type INewShortcutHookA = *mut ::core::ffi::c_void;
pub type INewShortcutHookW = *mut ::core::ffi::c_void;
pub type INewWDEvents = *mut ::core::ffi::c_void;
pub type INewWindowManager = *mut ::core::ffi::c_void;
pub type INotifyReplica = *mut ::core::ffi::c_void;
pub type IObjMgr = *mut ::core::ffi::c_void;
pub type IObjectProvider = *mut ::core::ffi::c_void;
pub type IObjectWithAppUserModelID = *mut ::core::ffi::c_void;
pub type IObjectWithBackReferences = *mut ::core::ffi::c_void;
pub type IObjectWithCancelEvent = *mut ::core::ffi::c_void;
pub type IObjectWithFolderEnumMode = *mut ::core::ffi::c_void;
pub type IObjectWithProgID = *mut ::core::ffi::c_void;
pub type IObjectWithSelection = *mut ::core::ffi::c_void;
pub type IOpenControlPanel = *mut ::core::ffi::c_void;
pub type IOpenSearchSource = *mut ::core::ffi::c_void;
pub type IOperationsProgressDialog = *mut ::core::ffi::c_void;
pub type IPackageDebugSettings = *mut ::core::ffi::c_void;
pub type IPackageDebugSettings2 = *mut ::core::ffi::c_void;
pub type IPackageExecutionStateChangeNotification = *mut ::core::ffi::c_void;
pub type IParentAndItem = *mut ::core::ffi::c_void;
pub type IParseAndCreateItem = *mut ::core::ffi::c_void;
pub type IPersistFolder = *mut ::core::ffi::c_void;
pub type IPersistFolder2 = *mut ::core::ffi::c_void;
pub type IPersistFolder3 = *mut ::core::ffi::c_void;
pub type IPersistIDList = *mut ::core::ffi::c_void;
pub type IPreviewHandler = *mut ::core::ffi::c_void;
pub type IPreviewHandlerFrame = *mut ::core::ffi::c_void;
pub type IPreviewHandlerVisuals = *mut ::core::ffi::c_void;
pub type IPreviewItem = *mut ::core::ffi::c_void;
pub type IPreviousVersionsInfo = *mut ::core::ffi::c_void;
pub type IProfferService = *mut ::core::ffi::c_void;
pub type IProgressDialog = *mut ::core::ffi::c_void;
pub type IPropertyKeyStore = *mut ::core::ffi::c_void;
pub type IPublishedApp = *mut ::core::ffi::c_void;
pub type IPublishedApp2 = *mut ::core::ffi::c_void;
pub type IPublishingWizard = *mut ::core::ffi::c_void;
pub type IQueryAssociations = *mut ::core::ffi::c_void;
pub type IQueryCancelAutoPlay = *mut ::core::ffi::c_void;
pub type IQueryCodePage = *mut ::core::ffi::c_void;
pub type IQueryContinue = *mut ::core::ffi::c_void;
pub type IQueryContinueWithStatus = *mut ::core::ffi::c_void;
pub type IQueryInfo = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IRTIR_TASK_FINISHED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IRTIR_TASK_NOT_RUNNING: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IRTIR_TASK_PENDING: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IRTIR_TASK_RUNNING: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IRTIR_TASK_SUSPENDED: u32 = 2u32;
pub type IRegTreeItem = *mut ::core::ffi::c_void;
pub type IRelatedItem = *mut ::core::ffi::c_void;
pub type IRemoteComputer = *mut ::core::ffi::c_void;
pub type IResolveShellLink = *mut ::core::ffi::c_void;
pub type IResultsFolder = *mut ::core::ffi::c_void;
pub type IRunnableTask = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFBVIEWMODE_LARGEICONS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFBVIEWMODE_LOGOS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFBVIEWMODE_SMALLICONS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_MASK_BKCOLOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_MASK_COLORS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_MASK_IDLIST: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_MASK_SHELLFOLDER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_MASK_STATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_MASK_VIEWMODE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_ALLOWRENAME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_BTNMINSIZE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_CHANNELBAR: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_DEBOSSED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_FULLOPEN: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_NONAMESORT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_NOSHOWTEXT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISFB_STATE_QLINKSMODE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISHCUTCMDID_COMMITHISTORY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISHCUTCMDID_DOWNLOADICON: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISHCUTCMDID_INTSHORTCUTCREATE: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISHCUTCMDID_SETUSERAWURL: i32 = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISIOI_ICONFILE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ISIOI_ICONINDEX: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IS_E_EXEC_FAILED: ::windows_sys::core::HRESULT = -2147213310i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IS_FULLSCREEN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IS_NORMAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IS_SPLIT: u32 = 4u32;
pub type IScriptErrorList = *mut ::core::ffi::c_void;
pub type ISearchBoxInfo = *mut ::core::ffi::c_void;
pub type ISearchContext = *mut ::core::ffi::c_void;
pub type ISearchFolderItemFactory = *mut ::core::ffi::c_void;
pub type ISharedBitmap = *mut ::core::ffi::c_void;
pub type ISharingConfigurationManager = *mut ::core::ffi::c_void;
pub type IShellApp = *mut ::core::ffi::c_void;
pub type IShellBrowser = *mut ::core::ffi::c_void;
pub type IShellChangeNotify = *mut ::core::ffi::c_void;
pub type IShellDetails = *mut ::core::ffi::c_void;
pub type IShellDispatch = *mut ::core::ffi::c_void;
pub type IShellDispatch2 = *mut ::core::ffi::c_void;
pub type IShellDispatch3 = *mut ::core::ffi::c_void;
pub type IShellDispatch4 = *mut ::core::ffi::c_void;
pub type IShellDispatch5 = *mut ::core::ffi::c_void;
pub type IShellDispatch6 = *mut ::core::ffi::c_void;
pub type IShellExtInit = *mut ::core::ffi::c_void;
pub type IShellFavoritesNameSpace = *mut ::core::ffi::c_void;
pub type IShellFolder = *mut ::core::ffi::c_void;
pub type IShellFolder2 = *mut ::core::ffi::c_void;
pub type IShellFolderBand = *mut ::core::ffi::c_void;
pub type IShellFolderView = *mut ::core::ffi::c_void;
pub type IShellFolderViewCB = *mut ::core::ffi::c_void;
pub type IShellFolderViewDual = *mut ::core::ffi::c_void;
pub type IShellFolderViewDual2 = *mut ::core::ffi::c_void;
pub type IShellFolderViewDual3 = *mut ::core::ffi::c_void;
pub type IShellIcon = *mut ::core::ffi::c_void;
pub type IShellIconOverlay = *mut ::core::ffi::c_void;
pub type IShellIconOverlayIdentifier = *mut ::core::ffi::c_void;
pub type IShellIconOverlayManager = *mut ::core::ffi::c_void;
pub type IShellImageData = *mut ::core::ffi::c_void;
pub type IShellImageDataAbort = *mut ::core::ffi::c_void;
pub type IShellImageDataFactory = *mut ::core::ffi::c_void;
pub type IShellItem = *mut ::core::ffi::c_void;
pub type IShellItem2 = *mut ::core::ffi::c_void;
pub type IShellItemArray = *mut ::core::ffi::c_void;
pub type IShellItemFilter = *mut ::core::ffi::c_void;
pub type IShellItemImageFactory = *mut ::core::ffi::c_void;
pub type IShellItemResources = *mut ::core::ffi::c_void;
pub type IShellLibrary = *mut ::core::ffi::c_void;
pub type IShellLinkA = *mut ::core::ffi::c_void;
pub type IShellLinkDataList = *mut ::core::ffi::c_void;
pub type IShellLinkDual = *mut ::core::ffi::c_void;
pub type IShellLinkDual2 = *mut ::core::ffi::c_void;
pub type IShellLinkW = *mut ::core::ffi::c_void;
pub type IShellMenu = *mut ::core::ffi::c_void;
pub type IShellMenuCallback = *mut ::core::ffi::c_void;
pub type IShellNameSpace = *mut ::core::ffi::c_void;
pub type IShellPropSheetExt = *mut ::core::ffi::c_void;
pub type IShellRunDll = *mut ::core::ffi::c_void;
pub type IShellService = *mut ::core::ffi::c_void;
pub type IShellTaskScheduler = *mut ::core::ffi::c_void;
pub type IShellUIHelper = *mut ::core::ffi::c_void;
pub type IShellUIHelper2 = *mut ::core::ffi::c_void;
pub type IShellUIHelper3 = *mut ::core::ffi::c_void;
pub type IShellUIHelper4 = *mut ::core::ffi::c_void;
pub type IShellUIHelper5 = *mut ::core::ffi::c_void;
pub type IShellUIHelper6 = *mut ::core::ffi::c_void;
pub type IShellUIHelper7 = *mut ::core::ffi::c_void;
pub type IShellUIHelper8 = *mut ::core::ffi::c_void;
pub type IShellUIHelper9 = *mut ::core::ffi::c_void;
pub type IShellView = *mut ::core::ffi::c_void;
pub type IShellView2 = *mut ::core::ffi::c_void;
pub type IShellView3 = *mut ::core::ffi::c_void;
pub type IShellWindows = *mut ::core::ffi::c_void;
pub type ISortColumnArray = *mut ::core::ffi::c_void;
pub type IStartMenuPinnedList = *mut ::core::ffi::c_void;
pub type IStorageProviderBanners = *mut ::core::ffi::c_void;
pub type IStorageProviderCopyHook = *mut ::core::ffi::c_void;
pub type IStorageProviderHandler = *mut ::core::ffi::c_void;
pub type IStorageProviderPropertyHandler = *mut ::core::ffi::c_void;
pub type IStreamAsync = *mut ::core::ffi::c_void;
pub type IStreamUnbufferedInfo = *mut ::core::ffi::c_void;
pub type ISuspensionDependencyManager = *mut ::core::ffi::c_void;
pub type ISyncMgrConflict = *mut ::core::ffi::c_void;
pub type ISyncMgrConflictFolder = *mut ::core::ffi::c_void;
pub type ISyncMgrConflictItems = *mut ::core::ffi::c_void;
pub type ISyncMgrConflictPresenter = *mut ::core::ffi::c_void;
pub type ISyncMgrConflictResolutionItems = *mut ::core::ffi::c_void;
pub type ISyncMgrConflictResolveInfo = *mut ::core::ffi::c_void;
pub type ISyncMgrConflictStore = *mut ::core::ffi::c_void;
pub type ISyncMgrControl = *mut ::core::ffi::c_void;
pub type ISyncMgrEnumItems = *mut ::core::ffi::c_void;
pub type ISyncMgrEvent = *mut ::core::ffi::c_void;
pub type ISyncMgrEventLinkUIOperation = *mut ::core::ffi::c_void;
pub type ISyncMgrEventStore = *mut ::core::ffi::c_void;
pub type ISyncMgrHandler = *mut ::core::ffi::c_void;
pub type ISyncMgrHandlerCollection = *mut ::core::ffi::c_void;
pub type ISyncMgrHandlerInfo = *mut ::core::ffi::c_void;
pub type ISyncMgrRegister = *mut ::core::ffi::c_void;
pub type ISyncMgrResolutionHandler = *mut ::core::ffi::c_void;
pub type ISyncMgrScheduleWizardUIOperation = *mut ::core::ffi::c_void;
pub type ISyncMgrSessionCreator = *mut ::core::ffi::c_void;
pub type ISyncMgrSyncCallback = *mut ::core::ffi::c_void;
pub type ISyncMgrSyncItem = *mut ::core::ffi::c_void;
pub type ISyncMgrSyncItemContainer = *mut ::core::ffi::c_void;
pub type ISyncMgrSyncItemInfo = *mut ::core::ffi::c_void;
pub type ISyncMgrSyncResult = *mut ::core::ffi::c_void;
pub type ISyncMgrSynchronize = *mut ::core::ffi::c_void;
pub type ISyncMgrSynchronizeCallback = *mut ::core::ffi::c_void;
pub type ISyncMgrSynchronizeInvoke = *mut ::core::ffi::c_void;
pub type ISyncMgrUIOperation = *mut ::core::ffi::c_void;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct ITEMSPACING {
    pub cxSmall: i32,
    pub cySmall: i32,
    pub cxLarge: i32,
    pub cyLarge: i32,
}
impl ::core::marker::Copy for ITEMSPACING {}
impl ::core::clone::Clone for ITEMSPACING {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ITSAT_DEFAULT_PRIORITY: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ITSAT_MAX_PRIORITY: u32 = 2147483647u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ITSAT_MIN_PRIORITY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ITSSFLAG_COMPLETE_ON_DESTROY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ITSSFLAG_FLAGS_MASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ITSSFLAG_KILL_ON_DESTROY: u32 = 1u32;
pub type ITaskbarList = *mut ::core::ffi::c_void;
pub type ITaskbarList2 = *mut ::core::ffi::c_void;
pub type ITaskbarList3 = *mut ::core::ffi::c_void;
pub type ITaskbarList4 = *mut ::core::ffi::c_void;
pub type IThumbnailCache = *mut ::core::ffi::c_void;
pub type IThumbnailCachePrimer = *mut ::core::ffi::c_void;
pub type IThumbnailCapture = *mut ::core::ffi::c_void;
pub type IThumbnailHandlerFactory = *mut ::core::ffi::c_void;
pub type IThumbnailProvider = *mut ::core::ffi::c_void;
pub type IThumbnailSettings = *mut ::core::ffi::c_void;
pub type IThumbnailStreamCache = *mut ::core::ffi::c_void;
pub type ITrackShellMenu = *mut ::core::ffi::c_void;
pub type ITranscodeImage = *mut ::core::ffi::c_void;
pub type ITransferAdviseSink = *mut ::core::ffi::c_void;
pub type ITransferDestination = *mut ::core::ffi::c_void;
pub type ITransferMediumItem = *mut ::core::ffi::c_void;
pub type ITransferSource = *mut ::core::ffi::c_void;
pub type ITravelEntry = *mut ::core::ffi::c_void;
pub type ITravelLog = *mut ::core::ffi::c_void;
pub type ITravelLogClient = *mut ::core::ffi::c_void;
pub type ITravelLogEntry = *mut ::core::ffi::c_void;
pub type ITravelLogStg = *mut ::core::ffi::c_void;
pub type ITrayDeskBand = *mut ::core::ffi::c_void;
pub type IURLSearchHook = *mut ::core::ffi::c_void;
pub type IURLSearchHook2 = *mut ::core::ffi::c_void;
pub type IUniformResourceLocatorA = *mut ::core::ffi::c_void;
pub type IUniformResourceLocatorW = *mut ::core::ffi::c_void;
pub type IUpdateIDList = *mut ::core::ffi::c_void;
pub type IUseToBrowseItem = *mut ::core::ffi::c_void;
pub type IUserAccountChangeCallback = *mut ::core::ffi::c_void;
pub type IUserNotification = *mut ::core::ffi::c_void;
pub type IUserNotification2 = *mut ::core::ffi::c_void;
pub type IUserNotificationCallback = *mut ::core::ffi::c_void;
pub type IViewStateIdentityItem = *mut ::core::ffi::c_void;
pub type IVirtualDesktopManager = *mut ::core::ffi::c_void;
pub type IVisualProperties = *mut ::core::ffi::c_void;
pub type IWebBrowser = *mut ::core::ffi::c_void;
pub type IWebBrowser2 = *mut ::core::ffi::c_void;
pub type IWebBrowserApp = *mut ::core::ffi::c_void;
pub type IWebWizardExtension = *mut ::core::ffi::c_void;
pub type IWebWizardHost = *mut ::core::ffi::c_void;
pub type IWebWizardHost2 = *mut ::core::ffi::c_void;
pub type IWizardExtension = *mut ::core::ffi::c_void;
pub type IWizardSite = *mut ::core::ffi::c_void;
pub const Identity_LocalUserProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2711114395, data2: 29455, data3: 16521, data4: [182, 70, 161, 37, 87, 245, 102, 94] };
pub const ImageProperties: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2058842311, data2: 3619, data3: 19834, data4: [138, 162, 25, 191, 173, 71, 152, 41] };
pub const ImageRecompress: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1848838428, data2: 54008, data3: 18240, data4: [181, 94, 46, 17, 209, 71, 122, 44] };
pub const ImageTranscode: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 397889894, data2: 37519, data3: 16765, data4: [150, 133, 100, 170, 19, 85, 101, 193] };
pub const InputPanelConfiguration: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 676572627, data2: 61590, data3: 19555, data4: [167, 143, 127, 163, 234, 131, 127, 183] };
pub const InternetExplorer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 188161, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const InternetExplorerMedium: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3588752413, data2: 37391, data3: 17897, data4: [184, 251, 177, 222, 184, 44, 110, 94] };
pub const InternetPrintOrdering: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2916313768, data2: 29978, data3: 17785, data4: [162, 102, 214, 111, 82, 2, 204, 187] };
pub const ItemCount_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2881444933, data2: 23756, data3: 18359, data4: [187, 78, 135, 203, 135, 187, 209, 98] };
pub const ItemIndex_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2459980762, data2: 10601, data3: 16417, data4: [191, 39, 81, 76, 252, 46, 74, 105] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type KF_CATEGORY = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_CATEGORY_VIRTUAL: KF_CATEGORY = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_CATEGORY_FIXED: KF_CATEGORY = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_CATEGORY_COMMON: KF_CATEGORY = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_CATEGORY_PERUSER: KF_CATEGORY = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type KNOWNDESTCATEGORY = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KDC_FREQUENT: KNOWNDESTCATEGORY = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KDC_RECENT: KNOWNDESTCATEGORY = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct KNOWNFOLDER_DEFINITION {
    pub category: KF_CATEGORY,
    pub pszName: ::windows_sys::core::PWSTR,
    pub pszDescription: ::windows_sys::core::PWSTR,
    pub fidParent: ::windows_sys::core::GUID,
    pub pszRelativePath: ::windows_sys::core::PWSTR,
    pub pszParsingName: ::windows_sys::core::PWSTR,
    pub pszTooltip: ::windows_sys::core::PWSTR,
    pub pszLocalizedName: ::windows_sys::core::PWSTR,
    pub pszIcon: ::windows_sys::core::PWSTR,
    pub pszSecurity: ::windows_sys::core::PWSTR,
    pub dwAttributes: u32,
    pub kfdFlags: u32,
    pub ftidType: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for KNOWNFOLDER_DEFINITION {}
impl ::core::clone::Clone for KNOWNFOLDER_DEFINITION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type KNOWN_FOLDER_FLAG = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_DEFAULT: KNOWN_FOLDER_FLAG = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_FORCE_APP_DATA_REDIRECTION: KNOWN_FOLDER_FLAG = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_RETURN_FILTER_REDIRECTION_TARGET: KNOWN_FOLDER_FLAG = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_FORCE_PACKAGE_REDIRECTION: KNOWN_FOLDER_FLAG = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_NO_PACKAGE_REDIRECTION: KNOWN_FOLDER_FLAG = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_FORCE_APPCONTAINER_REDIRECTION: KNOWN_FOLDER_FLAG = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_NO_APPCONTAINER_REDIRECTION: KNOWN_FOLDER_FLAG = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_CREATE: KNOWN_FOLDER_FLAG = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_DONT_VERIFY: KNOWN_FOLDER_FLAG = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_DONT_UNEXPAND: KNOWN_FOLDER_FLAG = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_NO_ALIAS: KNOWN_FOLDER_FLAG = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_INIT: KNOWN_FOLDER_FLAG = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_DEFAULT_PATH: KNOWN_FOLDER_FLAG = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_NOT_PARENT_RELATIVE: KNOWN_FOLDER_FLAG = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_SIMPLE_IDLIST: KNOWN_FOLDER_FLAG = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_FLAG_ALIAS_ONLY: KNOWN_FOLDER_FLAG = -2147483648i32;
pub const KnownFolderManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1307625264, data2: 57245, data3: 19171, data4: [145, 83, 170, 107, 130, 233, 121, 90] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type LIBRARYFOLDERFILTER = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LFF_FORCEFILESYSTEM: LIBRARYFOLDERFILTER = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LFF_STORAGEITEMS: LIBRARYFOLDERFILTER = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LFF_ALLITEMS: LIBRARYFOLDERFILTER = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type LIBRARYMANAGEDIALOGOPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LMD_DEFAULT: LIBRARYMANAGEDIALOGOPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LMD_ALLOWUNINDEXABLENETWORKLOCATIONS: LIBRARYMANAGEDIALOGOPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type LIBRARYOPTIONFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LOF_DEFAULT: LIBRARYOPTIONFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LOF_PINNEDTONAVPANE: LIBRARYOPTIONFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LOF_MASK_ALL: LIBRARYOPTIONFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type LIBRARYSAVEFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LSF_FAILIFTHERE: LIBRARYSAVEFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LSF_OVERRIDEEXISTING: LIBRARYSAVEFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LSF_MAKEUNIQUENAME: LIBRARYSAVEFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LIBRARY_E_NO_ACCESSIBLE_LOCATION: ::windows_sys::core::HRESULT = -2144927231i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LIBRARY_E_NO_SAVE_LOCATION: ::windows_sys::core::HRESULT = -2144927232i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const LINK_E_DELETE: ::windows_sys::core::HRESULT = -2144927485i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
pub type LPFNDFMCALLBACK = ::core::option::Option<unsafe extern "system" fn(psf: IShellFolder, hwnd: super::super::Foundation::HWND, pdtobj: super::super::System::Com::IDataObject, umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole"))]
pub type LPFNVIEWCALLBACK = ::core::option::Option<unsafe extern "system" fn(psvouter: IShellView, psf: IShellFolder, hwndmain: super::super::Foundation::HWND, umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM) -> ::windows_sys::core::HRESULT>;
pub const LocalThumbnailCache: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1357858116, data2: 44191, data3: 19086, data4: [178, 27, 138, 38, 24, 13, 177, 63] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAXFILELEN: u32 = 13u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_COLUMN_DESC_LEN: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_COLUMN_NAME_LEN: u32 = 80u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_SYNCMGRHANDLERNAME: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_SYNCMGRITEMNAME: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_SYNCMGR_ID: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_SYNCMGR_NAME: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAX_SYNCMGR_PROGRESSTEXT: u32 = 260u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type MENUBANDHANDLERCID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MBHANDCID_PIDLSELECT: MENUBANDHANDLERCID = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type MENUPOPUPPOPUPFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_SETFOCUS: MENUPOPUPPOPUPFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_INITIALSELECT: MENUPOPUPPOPUPFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_NOANIMATE: MENUPOPUPPOPUPFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_KEYBOARD: MENUPOPUPPOPUPFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_REPOSITION: MENUPOPUPPOPUPFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_FORCEZORDER: MENUPOPUPPOPUPFLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_FINALSELECT: MENUPOPUPPOPUPFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_TOP: MENUPOPUPPOPUPFLAGS = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_LEFT: MENUPOPUPPOPUPFLAGS = 1073741824i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_RIGHT: MENUPOPUPPOPUPFLAGS = 1610612736i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_BOTTOM: MENUPOPUPPOPUPFLAGS = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_POS_MASK: MENUPOPUPPOPUPFLAGS = -536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_ALIGN_LEFT: MENUPOPUPPOPUPFLAGS = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPPF_ALIGN_RIGHT: MENUPOPUPPOPUPFLAGS = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type MENUPOPUPSELECT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPOS_EXECUTE: MENUPOPUPSELECT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPOS_FULLCANCEL: MENUPOPUPSELECT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPOS_CANCELLEVEL: MENUPOPUPSELECT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPOS_SELECTLEFT: MENUPOPUPSELECT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPOS_SELECTRIGHT: MENUPOPUPSELECT = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MPOS_CHILDTRACKING: MENUPOPUPSELECT = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type MERGE_UPDATE_STATUS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MUS_COMPLETE: MERGE_UPDATE_STATUS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MUS_USERINPUTNEEDED: MERGE_UPDATE_STATUS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MUS_FAILED: MERGE_UPDATE_STATUS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type MM_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MM_ADDSEPARATOR: MM_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MM_SUBMENUSHAVEIDS: MM_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MM_DONTREMOVESEPS: MM_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type MONITOR_APP_VISIBILITY = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAV_UNKNOWN: MONITOR_APP_VISIBILITY = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAV_NO_APP_VISIBLE: MONITOR_APP_VISIBILITY = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MAV_APP_VISIBLE: MONITOR_APP_VISIBILITY = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MULTIKEYHELPA {
    pub mkSize: u32,
    pub mkKeylist: super::super::Foundation::CHAR,
    pub szKeyphrase: [super::super::Foundation::CHAR; 1],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MULTIKEYHELPA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MULTIKEYHELPA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct MULTIKEYHELPW {
    pub mkSize: u32,
    pub mkKeylist: u16,
    pub szKeyphrase: [u16; 1],
}
impl ::core::marker::Copy for MULTIKEYHELPW {}
impl ::core::clone::Clone for MULTIKEYHELPW {
    fn clone(&self) -> Self {
        *self
    }
}
pub const MailRecipient: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2656484960, data2: 50447, data3: 4559, data4: [154, 44, 0, 160, 201, 10, 144, 206] };
pub const MergedCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2390916113, data2: 13287, data3: 19393, data4: [178, 66, 140, 217, 161, 194, 179, 4] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NAMESPACEWALKFLAG = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_DEFAULT: NAMESPACEWALKFLAG = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_NONE_IMPLIES_ALL: NAMESPACEWALKFLAG = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_ONE_IMPLIES_ALL: NAMESPACEWALKFLAG = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_DONT_TRAVERSE_LINKS: NAMESPACEWALKFLAG = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_DONT_ACCUMULATE_RESULT: NAMESPACEWALKFLAG = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_TRAVERSE_STREAM_JUNCTIONS: NAMESPACEWALKFLAG = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_FILESYSTEM_ONLY: NAMESPACEWALKFLAG = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_SHOW_PROGRESS: NAMESPACEWALKFLAG = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_FLAG_VIEWORDER: NAMESPACEWALKFLAG = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_IGNORE_AUTOPLAY_HIDA: NAMESPACEWALKFLAG = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_ASYNC: NAMESPACEWALKFLAG = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_DONT_RESOLVE_LINKS: NAMESPACEWALKFLAG = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_ACCUMULATE_FOLDERS: NAMESPACEWALKFLAG = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_DONT_SORT: NAMESPACEWALKFLAG = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_USE_TRANSFER_MEDIUM: NAMESPACEWALKFLAG = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_DONT_TRAVERSE_STREAM_JUNCTIONS: NAMESPACEWALKFLAG = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSWF_ANY_IMPLIES_ALL: NAMESPACEWALKFLAG = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NATIVE_DISPLAY_ORIENTATION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NDO_LANDSCAPE: NATIVE_DISPLAY_ORIENTATION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NDO_PORTRAIT: NATIVE_DISPLAY_ORIENTATION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NCM_DISPLAYERRORTIP: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NCM_GETADDRESS: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NCM_GETALLOWTYPE: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NCM_SETALLOWTYPE: u32 = 1026u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct NC_ADDRESS {
    pub pAddrInfo: *mut NC_ADDRESS_0,
    pub PortNumber: u16,
    pub PrefixLength: u8,
}
impl ::core::marker::Copy for NC_ADDRESS {}
impl ::core::clone::Clone for NC_ADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct NC_ADDRESS_0(pub u8);
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NETCACHE_E_NEGATIVE_CACHE: ::windows_sys::core::HRESULT = -2144927488i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct NEWCPLINFOA {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwHelpContext: u32,
    pub lData: isize,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szName: [super::super::Foundation::CHAR; 32],
    pub szInfo: [super::super::Foundation::CHAR; 64],
    pub szHelpFile: [super::super::Foundation::CHAR; 128],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NEWCPLINFOA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NEWCPLINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct NEWCPLINFOW {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub dwHelpContext: u32,
    pub lData: isize,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szName: [u16; 32],
    pub szInfo: [u16; 64],
    pub szHelpFile: [u16; 128],
}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for NEWCPLINFOW {}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for NEWCPLINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_ERROR: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_ICON_MASK: u32 = 15u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_INFO: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_LARGE_ICON: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_NOSOUND: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_RESPECT_QUIET_TIME: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_USER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIIF_WARNING: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NINF_KEY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_BALLOONHIDE: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_BALLOONSHOW: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_BALLOONTIMEOUT: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_BALLOONUSERCLICK: u32 = 1029u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_POPUPCLOSE: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_POPUPOPEN: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIN_SELECT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIS_HIDDEN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIS_SHAREDICON: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct NOTIFYICONDATAA {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uID: u32,
    pub uFlags: NOTIFY_ICON_DATA_FLAGS,
    pub uCallbackMessage: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szTip: [super::super::Foundation::CHAR; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [super::super::Foundation::CHAR; 256],
    pub Anonymous: NOTIFYICONDATAA_0,
    pub szInfoTitle: [super::super::Foundation::CHAR; 64],
    pub dwInfoFlags: u32,
    pub guidItem: ::windows_sys::core::GUID,
    pub hBalloonIcon: super::WindowsAndMessaging::HICON,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub union NOTIFYICONDATAA_0 {
    pub uTimeout: u32,
    pub uVersion: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAA_0 {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct NOTIFYICONDATAA {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uID: u32,
    pub uFlags: NOTIFY_ICON_DATA_FLAGS,
    pub uCallbackMessage: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szTip: [super::super::Foundation::CHAR; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [super::super::Foundation::CHAR; 256],
    pub Anonymous: NOTIFYICONDATAA_0,
    pub szInfoTitle: [super::super::Foundation::CHAR; 64],
    pub dwInfoFlags: u32,
    pub guidItem: ::windows_sys::core::GUID,
    pub hBalloonIcon: super::WindowsAndMessaging::HICON,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAA {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub union NOTIFYICONDATAA_0 {
    pub uTimeout: u32,
    pub uVersion: u32,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAA_0 {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uID: u32,
    pub uFlags: NOTIFY_ICON_DATA_FLAGS,
    pub uCallbackMessage: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub Anonymous: NOTIFYICONDATAW_0,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: ::windows_sys::core::GUID,
    pub hBalloonIcon: super::WindowsAndMessaging::HICON,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub union NOTIFYICONDATAW_0 {
    pub uTimeout: u32,
    pub uVersion: u32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAW_0 {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAW_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uID: u32,
    pub uFlags: NOTIFY_ICON_DATA_FLAGS,
    pub uCallbackMessage: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub Anonymous: NOTIFYICONDATAW_0,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: ::windows_sys::core::GUID,
    pub hBalloonIcon: super::WindowsAndMessaging::HICON,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAW {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub union NOTIFYICONDATAW_0 {
    pub uTimeout: u32,
    pub uVersion: u32,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for NOTIFYICONDATAW_0 {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for NOTIFYICONDATAW_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct NOTIFYICONIDENTIFIER {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uID: u32,
    pub guidItem: ::windows_sys::core::GUID,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NOTIFYICONIDENTIFIER {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NOTIFYICONIDENTIFIER {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct NOTIFYICONIDENTIFIER {
    pub cbSize: u32,
    pub hWnd: super::super::Foundation::HWND,
    pub uID: u32,
    pub guidItem: ::windows_sys::core::GUID,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NOTIFYICONIDENTIFIER {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NOTIFYICONIDENTIFIER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NOTIFYICON_VERSION: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NOTIFYICON_VERSION_4: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NOTIFY_ICON_DATA_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_MESSAGE: NOTIFY_ICON_DATA_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_ICON: NOTIFY_ICON_DATA_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_TIP: NOTIFY_ICON_DATA_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_STATE: NOTIFY_ICON_DATA_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_INFO: NOTIFY_ICON_DATA_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_GUID: NOTIFY_ICON_DATA_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_REALTIME: NOTIFY_ICON_DATA_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIF_SHOWTIP: NOTIFY_ICON_DATA_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NOTIFY_ICON_MESSAGE = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIM_ADD: NOTIFY_ICON_MESSAGE = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIM_MODIFY: NOTIFY_ICON_MESSAGE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIM_DELETE: NOTIFY_ICON_MESSAGE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIM_SETFOCUS: NOTIFY_ICON_MESSAGE = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NIM_SETVERSION: NOTIFY_ICON_MESSAGE = 4u32;
pub const NPCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1037483712, data2: 33171, data3: 20478, data4: [174, 37, 224, 142, 57, 234, 64, 99] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_NetworkManagement_WNet\"`*"]
#[cfg(feature = "Win32_NetworkManagement_WNet")]
pub struct NRESARRAY {
    pub cItems: u32,
    pub nr: [super::super::NetworkManagement::WNet::NETRESOURCEA; 1],
}
#[cfg(feature = "Win32_NetworkManagement_WNet")]
impl ::core::marker::Copy for NRESARRAY {}
#[cfg(feature = "Win32_NetworkManagement_WNet")]
impl ::core::clone::Clone for NRESARRAY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Controls\"`*"]
#[cfg(feature = "Win32_UI_Controls")]
pub struct NSTCCUSTOMDRAW {
    pub psi: IShellItem,
    pub uItemState: u32,
    pub nstcis: u32,
    pub pszText: ::windows_sys::core::PCWSTR,
    pub iImage: i32,
    pub himl: super::Controls::HIMAGELIST,
    pub iLevel: i32,
    pub iIndent: i32,
}
#[cfg(feature = "Win32_UI_Controls")]
impl ::core::marker::Copy for NSTCCUSTOMDRAW {}
#[cfg(feature = "Win32_UI_Controls")]
impl ::core::clone::Clone for NSTCCUSTOMDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCDHPOS_ONTOP: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NSTCFOLDERCAPABILITIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCFC_NONE: NSTCFOLDERCAPABILITIES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCFC_PINNEDITEMFILTERING: NSTCFOLDERCAPABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCFC_DELAY_REGISTER_NOTIFY: NSTCFOLDERCAPABILITIES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NSTCGNI = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_NEXT: NSTCGNI = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_NEXTVISIBLE: NSTCGNI = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_PREV: NSTCGNI = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_PREVVISIBLE: NSTCGNI = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_PARENT: NSTCGNI = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_CHILD: NSTCGNI = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_FIRSTVISIBLE: NSTCGNI = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCGNI_LASTVISIBLE: NSTCGNI = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NSTCSTYLE2 = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS2_DEFAULT: NSTCSTYLE2 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS2_INTERRUPTNOTIFICATIONS: NSTCSTYLE2 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS2_SHOWNULLSPACEMENU: NSTCSTYLE2 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS2_DISPLAYPADDING: NSTCSTYLE2 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS2_DISPLAYPINNEDONLY: NSTCSTYLE2 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NTSCS2_NOSINGLETONAUTOEXPAND: NSTCSTYLE2 = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NTSCS2_NEVERINSERTNONENUMERATED: NSTCSTYLE2 = 32i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Console\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Console"))]
pub struct NT_CONSOLE_PROPS {
    pub dbh: DATABLOCK_HEADER,
    pub wFillAttribute: u16,
    pub wPopupFillAttribute: u16,
    pub dwScreenBufferSize: super::super::System::Console::COORD,
    pub dwWindowSize: super::super::System::Console::COORD,
    pub dwWindowOrigin: super::super::System::Console::COORD,
    pub nFont: u32,
    pub nInputBufferSize: u32,
    pub dwFontSize: super::super::System::Console::COORD,
    pub uFontFamily: u32,
    pub uFontWeight: u32,
    pub FaceName: [u16; 32],
    pub uCursorSize: u32,
    pub bFullScreen: super::super::Foundation::BOOL,
    pub bQuickEdit: super::super::Foundation::BOOL,
    pub bInsertMode: super::super::Foundation::BOOL,
    pub bAutoPosition: super::super::Foundation::BOOL,
    pub uHistoryBufferSize: u32,
    pub uNumberOfHistoryBuffers: u32,
    pub bHistoryNoDup: super::super::Foundation::BOOL,
    pub ColorTable: [u32; 16],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Console"))]
impl ::core::marker::Copy for NT_CONSOLE_PROPS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Console"))]
impl ::core::clone::Clone for NT_CONSOLE_PROPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NT_CONSOLE_PROPS_SIG: u32 = 2684354562u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct NT_FE_CONSOLE_PROPS {
    pub dbh: DATABLOCK_HEADER,
    pub uCodePage: u32,
}
impl ::core::marker::Copy for NT_FE_CONSOLE_PROPS {}
impl ::core::clone::Clone for NT_FE_CONSOLE_PROPS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NT_FE_CONSOLE_PROPS_SIG: u32 = 2684354564u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NUM_POINTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NWMF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_UNLOADING: NWMF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_USERINITED: NWMF = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_FIRST: NWMF = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_OVERRIDEKEY: NWMF = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_SHOWHELP: NWMF = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_HTMLDIALOG: NWMF = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_FROMDIALOGCHILD: NWMF = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_USERREQUESTED: NWMF = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_USERALLOWED: NWMF = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_FORCEWINDOW: NWMF = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_FORCETAB: NWMF = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_SUGGESTWINDOW: NWMF = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_SUGGESTTAB: NWMF = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NWMF_INACTIVETAB: NWMF = 1048576i32;
pub const NamespaceTreeControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2919580178, data2: 13621, data3: 17456, data4: [131, 237, 213, 1, 170, 102, 128, 230] };
pub const NamespaceWalker: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1928028640, data2: 34418, data3: 17155, data4: [145, 117, 242, 228, 198, 139, 46, 124] };
pub const NetworkConnections: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1879551175, data2: 12802, data3: 4561, data4: [170, 210, 0, 128, 95, 193, 39, 14] };
pub const NetworkExplorerFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4029422093, data2: 48673, data3: 17232, data4: [136, 176, 115, 103, 252, 150, 239, 60] };
pub const NetworkPlaces: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 546122848, data2: 15082, data3: 4201, data4: [162, 215, 8, 0, 43, 48, 48, 157] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type NewProcessCauseConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ProtectedModeRedirect: NewProcessCauseConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFASI_EDIT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFASI_OPENDESKTOP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFFLINE_STATUS_INCOMPLETE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFFLINE_STATUS_LOCAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFFLINE_STATUS_REMOTE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OF_CAP_CANCLOSE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OF_CAP_CANSWITCHTO: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OI_ASYNC: u32 = 4294962926u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OI_DEFAULT: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct OPENASINFO {
    pub pcszFile: ::windows_sys::core::PCWSTR,
    pub pcszClass: ::windows_sys::core::PCWSTR,
    pub oaifInFlags: OPEN_AS_INFO_FLAGS,
}
impl ::core::marker::Copy for OPENASINFO {}
impl ::core::clone::Clone for OPENASINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPENPROPS_INHIBITPIF: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPENPROPS_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type OPEN_AS_INFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_ALLOW_REGISTRATION: OPEN_AS_INFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_REGISTER_EXT: OPEN_AS_INFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_EXEC: OPEN_AS_INFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_FORCE_REGISTRATION: OPEN_AS_INFO_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_HIDE_REGISTRATION: OPEN_AS_INFO_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_URL_PROTOCOL: OPEN_AS_INFO_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OAIF_FILE_IS_URI: OPEN_AS_INFO_FLAGS = 128u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct OPEN_PRINTER_PROPS_INFOA {
    pub dwSize: u32,
    pub pszSheetName: ::windows_sys::core::PSTR,
    pub uSheetIndex: u32,
    pub dwFlags: u32,
    pub bModal: super::super::Foundation::BOOL,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for OPEN_PRINTER_PROPS_INFOA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for OPEN_PRINTER_PROPS_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct OPEN_PRINTER_PROPS_INFOA {
    pub dwSize: u32,
    pub pszSheetName: ::windows_sys::core::PSTR,
    pub uSheetIndex: u32,
    pub dwFlags: u32,
    pub bModal: super::super::Foundation::BOOL,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for OPEN_PRINTER_PROPS_INFOA {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for OPEN_PRINTER_PROPS_INFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct OPEN_PRINTER_PROPS_INFOW {
    pub dwSize: u32,
    pub pszSheetName: ::windows_sys::core::PWSTR,
    pub uSheetIndex: u32,
    pub dwFlags: u32,
    pub bModal: super::super::Foundation::BOOL,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for OPEN_PRINTER_PROPS_INFOW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for OPEN_PRINTER_PROPS_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct OPEN_PRINTER_PROPS_INFOW {
    pub dwSize: u32,
    pub pszSheetName: ::windows_sys::core::PWSTR,
    pub uSheetIndex: u32,
    pub dwFlags: u32,
    pub bModal: super::super::Foundation::BOOL,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for OPEN_PRINTER_PROPS_INFOW {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for OPEN_PRINTER_PROPS_INFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type OS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WINDOWS: OS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_NT: OS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN95ORGREATER: OS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_NT4ORGREATER: OS = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN98ORGREATER: OS = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN98_GOLD: OS = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN2000ORGREATER: OS = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN2000PRO: OS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN2000SERVER: OS = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN2000ADVSERVER: OS = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN2000DATACENTER: OS = 11u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN2000TERMINAL: OS = 12u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_EMBEDDED: OS = 13u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_TERMINALCLIENT: OS = 14u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_TERMINALREMOTEADMIN: OS = 15u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WIN95_GOLD: OS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_MEORGREATER: OS = 17u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_XPORGREATER: OS = 18u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_HOME: OS = 19u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_PROFESSIONAL: OS = 20u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_DATACENTER: OS = 21u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_ADVSERVER: OS = 22u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_SERVER: OS = 23u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_TERMINALSERVER: OS = 24u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_PERSONALTERMINALSERVER: OS = 25u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_FASTUSERSWITCHING: OS = 26u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WELCOMELOGONUI: OS = 27u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_DOMAINMEMBER: OS = 28u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_ANYSERVER: OS = 29u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WOW6432: OS = 30u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_WEBSERVER: OS = 31u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_SMALLBUSINESSSERVER: OS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_TABLETPC: OS = 33u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_SERVERADMINUI: OS = 34u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_MEDIACENTER: OS = 35u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OS_APPLIANCE: OS = 36u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type OfflineFolderStatus = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFS_INACTIVE: OfflineFolderStatus = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFS_ONLINE: OfflineFolderStatus = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFS_OFFLINE: OfflineFolderStatus = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFS_SERVERBACK: OfflineFolderStatus = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OFS_DIRTYCACHE: OfflineFolderStatus = 3i32;
pub const OnexCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 128583814, data2: 52365, data3: 19993, data4: [164, 16, 28, 117, 175, 104, 110, 98] };
pub const OnexPlapSmartcardCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 868773078, data2: 28767, data3: 19361, data4: [154, 219, 103, 7, 11, 131, 119, 117] };
pub const OpenControlPanel: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 107097477, data2: 26710, data3: 17504, data4: [141, 225, 168, 25, 33, 180, 28, 75] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PACKAGE_EXECUTION_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PES_UNKNOWN: PACKAGE_EXECUTION_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PES_RUNNING: PACKAGE_EXECUTION_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PES_SUSPENDING: PACKAGE_EXECUTION_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PES_SUSPENDED: PACKAGE_EXECUTION_STATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PES_TERMINATED: PACKAGE_EXECUTION_STATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_NAVIGATION: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_NONE: u32 = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_OFFLINE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_PRINTER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_PRIVACY: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_PROGRESS: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_SSL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PANE_ZONE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PAPPCONSTRAIN_CHANGE_ROUTINE = ::core::option::Option<unsafe extern "system" fn(constrained: super::super::Foundation::BOOLEAN, context: *const ::core::ffi::c_void)>;
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PAPPSTATE_CHANGE_ROUTINE = ::core::option::Option<unsafe extern "system" fn(quiesced: super::super::Foundation::BOOLEAN, context: *const ::core::ffi::c_void)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct PARSEDURLA {
    pub cbSize: u32,
    pub pszProtocol: ::windows_sys::core::PCSTR,
    pub cchProtocol: u32,
    pub pszSuffix: ::windows_sys::core::PCSTR,
    pub cchSuffix: u32,
    pub nScheme: u32,
}
impl ::core::marker::Copy for PARSEDURLA {}
impl ::core::clone::Clone for PARSEDURLA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct PARSEDURLW {
    pub cbSize: u32,
    pub pszProtocol: ::windows_sys::core::PCWSTR,
    pub cchProtocol: u32,
    pub pszSuffix: ::windows_sys::core::PCWSTR,
    pub cchSuffix: u32,
    pub nScheme: u32,
}
impl ::core::marker::Copy for PARSEDURLW {}
impl ::core::clone::Clone for PARSEDURLW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_MAX_CCH: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PATHCCH_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_NONE: PATHCCH_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_ALLOW_LONG_PATHS: PATHCCH_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_FORCE_ENABLE_LONG_NAME_PROCESS: PATHCCH_OPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_FORCE_DISABLE_LONG_NAME_PROCESS: PATHCCH_OPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_DO_NOT_NORMALIZE_SEGMENTS: PATHCCH_OPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_ENSURE_IS_EXTENDED_LENGTH_PATH: PATHCCH_OPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_ENSURE_TRAILING_SLASH: PATHCCH_OPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PATHCCH_CANONICALIZE_SLASHES: PATHCCH_OPTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PCS_RET = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PCS_FATAL: PCS_RET = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PCS_REPLACEDCHAR: PCS_RET = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PCS_REMOVEDCHAR: PCS_RET = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PCS_TRUNCATED: PCS_RET = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PCS_PATHTOOLONG: PCS_RET = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDTIMER_PAUSE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDTIMER_RESET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDTIMER_RESUME: u32 = 3u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(feature = "Win32_UI_Shell_Common")]
pub struct PERSIST_FOLDER_TARGET_INFO {
    pub pidlTargetFolder: *mut Common::ITEMIDLIST,
    pub szTargetParsingName: [u16; 260],
    pub szNetworkProvider: [u16; 260],
    pub dwAttributes: u32,
    pub csidl: i32,
}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::marker::Copy for PERSIST_FOLDER_TARGET_INFO {}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::clone::Clone for PERSIST_FOLDER_TARGET_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PFNCANSHAREFOLDERW = ::core::option::Option<unsafe extern "system" fn(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNSHOWSHAREFOLDERUIW = ::core::option::Option<unsafe extern "system" fn(hwndparent: super::super::Foundation::HWND, pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_AVG_DATA_RATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_CHANNEL_COUNT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_COMPRESSION: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_FORMAT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_SAMPLE_RATE: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_SAMPLE_SIZE: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_STREAM_NAME: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_STREAM_NUMBER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDASI_TIMELENGTH: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDDRSI_DESCRIPTION: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDDRSI_PLAYCOUNT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDDRSI_PLAYEXPIRES: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDDRSI_PLAYSTARTS: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDDRSI_PROTECTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PIDISF_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISF_RECENTLYCHANGED: PIDISF_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISF_CACHEDSTICKY: PIDISF_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISF_CACHEIMAGES: PIDISF_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISF_FOLLOWALLLINKS: PIDISF_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PIDISM_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISM_GLOBAL: PIDISM_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISM_WATCH: PIDISM_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISM_DONTWATCH: PIDISM_OPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PIDISR_INFO = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISR_UP_TO_DATE: PIDISR_INFO = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISR_NEEDS_ADD: PIDISR_INFO = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISR_NEEDS_UPDATE: PIDISR_INFO = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDISR_NEEDS_DELETE: PIDISR_INFO = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_ALBUM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_ARTIST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_COMMENT: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_GENRE: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_LYRICS: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_SONGTITLE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_TRACK: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDSI_YEAR: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_COMPRESSION: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_DATA_RATE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_FRAME_COUNT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_FRAME_HEIGHT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_FRAME_RATE: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_FRAME_WIDTH: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_SAMPLE_SIZE: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_STREAM_NAME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_STREAM_NUMBER: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIDVSI_TIMELENGTH: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_COMPUTERNAME: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_CONTROLPANEL_CATEGORY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_DESCRIPTIONID: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_DISPLACED_DATE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_DISPLACED_FROM: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_DISPLAY_PROPERTIES: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_FINDDATA: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_HTMLINFOTIPFILE: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTROTEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PID_INTSITE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_WHATSNEW: PID_INTSITE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_AUTHOR: PID_INTSITE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_LASTVISIT: PID_INTSITE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_LASTMOD: PID_INTSITE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_VISITCOUNT: PID_INTSITE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_DESCRIPTION: PID_INTSITE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_COMMENT: PID_INTSITE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_FLAGS: PID_INTSITE = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_CONTENTLEN: PID_INTSITE = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_CONTENTCODE: PID_INTSITE = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_RECURSE: PID_INTSITE = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_WATCH: PID_INTSITE = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_SUBSCRIPTION: PID_INTSITE = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_URL: PID_INTSITE = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_TITLE: PID_INTSITE = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_CODEPAGE: PID_INTSITE = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_TRACKING: PID_INTSITE = 19i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_ICONINDEX: PID_INTSITE = 20i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_ICONFILE: PID_INTSITE = 21i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_INTSITE_ROAMED: PID_INTSITE = 34i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PID_IS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_URL: PID_IS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_NAME: PID_IS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_WORKINGDIR: PID_IS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_HOTKEY: PID_IS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_SHOWCMD: PID_IS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_ICONINDEX: PID_IS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_ICONFILE: PID_IS = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_WHATSNEW: PID_IS = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_AUTHOR: PID_IS = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_DESCRIPTION: PID_IS = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_COMMENT: PID_IS = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_IS_ROAMED: PID_IS = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_LINK_TARGET: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_LINK_TARGET_TYPE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_MISC_ACCESSCOUNT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_MISC_OWNER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_MISC_PICS: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_MISC_STATUS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_NETRESOURCE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_NETWORKLOCATION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_QUERY_RANK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_SHARE_CSC_STATUS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_SYNC_COPY_IN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_VOLUME_CAPACITY: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_VOLUME_FILESYSTEM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_VOLUME_FREE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PID_WHICHFOLDER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFDEFFILESIZE: u32 = 80u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFDEFPATHSIZE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFMAXFILEPATH: u32 = 260u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFNAMESIZE: u32 = 30u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFPARAMSSIZE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFSHDATASIZE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFSHPROGSIZE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PIFSTARTLOCSIZE: u32 = 63u32;
pub const PINLogonCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3414354450, data2: 40817, data3: 17517, data4: [137, 225, 141, 9, 36, 225, 37, 110] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PLATFORM_BROWSERONLY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PLATFORM_IE3: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PLATFORM_INTEGRATED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PLATFORM_UNKNOWN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PMSF_DONT_STRIP_SPACES: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PMSF_MULTIPLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PMSF_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PO_DELETE: u32 = 19u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PO_PORTCHANGE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PO_RENAME: u32 = 20u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PO_REN_PORT: u32 = 52u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PPCF_ADDARGUMENTS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PPCF_ADDQUOTES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PPCF_FORCEQUALIFY: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PPCF_LONGESTPOSSIBLE: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PPCF_NODIRECTORIES: u32 = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct PREVIEWHANDLERFRAMEINFO {
    pub haccel: super::WindowsAndMessaging::HACCEL,
    pub cAccelEntries: u32,
}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for PREVIEWHANDLERFRAMEINFO {}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for PREVIEWHANDLERFRAMEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PRF_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRF_VERIFYEXISTS: PRF_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRF_TRYPROGRAMEXTENSIONS: PRF_FLAGS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRF_FIRSTDIRDEF: PRF_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRF_DONTFINDLNK: PRF_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRF_REQUIREABSOLUTE: PRF_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_DOCUMENTDEFAULTS: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_NETINSTALL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_NETINSTALLLINK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_OPEN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_OPENNETPRN: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_PROPERTIES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_SERVERPROPERTIES: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINTACTION_TESTPAGE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PRINT_PROP_FORCE_NAME: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct PROFILEINFOA {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub lpUserName: ::windows_sys::core::PSTR,
    pub lpProfilePath: ::windows_sys::core::PSTR,
    pub lpDefaultPath: ::windows_sys::core::PSTR,
    pub lpServerName: ::windows_sys::core::PSTR,
    pub lpPolicyPath: ::windows_sys::core::PSTR,
    pub hProfile: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for PROFILEINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for PROFILEINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct PROFILEINFOW {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub lpUserName: ::windows_sys::core::PWSTR,
    pub lpProfilePath: ::windows_sys::core::PWSTR,
    pub lpDefaultPath: ::windows_sys::core::PWSTR,
    pub lpServerName: ::windows_sys::core::PWSTR,
    pub lpPolicyPath: ::windows_sys::core::PWSTR,
    pub hProfile: super::super::Foundation::HANDLE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for PROFILEINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for PROFILEINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_AUTOTIME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_MARQUEEPROGRESS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_MODAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_NOCANCEL: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_NOMINIMIZE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_NOPROGRESSBAR: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROGDLG_NOTIME: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROPSTR_EXTENSIONCOMPLETIONSTATE: &str = "ExtensionCompletionState";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PROP_CONTRACT_DELEGATE: &str = "ContractDelegate";
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct PUBAPPINFO {
    pub cbSize: u32,
    pub dwMask: u32,
    pub pszSource: ::windows_sys::core::PWSTR,
    pub stAssigned: super::super::Foundation::SYSTEMTIME,
    pub stPublished: super::super::Foundation::SYSTEMTIME,
    pub stScheduled: super::super::Foundation::SYSTEMTIME,
    pub stExpire: super::super::Foundation::SYSTEMTIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for PUBAPPINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for PUBAPPINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type PUBAPPINFOFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PAI_SOURCE: PUBAPPINFOFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PAI_ASSIGNEDTIME: PUBAPPINFOFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PAI_PUBLISHEDTIME: PUBAPPINFOFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PAI_SCHEDULEDTIME: PUBAPPINFOFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PAI_EXPIRETIME: PUBAPPINFOFLAGS = 16i32;
pub const PackageDebugSettings: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2981020015, data2: 9091, data3: 18514, data4: [176, 233, 143, 11, 29, 198, 107, 77] };
pub const PasswordCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1622642312, data2: 60120, data3: 17500, data4: [156, 253, 11, 135, 247, 78, 166, 205] };
pub const PreviousVersions: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1500164194, data2: 46290, data3: 16917, data4: [159, 116, 233, 16, 155, 10, 129, 83] };
pub const PropertiesUI: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3641899215, data2: 918, data3: 18709, data4: [136, 78, 251, 66, 93, 50, 148, 59] };
pub const PublishDropTarget: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3429822459, data2: 17398, data3: 18117, data4: [150, 25, 81, 213, 113, 150, 127, 125] };
pub const PublishingWizard: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1798510140, data2: 30373, data3: 19308, data4: [191, 33, 69, 222, 156, 213, 3, 161] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct QCMINFO {
    pub hmenu: super::WindowsAndMessaging::HMENU,
    pub indexMenu: u32,
    pub idCmdFirst: u32,
    pub idCmdLast: u32,
    pub pIdMap: *const QCMINFO_IDMAP,
}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for QCMINFO {}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for QCMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct QCMINFO_IDMAP {
    pub nMaxIds: u32,
    pub pIdList: [QCMINFO_IDMAP_PLACEMENT; 1],
}
impl ::core::marker::Copy for QCMINFO_IDMAP {}
impl ::core::clone::Clone for QCMINFO_IDMAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct QCMINFO_IDMAP_PLACEMENT {
    pub id: u32,
    pub fFlags: u32,
}
impl ::core::marker::Copy for QCMINFO_IDMAP_PLACEMENT {}
impl ::core::clone::Clone for QCMINFO_IDMAP_PLACEMENT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QCMINFO_PLACE_AFTER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QCMINFO_PLACE_BEFORE: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct QITAB {
    pub piid: *const ::windows_sys::core::GUID,
    pub dwOffset: u32,
}
impl ::core::marker::Copy for QITAB {}
impl ::core::clone::Clone for QITAB {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type QITIPF_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QITIPF_DEFAULT: QITIPF_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QITIPF_USENAME: QITIPF_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QITIPF_LINKNOTARGET: QITIPF_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QITIPF_LINKUSETARGET: QITIPF_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QITIPF_USESLOWTIP: QITIPF_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QITIPF_SINGLELINE: QITIPF_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QIF_CACHED: QITIPF_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QIF_DONTEXPANDFOLDER: QITIPF_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type QUERY_USER_NOTIFICATION_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_NOT_PRESENT: QUERY_USER_NOTIFICATION_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_BUSY: QUERY_USER_NOTIFICATION_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_RUNNING_D3D_FULL_SCREEN: QUERY_USER_NOTIFICATION_STATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_PRESENTATION_MODE: QUERY_USER_NOTIFICATION_STATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_ACCEPTS_NOTIFICATIONS: QUERY_USER_NOTIFICATION_STATE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_QUIET_TIME: QUERY_USER_NOTIFICATION_STATE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const QUNS_APP: QUERY_USER_NOTIFICATION_STATE = 7i32;
pub const QueryCancelAutoPlay: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 857675624, data2: 1449, data3: 19933, data4: [184, 110, 218, 227, 77, 220, 153, 138] };
pub const RASProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1429725827, data2: 45543, data3: 20216, data4: [156, 110, 122, 176, 175, 229, 5, 109] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type RESTRICTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NONE: RESTRICTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NORUN: RESTRICTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCLOSE: RESTRICTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSAVESET: RESTRICTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOFILEMENU: RESTRICTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSETFOLDERS: RESTRICTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSETTASKBAR: RESTRICTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODESKTOP: RESTRICTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOFIND: RESTRICTIONS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODRIVES: RESTRICTIONS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODRIVEAUTORUN: RESTRICTIONS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODRIVETYPEAUTORUN: RESTRICTIONS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NONETHOOD: RESTRICTIONS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_STARTBANNER: RESTRICTIONS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_RESTRICTRUN: RESTRICTIONS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOPRINTERTABS: RESTRICTIONS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOPRINTERDELETE: RESTRICTIONS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOPRINTERADD: RESTRICTIONS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSTARTMENUSUBFOLDERS: RESTRICTIONS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_MYDOCSONNET: RESTRICTIONS = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOEXITTODOS: RESTRICTIONS = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ENFORCESHELLEXTSECURITY: RESTRICTIONS = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_LINKRESOLVEIGNORELINKINFO: RESTRICTIONS = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCOMMONGROUPS: RESTRICTIONS = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_SEPARATEDESKTOPPROCESS: RESTRICTIONS = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOWEB: RESTRICTIONS = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOTRAYCONTEXTMENU: RESTRICTIONS = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOVIEWCONTEXTMENU: RESTRICTIONS = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NONETCONNECTDISCONNECT: RESTRICTIONS = 134217728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_STARTMENULOGOFF: RESTRICTIONS = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSETTINGSASSIST: RESTRICTIONS = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOINTERNETICON: RESTRICTIONS = 1073741825i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NORECENTDOCSHISTORY: RESTRICTIONS = 1073741826i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NORECENTDOCSMENU: RESTRICTIONS = 1073741827i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOACTIVEDESKTOP: RESTRICTIONS = 1073741828i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOACTIVEDESKTOPCHANGES: RESTRICTIONS = 1073741829i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOFAVORITESMENU: RESTRICTIONS = 1073741830i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_CLEARRECENTDOCSONEXIT: RESTRICTIONS = 1073741831i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_CLASSICSHELL: RESTRICTIONS = 1073741832i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCUSTOMIZEWEBVIEW: RESTRICTIONS = 1073741833i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOHTMLWALLPAPER: RESTRICTIONS = 1073741840i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCHANGINGWALLPAPER: RESTRICTIONS = 1073741841i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODESKCOMP: RESTRICTIONS = 1073741842i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOADDDESKCOMP: RESTRICTIONS = 1073741843i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODELDESKCOMP: RESTRICTIONS = 1073741844i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCLOSEDESKCOMP: RESTRICTIONS = 1073741845i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCLOSE_DRAGDROPBAND: RESTRICTIONS = 1073741846i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOMOVINGBAND: RESTRICTIONS = 1073741847i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOEDITDESKCOMP: RESTRICTIONS = 1073741848i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NORESOLVESEARCH: RESTRICTIONS = 1073741849i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NORESOLVETRACK: RESTRICTIONS = 1073741850i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_FORCECOPYACLWITHFILE: RESTRICTIONS = 1073741851i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOFORGETSOFTWAREUPDATE: RESTRICTIONS = 1073741853i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSETACTIVEDESKTOP: RESTRICTIONS = 1073741854i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOUPDATEWINDOWS: RESTRICTIONS = 1073741855i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCHANGESTARMENU: RESTRICTIONS = 1073741856i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOFOLDEROPTIONS: RESTRICTIONS = 1073741857i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_HASFINDCOMPUTERS: RESTRICTIONS = 1073741858i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_INTELLIMENUS: RESTRICTIONS = 1073741859i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_RUNDLGMEMCHECKBOX: RESTRICTIONS = 1073741860i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_ShowPostSetup: RESTRICTIONS = 1073741861i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCSC: RESTRICTIONS = 1073741862i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCONTROLPANEL: RESTRICTIONS = 1073741863i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ENUMWORKGROUP: RESTRICTIONS = 1073741864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_NOARP: RESTRICTIONS = 1073741865i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_NOREMOVEPAGE: RESTRICTIONS = 1073741866i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_NOADDPAGE: RESTRICTIONS = 1073741867i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_NOWINSETUPPAGE: RESTRICTIONS = 1073741868i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_GREYMSIADS: RESTRICTIONS = 1073741869i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCHANGEMAPPEDDRIVELABEL: RESTRICTIONS = 1073741870i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCHANGEMAPPEDDRIVECOMMENT: RESTRICTIONS = 1073741871i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_MaxRecentDocs: RESTRICTIONS = 1073741872i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NONETWORKCONNECTIONS: RESTRICTIONS = 1073741873i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_FORCESTARTMENULOGOFF: RESTRICTIONS = 1073741874i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOWEBVIEW: RESTRICTIONS = 1073741875i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCUSTOMIZETHISFOLDER: RESTRICTIONS = 1073741876i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOENCRYPTION: RESTRICTIONS = 1073741877i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_DONTSHOWSUPERHIDDEN: RESTRICTIONS = 1073741879i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSHELLSEARCHBUTTON: RESTRICTIONS = 1073741880i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOHARDWARETAB: RESTRICTIONS = 1073741881i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NORUNASINSTALLPROMPT: RESTRICTIONS = 1073741882i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_PROMPTRUNASINSTALLNETPATH: RESTRICTIONS = 1073741883i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOMANAGEMYCOMPUTERVERB: RESTRICTIONS = 1073741884i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_DISALLOWRUN: RESTRICTIONS = 1073741886i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOWELCOMESCREEN: RESTRICTIONS = 1073741887i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_RESTRICTCPL: RESTRICTIONS = 1073741888i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_DISALLOWCPL: RESTRICTIONS = 1073741889i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMBALLOONTIP: RESTRICTIONS = 1073741890i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMHELP: RESTRICTIONS = 1073741891i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOWINKEYS: RESTRICTIONS = 1073741892i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOENCRYPTONMOVE: RESTRICTIONS = 1073741893i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOLOCALMACHINERUN: RESTRICTIONS = 1073741894i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCURRENTUSERRUN: RESTRICTIONS = 1073741895i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOLOCALMACHINERUNONCE: RESTRICTIONS = 1073741896i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCURRENTUSERRUNONCE: RESTRICTIONS = 1073741897i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_FORCEACTIVEDESKTOPON: RESTRICTIONS = 1073741898i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOVIEWONDRIVE: RESTRICTIONS = 1073741900i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NONETCRAWL: RESTRICTIONS = 1073741901i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSHAREDDOCUMENTS: RESTRICTIONS = 1073741902i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMMYDOCS: RESTRICTIONS = 1073741903i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMMYPICS: RESTRICTIONS = 1073741904i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ALLOWBITBUCKDRIVES: RESTRICTIONS = 1073741905i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NONLEGACYSHELLMODE: RESTRICTIONS = 1073741906i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCONTROLPANELBARRICADE: RESTRICTIONS = 1073741907i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSTARTPAGE: RESTRICTIONS = 1073741908i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOAUTOTRAYNOTIFY: RESTRICTIONS = 1073741909i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOTASKGROUPING: RESTRICTIONS = 1073741910i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCDBURNING: RESTRICTIONS = 1073741911i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_MYCOMPNOPROP: RESTRICTIONS = 1073741912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_MYDOCSNOPROP: RESTRICTIONS = 1073741913i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSTARTPANEL: RESTRICTIONS = 1073741914i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISPLAYAPPEARANCEPAGE: RESTRICTIONS = 1073741915i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOTHEMESTAB: RESTRICTIONS = 1073741916i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOVISUALSTYLECHOICE: RESTRICTIONS = 1073741917i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSIZECHOICE: RESTRICTIONS = 1073741918i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOCOLORCHOICE: RESTRICTIONS = 1073741919i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_SETVISUALSTYLE: RESTRICTIONS = 1073741920i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_STARTRUNNOHOMEPATH: RESTRICTIONS = 1073741921i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOUSERNAMEINSTARTPANEL: RESTRICTIONS = 1073741922i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOMYCOMPUTERICON: RESTRICTIONS = 1073741923i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMNETWORKPLACES: RESTRICTIONS = 1073741924i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMPINNEDLIST: RESTRICTIONS = 1073741925i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMMYMUSIC: RESTRICTIONS = 1073741926i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMEJECTPC: RESTRICTIONS = 1073741927i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMMOREPROGRAMS: RESTRICTIONS = 1073741928i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMMFUPROGRAMS: RESTRICTIONS = 1073741929i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOTRAYITEMSDISPLAY: RESTRICTIONS = 1073741930i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOTOOLBARSONTASKBAR: RESTRICTIONS = 1073741931i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSMCONFIGUREPROGRAMS: RESTRICTIONS = 1073741935i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_HIDECLOCK: RESTRICTIONS = 1073741936i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOLOWDISKSPACECHECKS: RESTRICTIONS = 1073741937i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOENTIRENETWORK: RESTRICTIONS = 1073741938i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODESKTOPCLEANUP: RESTRICTIONS = 1073741939i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_BITBUCKNUKEONDELETE: RESTRICTIONS = 1073741940i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_BITBUCKCONFIRMDELETE: RESTRICTIONS = 1073741941i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_BITBUCKNOPROP: RESTRICTIONS = 1073741942i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISPBACKGROUND: RESTRICTIONS = 1073741943i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISPSCREENSAVEPG: RESTRICTIONS = 1073741944i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISPSETTINGSPG: RESTRICTIONS = 1073741945i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISPSCREENSAVEPREVIEW: RESTRICTIONS = 1073741946i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISPLAYCPL: RESTRICTIONS = 1073741947i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_HIDERUNASVERB: RESTRICTIONS = 1073741948i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOTHUMBNAILCACHE: RESTRICTIONS = 1073741949i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSTRCMPLOGICAL: RESTRICTIONS = 1073741950i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOPUBLISHWIZARD: RESTRICTIONS = 1073741951i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOONLINEPRINTSWIZARD: RESTRICTIONS = 1073741952i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOWEBSERVICES: RESTRICTIONS = 1073741953i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ALLOWUNHASHEDWEBVIEW: RESTRICTIONS = 1073741954i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ALLOWLEGACYWEBVIEW: RESTRICTIONS = 1073741955i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_REVERTWEBVIEWSECURITY: RESTRICTIONS = 1073741956i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_INHERITCONSOLEHANDLES: RESTRICTIONS = 1073741958i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOREMOTERECURSIVEEVENTS: RESTRICTIONS = 1073741961i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOREMOTECHANGENOTIFY: RESTRICTIONS = 1073741969i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOENUMENTIRENETWORK: RESTRICTIONS = 1073741971i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOINTERNETOPENWITH: RESTRICTIONS = 1073741973i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_DONTRETRYBADNETNAME: RESTRICTIONS = 1073741979i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ALLOWFILECLSIDJUNCTIONS: RESTRICTIONS = 1073741980i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOUPNPINSTALL: RESTRICTIONS = 1073741981i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_DONTGROUPPATCHES: RESTRICTIONS = 1073741996i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ARP_NOCHOOSEPROGRAMSPAGE: RESTRICTIONS = 1073741997i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NODISCONNECT: RESTRICTIONS = 1090519041i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOSECURITY: RESTRICTIONS = 1090519042i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_NOFILEASSOCIATE: RESTRICTIONS = 1090519043i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REST_ALLOWCOMMENTTOGGLE: RESTRICTIONS = 1090519044i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type RefreshConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REFRESH_NORMAL: RefreshConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REFRESH_IFEXPIRED: RefreshConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const REFRESH_COMPLETELY: RefreshConstants = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_ABSOLUTE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_ACTIVATE_NOFOCUS: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_ALLOW_AUTONAVIGATE: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_CALLERUNTRUSTED: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_CREATENOHISTORY: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_DEFBROWSER: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_DEFMODE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_EXPLOREMODE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_FEEDNAVIGATION: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_HELPMODE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_INITIATEDBYHLINKFRAME: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_KEEPSAMETEMPLATE: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_KEEPWORDWHEELTEXT: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_NAVIGATEBACK: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_NAVIGATEFORWARD: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_NEWBROWSER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_NOAUTOSELECT: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_NOTRANSFERHIST: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_OPENMODE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_PARENT: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_PLAYNOSOUND: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_REDIRECT: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_RELATIVE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_SAMEBROWSER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_TRUSTEDFORACTIVEX: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_TRUSTFIRSTDOWNLOAD: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_UNTRUSTEDFORDOWNLOAD: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSP_WRITENOHISTORY: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SCALE_CHANGE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCF_VALUE_NONE: SCALE_CHANGE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCF_SCALE: SCALE_CHANGE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCF_PHYSICAL: SCALE_CHANGE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_CREATE: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_DISPLAY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_DONOTUSE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_EDIT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_GLOBAL: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_LOCAL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_REFRESH: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCHEME_UPDATE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SCNRT_STATUS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCNRT_ENABLE: SCNRT_STATUS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCNRT_DISABLE: SCNRT_STATUS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SCRM_VERIFYPW: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SECURELOCKCODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_NOCHANGE: SECURELOCKCODE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_UNSECURE: SECURELOCKCODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_MIXED: SECURELOCKCODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_SECUREUNKNOWNBIT: SECURELOCKCODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_SECURE40BIT: SECURELOCKCODE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_SECURE56BIT: SECURELOCKCODE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_FORTEZZA: SECURELOCKCODE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SET_SECURE128BIT: SECURELOCKCODE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_FIRSTSUGGEST: SECURELOCKCODE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_UNSECURE: SECURELOCKCODE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_MIXED: SECURELOCKCODE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_SECUREUNKNOWNBIT: SECURELOCKCODE = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_SECURE40BIT: SECURELOCKCODE = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_SECURE56BIT: SECURELOCKCODE = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_FORTEZZA: SECURELOCKCODE = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SECURELOCK_SUGGEST_SECURE128BIT: SECURELOCKCODE = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_ASYNCOK: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_CLASSKEY: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_CLASSNAME: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_CONNECTNETDRV: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_DOENVSUBST: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_FLAG_DDEWAIT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_FLAG_HINST_IS_SITE: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_FLAG_LOG_USAGE: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_FLAG_NO_UI: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_HMONITOR: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_HOTKEY: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_ICON: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_IDLIST: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_INVOKEIDLIST: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_NOASYNC: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_NOCLOSEPROCESS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_NOQUERYCLASSSTORE: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_NOZONECHECKS: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_NO_CONSOLE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_UNICODE: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SEE_MASK_WAITFORINPUTIDLE: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SETPROPS_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_ACCESSDENIED: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_ASSOCINCOMPLETE: u32 = 27u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_DDEBUSY: u32 = 30u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_DDEFAIL: u32 = 29u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_DDETIMEOUT: u32 = 28u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_DLLNOTFOUND: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_FNF: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_NOASSOC: u32 = 31u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_OOM: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_PNF: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SE_ERR_SHARE: u32 = 26u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFBID_PIDLCHANGED: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SFBS_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFBS_FLAGS_ROUND_TO_NEAREST_DISPLAYED_DIGIT: SFBS_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFBS_FLAGS_TRUNCATE_UNDISPLAYED_DECIMAL_DIGITS: SFBS_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_BROWSABLE: i32 = 134217728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CANCOPY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CANDELETE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CANLINK: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CANMONIKER: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CANMOVE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CANRENAME: i32 = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CAPABILITYMASK: i32 = 375i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_COMPRESSED: i32 = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_CONTENTSMASK: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_DISPLAYATTRMASK: i32 = 1032192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_DROPTARGET: i32 = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_ENCRYPTED: i32 = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_FILESYSANCESTOR: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_FILESYSTEM: i32 = 1073741824i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_FOLDER: i32 = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_GHOSTED: i32 = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_HASPROPSHEET: i32 = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_HASSTORAGE: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_HASSUBFOLDER: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_HIDDEN: i32 = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_ISSLOW: i32 = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_LINK: i32 = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_NEWCONTENT: i32 = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_NONENUMERATED: i32 = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_PKEYSFGAOMASK: i32 = -2130427904i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_PLACEHOLDER: i32 = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_READONLY: i32 = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_REMOVABLE: i32 = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_SHARE: i32 = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_STORAGE: i32 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_STORAGEANCESTOR: i32 = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_STORAGECAPMASK: i32 = 1891958792i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_STREAM: i32 = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_SYSTEM: i32 = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFGAO_VALIDATE: i32 = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_ADDOBJECT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETSELECTEDOBJECTS: u32 = 9u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SFVM_HELPTOPIC_DATA {
    pub wszHelpFile: [u16; 260],
    pub wszHelpTopic: [u16; 260],
}
impl ::core::marker::Copy for SFVM_HELPTOPIC_DATA {}
impl ::core::clone::Clone for SFVM_HELPTOPIC_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SFVM_MESSAGE_ID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_MERGEMENU: SFVM_MESSAGE_ID = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_INVOKECOMMAND: SFVM_MESSAGE_ID = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETHELPTEXT: SFVM_MESSAGE_ID = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETTOOLTIPTEXT: SFVM_MESSAGE_ID = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETBUTTONINFO: SFVM_MESSAGE_ID = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETBUTTONS: SFVM_MESSAGE_ID = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_INITMENUPOPUP: SFVM_MESSAGE_ID = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_FSNOTIFY: SFVM_MESSAGE_ID = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_WINDOWCREATED: SFVM_MESSAGE_ID = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETDETAILSOF: SFVM_MESSAGE_ID = 23i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_COLUMNCLICK: SFVM_MESSAGE_ID = 24i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_QUERYFSNOTIFY: SFVM_MESSAGE_ID = 25i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_DEFITEMCOUNT: SFVM_MESSAGE_ID = 26i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_DEFVIEWMODE: SFVM_MESSAGE_ID = 27i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_UNMERGEMENU: SFVM_MESSAGE_ID = 28i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_UPDATESTATUSBAR: SFVM_MESSAGE_ID = 31i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_BACKGROUNDENUM: SFVM_MESSAGE_ID = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_DIDDRAGDROP: SFVM_MESSAGE_ID = 36i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_SETISFV: SFVM_MESSAGE_ID = 39i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_THISIDLIST: SFVM_MESSAGE_ID = 41i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_ADDPROPERTYPAGES: SFVM_MESSAGE_ID = 47i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_BACKGROUNDENUMDONE: SFVM_MESSAGE_ID = 48i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETNOTIFY: SFVM_MESSAGE_ID = 49i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETSORTDEFAULTS: SFVM_MESSAGE_ID = 53i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_SIZE: SFVM_MESSAGE_ID = 57i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETZONE: SFVM_MESSAGE_ID = 58i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETPANE: SFVM_MESSAGE_ID = 59i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETHELPTOPIC: SFVM_MESSAGE_ID = 63i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_GETANIMATION: SFVM_MESSAGE_ID = 68i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Controls\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
pub struct SFVM_PROPPAGE_DATA {
    pub dwReserved: u32,
    pub pfn: super::Controls::LPFNSVADDPROPSHEETPAGE,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
impl ::core::marker::Copy for SFVM_PROPPAGE_DATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Controls"))]
impl ::core::clone::Clone for SFVM_PROPPAGE_DATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_REARRANGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_REMOVEOBJECT: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_SETCLIPBOARD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_SETITEMPOS: u32 = 14u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_SETPOINTS: u32 = 23u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVM_UPDATEOBJECT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVSOC_INVALIDATE_ALL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVSOC_NOSCROLL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SFVS_SELECT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVS_SELECT_NONE: SFVS_SELECT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVS_SELECT_ALLITEMS: SFVS_SELECT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVS_SELECT_INVERT: SFVS_SELECT = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Ole\"`*"]
#[cfg(feature = "Win32_System_Ole")]
pub struct SFV_CREATE {
    pub cbSize: u32,
    pub pshf: IShellFolder,
    pub psvOuter: IShellView,
    pub psfvcb: IShellFolderViewCB,
}
#[cfg(feature = "Win32_System_Ole")]
impl ::core::marker::Copy for SFV_CREATE {}
#[cfg(feature = "Win32_System_Ole")]
impl ::core::clone::Clone for SFV_CREATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
pub struct SFV_SETITEMPOS {
    pub pidl: *mut Common::ITEMIDLIST,
    pub pt: super::super::Foundation::POINT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for SFV_SETITEMPOS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for SFV_SETITEMPOS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_AUTOAPPEND_FORCE_OFF: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_AUTOAPPEND_FORCE_ON: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_AUTOSUGGEST_FORCE_OFF: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_AUTOSUGGEST_FORCE_ON: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_FILESYSTEM: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_FILESYS_DIRS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_FILESYS_ONLY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_URLHISTORY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_URLMRU: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_USETAB: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHACF_VIRTUAL_NAMESPACE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHARD = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_PIDL: SHARD = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_PATHA: SHARD = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_PATHW: SHARD = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_APPIDINFO: SHARD = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_APPIDINFOIDLIST: SHARD = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_LINK: SHARD = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_APPIDINFOLINK: SHARD = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARD_SHELLITEM: SHARD = 8i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHARDAPPIDINFO {
    pub psi: IShellItem,
    pub pszAppID: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for SHARDAPPIDINFO {}
impl ::core::clone::Clone for SHARDAPPIDINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(feature = "Win32_UI_Shell_Common")]
pub struct SHARDAPPIDINFOIDLIST {
    pub pidl: *mut Common::ITEMIDLIST,
    pub pszAppID: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::marker::Copy for SHARDAPPIDINFOIDLIST {}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::clone::Clone for SHARDAPPIDINFOIDLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHARDAPPIDINFOLINK {
    pub psl: IShellLinkA,
    pub pszAppID: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for SHARDAPPIDINFOLINK {}
impl ::core::clone::Clone for SHARDAPPIDINFOLINK {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHARE_ROLE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_INVALID: SHARE_ROLE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_READER: SHARE_ROLE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_CONTRIBUTOR: SHARE_ROLE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_CO_OWNER: SHARE_ROLE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_OWNER: SHARE_ROLE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_CUSTOM: SHARE_ROLE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHARE_ROLE_MIXED: SHARE_ROLE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCDF_UPDATEITEM: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCIDS_ALLFIELDS: i32 = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCIDS_BITMASK: i32 = -65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCIDS_CANONICALONLY: i32 = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCIDS_COLUMNMASK: i32 = 65535i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNEE_MSI_CHANGE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNEE_MSI_UNINSTALL: i32 = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNEE_ORDERCHANGED: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHCNE_ID = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_RENAMEITEM: SHCNE_ID = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_CREATE: SHCNE_ID = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_DELETE: SHCNE_ID = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_MKDIR: SHCNE_ID = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_RMDIR: SHCNE_ID = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_MEDIAINSERTED: SHCNE_ID = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_MEDIAREMOVED: SHCNE_ID = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_DRIVEREMOVED: SHCNE_ID = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_DRIVEADD: SHCNE_ID = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_NETSHARE: SHCNE_ID = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_NETUNSHARE: SHCNE_ID = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_ATTRIBUTES: SHCNE_ID = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_UPDATEDIR: SHCNE_ID = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_UPDATEITEM: SHCNE_ID = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_SERVERDISCONNECT: SHCNE_ID = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_UPDATEIMAGE: SHCNE_ID = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_DRIVEADDGUI: SHCNE_ID = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_RENAMEFOLDER: SHCNE_ID = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_FREESPACE: SHCNE_ID = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_EXTENDED_EVENT: SHCNE_ID = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_ASSOCCHANGED: SHCNE_ID = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_DISKEVENTS: SHCNE_ID = 145439u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_GLOBALEVENTS: SHCNE_ID = 201687520u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_ALLEVENTS: SHCNE_ID = 2147483647u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNE_INTERRUPT: SHCNE_ID = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHCNF_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_IDLIST: SHCNF_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_PATHA: SHCNF_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_PRINTERA: SHCNF_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_DWORD: SHCNF_FLAGS = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_PATHW: SHCNF_FLAGS = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_PRINTERW: SHCNF_FLAGS = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_TYPE: SHCNF_FLAGS = 255u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_FLUSH: SHCNF_FLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_FLUSHNOWAIT: SHCNF_FLAGS = 12288u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_NOTIFYRECURSIVE: SHCNF_FLAGS = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_PATH: SHCNF_FLAGS = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNF_PRINTER: SHCNF_FLAGS = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHCNRF_SOURCE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNRF_InterruptLevel: SHCNRF_SOURCE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNRF_ShellLevel: SHCNRF_SOURCE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNRF_RecursiveInterrupt: SHCNRF_SOURCE = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCNRF_NewDelivery: SHCNRF_SOURCE = 32768i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHCOLUMNDATA {
    pub dwFlags: u32,
    pub dwFileAttributes: u32,
    pub dwReserved: u32,
    pub pwszExt: ::windows_sys::core::PWSTR,
    pub wszFile: [u16; 260],
}
impl ::core::marker::Copy for SHCOLUMNDATA {}
impl ::core::clone::Clone for SHCOLUMNDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub struct SHCOLUMNINFO {
    pub scid: PropertiesSystem::PROPERTYKEY,
    pub vt: u16,
    pub fmt: u32,
    pub cChars: u32,
    pub csFlags: u32,
    pub wszTitle: [u16; 80],
    pub wszDescription: [u16; 128],
}
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
impl ::core::marker::Copy for SHCOLUMNINFO {}
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
impl ::core::clone::Clone for SHCOLUMNINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHCOLUMNINIT {
    pub dwFlags: u32,
    pub dwReserved: u32,
    pub wszFolder: [u16; 260],
}
impl ::core::marker::Copy for SHCOLUMNINIT {}
impl ::core::clone::Clone for SHCOLUMNINIT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`, `\"Win32_System_Threading\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
pub struct SHCREATEPROCESSINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub pszFile: ::windows_sys::core::PCWSTR,
    pub pszParameters: ::windows_sys::core::PCWSTR,
    pub pszCurrentDirectory: ::windows_sys::core::PCWSTR,
    pub hUserToken: super::super::Foundation::HANDLE,
    pub lpProcessAttributes: *mut super::super::Security::SECURITY_ATTRIBUTES,
    pub lpThreadAttributes: *mut super::super::Security::SECURITY_ATTRIBUTES,
    pub bInheritHandles: super::super::Foundation::BOOL,
    pub dwCreationFlags: u32,
    pub lpStartupInfo: *mut super::super::System::Threading::STARTUPINFOW,
    pub lpProcessInformation: *mut super::super::System::Threading::PROCESS_INFORMATION,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
impl ::core::marker::Copy for SHCREATEPROCESSINFOW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
impl ::core::clone::Clone for SHCREATEPROCESSINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Security\"`, `\"Win32_System_Threading\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
pub struct SHCREATEPROCESSINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub pszFile: ::windows_sys::core::PCWSTR,
    pub pszParameters: ::windows_sys::core::PCWSTR,
    pub pszCurrentDirectory: ::windows_sys::core::PCWSTR,
    pub hUserToken: super::super::Foundation::HANDLE,
    pub lpProcessAttributes: *mut super::super::Security::SECURITY_ATTRIBUTES,
    pub lpThreadAttributes: *mut super::super::Security::SECURITY_ATTRIBUTES,
    pub bInheritHandles: super::super::Foundation::BOOL,
    pub dwCreationFlags: u32,
    pub lpStartupInfo: *mut super::super::System::Threading::STARTUPINFOW,
    pub lpProcessInformation: *mut super::super::System::Threading::PROCESS_INFORMATION,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
impl ::core::marker::Copy for SHCREATEPROCESSINFOW {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Security", feature = "Win32_System_Threading"))]
impl ::core::clone::Clone for SHCREATEPROCESSINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHC_E_SHELL_COMPONENT_STARTUP_FAILURE: ::windows_sys::core::HRESULT = -2144927180i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHChangeDWORDAsIDList {
    pub cb: u16,
    pub dwItem1: u32,
    pub dwItem2: u32,
    pub cbZero: u16,
}
impl ::core::marker::Copy for SHChangeDWORDAsIDList {}
impl ::core::clone::Clone for SHChangeDWORDAsIDList {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
pub struct SHChangeNotifyEntry {
    pub pidl: *mut Common::ITEMIDLIST,
    pub fRecursive: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::marker::Copy for SHChangeNotifyEntry {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common"))]
impl ::core::clone::Clone for SHChangeNotifyEntry {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHChangeProductKeyAsIDList {
    pub cb: u16,
    pub wszProductKey: [u16; 39],
    pub cbZero: u16,
}
impl ::core::marker::Copy for SHChangeProductKeyAsIDList {}
impl ::core::clone::Clone for SHChangeProductKeyAsIDList {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHChangeUpdateImageIDList {
    pub cb: u16,
    pub iIconIndex: i32,
    pub iCurIndex: i32,
    pub uFlags: u32,
    pub dwProcessID: u32,
    pub szName: [u16; 260],
    pub cbZero: u16,
}
impl ::core::marker::Copy for SHChangeUpdateImageIDList {}
impl ::core::clone::Clone for SHChangeUpdateImageIDList {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHDESCRIPTIONID {
    pub dwDescriptionId: SHDID_ID,
    pub clsid: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for SHDESCRIPTIONID {}
impl ::core::clone::Clone for SHDESCRIPTIONID {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHDID_ID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_ROOT_REGITEM: SHDID_ID = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_FS_FILE: SHDID_ID = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_FS_DIRECTORY: SHDID_ID = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_FS_OTHER: SHDID_ID = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_DRIVE35: SHDID_ID = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_DRIVE525: SHDID_ID = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_REMOVABLE: SHDID_ID = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_FIXED: SHDID_ID = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_NETDRIVE: SHDID_ID = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_CDROM: SHDID_ID = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_RAMDISK: SHDID_ID = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_OTHER: SHDID_ID = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_NET_DOMAIN: SHDID_ID = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_NET_SERVER: SHDID_ID = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_NET_SHARE: SHDID_ID = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_NET_RESTOFNET: SHDID_ID = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_NET_OTHER: SHDID_ID = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_IMAGING: SHDID_ID = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_AUDIO: SHDID_ID = 19i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_COMPUTER_SHAREDDOCS: SHDID_ID = 20i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_MOBILE_DEVICE: SHDID_ID = 21i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHDID_REMOTE_DESKTOP_DRIVE: SHDID_ID = 22i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct SHDRAGIMAGE {
    pub sizeDragImage: super::super::Foundation::SIZE,
    pub ptOffset: super::super::Foundation::POINT,
    pub hbmpDragImage: super::super::Graphics::Gdi::HBITMAP,
    pub crColorKey: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for SHDRAGIMAGE {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for SHDRAGIMAGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHELLBROWSERSHOWCONTROL = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSC_HIDE: SHELLBROWSERSHOWCONTROL = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSC_SHOW: SHELLBROWSERSHOWCONTROL = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSC_TOGGLE: SHELLBROWSERSHOWCONTROL = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SBSC_QUERY: SHELLBROWSERSHOWCONTROL = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub struct SHELLEXECUTEINFOA {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerb: ::windows_sys::core::PCSTR,
    pub lpFile: ::windows_sys::core::PCSTR,
    pub lpParameters: ::windows_sys::core::PCSTR,
    pub lpDirectory: ::windows_sys::core::PCSTR,
    pub nShow: i32,
    pub hInstApp: super::super::Foundation::HINSTANCE,
    pub lpIDList: *mut ::core::ffi::c_void,
    pub lpClass: ::windows_sys::core::PCSTR,
    pub hkeyClass: super::super::System::Registry::HKEY,
    pub dwHotKey: u32,
    pub Anonymous: SHELLEXECUTEINFOA_0,
    pub hProcess: super::super::Foundation::HANDLE,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub union SHELLEXECUTEINFOA_0 {
    pub hIcon: super::super::Foundation::HANDLE,
    pub hMonitor: super::super::Foundation::HANDLE,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOA_0 {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub struct SHELLEXECUTEINFOA {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerb: ::windows_sys::core::PCSTR,
    pub lpFile: ::windows_sys::core::PCSTR,
    pub lpParameters: ::windows_sys::core::PCSTR,
    pub lpDirectory: ::windows_sys::core::PCSTR,
    pub nShow: i32,
    pub hInstApp: super::super::Foundation::HINSTANCE,
    pub lpIDList: *mut ::core::ffi::c_void,
    pub lpClass: ::windows_sys::core::PCSTR,
    pub hkeyClass: super::super::System::Registry::HKEY,
    pub dwHotKey: u32,
    pub Anonymous: SHELLEXECUTEINFOA_0,
    pub hProcess: super::super::Foundation::HANDLE,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOA {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub union SHELLEXECUTEINFOA_0 {
    pub hIcon: super::super::Foundation::HANDLE,
    pub hMonitor: super::super::Foundation::HANDLE,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOA_0 {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerb: ::windows_sys::core::PCWSTR,
    pub lpFile: ::windows_sys::core::PCWSTR,
    pub lpParameters: ::windows_sys::core::PCWSTR,
    pub lpDirectory: ::windows_sys::core::PCWSTR,
    pub nShow: i32,
    pub hInstApp: super::super::Foundation::HINSTANCE,
    pub lpIDList: *mut ::core::ffi::c_void,
    pub lpClass: ::windows_sys::core::PCWSTR,
    pub hkeyClass: super::super::System::Registry::HKEY,
    pub dwHotKey: u32,
    pub Anonymous: SHELLEXECUTEINFOW_0,
    pub hProcess: super::super::Foundation::HANDLE,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub union SHELLEXECUTEINFOW_0 {
    pub hIcon: super::super::Foundation::HANDLE,
    pub hMonitor: super::super::Foundation::HANDLE,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOW_0 {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOW_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub struct SHELLEXECUTEINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub lpVerb: ::windows_sys::core::PCWSTR,
    pub lpFile: ::windows_sys::core::PCWSTR,
    pub lpParameters: ::windows_sys::core::PCWSTR,
    pub lpDirectory: ::windows_sys::core::PCWSTR,
    pub nShow: i32,
    pub hInstApp: super::super::Foundation::HINSTANCE,
    pub lpIDList: *mut ::core::ffi::c_void,
    pub lpClass: ::windows_sys::core::PCWSTR,
    pub hkeyClass: super::super::System::Registry::HKEY,
    pub dwHotKey: u32,
    pub Anonymous: SHELLEXECUTEINFOW_0,
    pub hProcess: super::super::Foundation::HANDLE,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOW {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Registry\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
pub union SHELLEXECUTEINFOW_0 {
    pub hIcon: super::super::Foundation::HANDLE,
    pub hMonitor: super::super::Foundation::HANDLE,
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::marker::Copy for SHELLEXECUTEINFOW_0 {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Registry"))]
impl ::core::clone::Clone for SHELLEXECUTEINFOW_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHELLFLAGSTATE {
    pub _bitfield: i32,
}
impl ::core::marker::Copy for SHELLFLAGSTATE {}
impl ::core::clone::Clone for SHELLFLAGSTATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHELLSTATEA {
    pub _bitfield1: i32,
    pub dwWin95Unused: u32,
    pub uWin95Unused: u32,
    pub lParamSort: i32,
    pub iSortDirection: i32,
    pub version: u32,
    pub uNotUsed: u32,
    pub _bitfield2: i32,
}
impl ::core::marker::Copy for SHELLSTATEA {}
impl ::core::clone::Clone for SHELLSTATEA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHELLSTATEVERSION_IE4: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHELLSTATEVERSION_WIN2K: u32 = 10u32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHELLSTATEW {
    pub _bitfield1: i32,
    pub dwWin95Unused: u32,
    pub uWin95Unused: u32,
    pub lParamSort: i32,
    pub iSortDirection: i32,
    pub version: u32,
    pub uNotUsed: u32,
    pub _bitfield2: i32,
}
impl ::core::marker::Copy for SHELLSTATEW {}
impl ::core::clone::Clone for SHELLSTATEW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHELL_E_WRONG_BITDEPTH: ::windows_sys::core::HRESULT = -2144927486i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHELL_ITEM_RESOURCE {
    pub guidType: ::windows_sys::core::GUID,
    pub szName: [u16; 260],
}
impl ::core::marker::Copy for SHELL_ITEM_RESOURCE {}
impl ::core::clone::Clone for SHELL_ITEM_RESOURCE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHELL_LINK_DATA_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_DEFAULT: SHELL_LINK_DATA_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_ID_LIST: SHELL_LINK_DATA_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_LINK_INFO: SHELL_LINK_DATA_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_NAME: SHELL_LINK_DATA_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_RELPATH: SHELL_LINK_DATA_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_WORKINGDIR: SHELL_LINK_DATA_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_ARGS: SHELL_LINK_DATA_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_ICONLOCATION: SHELL_LINK_DATA_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_UNICODE: SHELL_LINK_DATA_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_FORCE_NO_LINKINFO: SHELL_LINK_DATA_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_EXP_SZ: SHELL_LINK_DATA_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_RUN_IN_SEPARATE: SHELL_LINK_DATA_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_DARWINID: SHELL_LINK_DATA_FLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_RUNAS_USER: SHELL_LINK_DATA_FLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_HAS_EXP_ICON_SZ: SHELL_LINK_DATA_FLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_NO_PIDL_ALIAS: SHELL_LINK_DATA_FLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_FORCE_UNCNAME: SHELL_LINK_DATA_FLAGS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_RUN_WITH_SHIMLAYER: SHELL_LINK_DATA_FLAGS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_FORCE_NO_LINKTRACK: SHELL_LINK_DATA_FLAGS = 262144i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_ENABLE_TARGET_METADATA: SHELL_LINK_DATA_FLAGS = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_DISABLE_LINK_PATH_TRACKING: SHELL_LINK_DATA_FLAGS = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_DISABLE_KNOWNFOLDER_RELATIVE_TRACKING: SHELL_LINK_DATA_FLAGS = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_NO_KF_ALIAS: SHELL_LINK_DATA_FLAGS = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_ALLOW_LINK_TO_LINK: SHELL_LINK_DATA_FLAGS = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_UNALIAS_ON_SAVE: SHELL_LINK_DATA_FLAGS = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_PREFER_ENVIRONMENT_PATH: SHELL_LINK_DATA_FLAGS = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_KEEP_LOCAL_IDLIST_FOR_UNC_TARGET: SHELL_LINK_DATA_FLAGS = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_PERSIST_VOLUME_ID_RELATIVE: SHELL_LINK_DATA_FLAGS = 134217728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_VALID: SHELL_LINK_DATA_FLAGS = 268433407i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLDF_RESERVED: SHELL_LINK_DATA_FLAGS = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHELL_UI_COMPONENT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHELL_UI_COMPONENT_TASKBARS: SHELL_UI_COMPONENT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHELL_UI_COMPONENT_NOTIFICATIONAREA: SHELL_UI_COMPONENT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHELL_UI_COMPONENT_DESKBAND: SHELL_UI_COMPONENT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHERB_NOCONFIRMATION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHERB_NOPROGRESSUI: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHERB_NOSOUND: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct SHFILEINFOA {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [super::super::Foundation::CHAR; 260],
    pub szTypeName: [super::super::Foundation::CHAR; 80],
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for SHFILEINFOA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for SHFILEINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct SHFILEINFOA {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [super::super::Foundation::CHAR; 260],
    pub szTypeName: [super::super::Foundation::CHAR; 80],
}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for SHFILEINFOA {}
#[cfg(target_arch = "x86")]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for SHFILEINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct SHFILEINFOW {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for SHFILEINFOW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for SHFILEINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct SHFILEINFOW {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub iIcon: i32,
    pub dwAttributes: u32,
    pub szDisplayName: [u16; 260],
    pub szTypeName: [u16; 80],
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for SHFILEINFOW {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for SHFILEINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct SHFILEOPSTRUCTA {
    pub hwnd: super::super::Foundation::HWND,
    pub wFunc: u32,
    pub pFrom: *mut i8,
    pub pTo: *mut i8,
    pub fFlags: u16,
    pub fAnyOperationsAborted: super::super::Foundation::BOOL,
    pub hNameMappings: *mut ::core::ffi::c_void,
    pub lpszProgressTitle: ::windows_sys::core::PCSTR,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SHFILEOPSTRUCTA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SHFILEOPSTRUCTA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct SHFILEOPSTRUCTA {
    pub hwnd: super::super::Foundation::HWND,
    pub wFunc: u32,
    pub pFrom: *mut i8,
    pub pTo: *mut i8,
    pub fFlags: u16,
    pub fAnyOperationsAborted: super::super::Foundation::BOOL,
    pub hNameMappings: *mut ::core::ffi::c_void,
    pub lpszProgressTitle: ::windows_sys::core::PCSTR,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SHFILEOPSTRUCTA {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SHFILEOPSTRUCTA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
pub struct SHFILEOPSTRUCTW {
    pub hwnd: super::super::Foundation::HWND,
    pub wFunc: u32,
    pub pFrom: ::windows_sys::core::PCWSTR,
    pub pTo: ::windows_sys::core::PCWSTR,
    pub fFlags: u16,
    pub fAnyOperationsAborted: super::super::Foundation::BOOL,
    pub hNameMappings: *mut ::core::ffi::c_void,
    pub lpszProgressTitle: ::windows_sys::core::PCWSTR,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SHFILEOPSTRUCTW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SHFILEOPSTRUCTW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
pub struct SHFILEOPSTRUCTW {
    pub hwnd: super::super::Foundation::HWND,
    pub wFunc: u32,
    pub pFrom: ::windows_sys::core::PCWSTR,
    pub pTo: ::windows_sys::core::PCWSTR,
    pub fFlags: u16,
    pub fAnyOperationsAborted: super::super::Foundation::BOOL,
    pub hNameMappings: *mut ::core::ffi::c_void,
    pub lpszProgressTitle: ::windows_sys::core::PCWSTR,
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SHFILEOPSTRUCTW {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SHFILEOPSTRUCTW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHFMT_ID = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_ID_DEFAULT: SHFMT_ID = 65535u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHFMT_OPT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_OPT_NONE: SHFMT_OPT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_OPT_FULL: SHFMT_OPT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_OPT_SYSONLY: SHFMT_OPT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHFMT_RET = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_ERROR: SHFMT_RET = 4294967295u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_CANCEL: SHFMT_RET = 4294967294u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHFMT_NOFORMAT: SHFMT_RET = 4294967293u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SHFOLDERCUSTOMSETTINGS {
    pub dwSize: u32,
    pub dwMask: u32,
    pub pvid: *mut ::windows_sys::core::GUID,
    pub pszWebViewTemplate: ::windows_sys::core::PWSTR,
    pub cchWebViewTemplate: u32,
    pub pszWebViewTemplateVersion: ::windows_sys::core::PWSTR,
    pub pszInfoTip: ::windows_sys::core::PWSTR,
    pub cchInfoTip: u32,
    pub pclsid: *mut ::windows_sys::core::GUID,
    pub dwFlags: u32,
    pub pszIconFile: ::windows_sys::core::PWSTR,
    pub cchIconFile: u32,
    pub iIconIndex: i32,
    pub pszLogo: ::windows_sys::core::PWSTR,
    pub cchLogo: u32,
}
impl ::core::marker::Copy for SHFOLDERCUSTOMSETTINGS {}
impl ::core::clone::Clone for SHFOLDERCUSTOMSETTINGS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHGDFIL_FORMAT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDFIL_FINDDATA: SHGDFIL_FORMAT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDFIL_NETRESOURCE: SHGDFIL_FORMAT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDFIL_DESCRIPTIONID: SHGDFIL_FORMAT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHGFI_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_ADDOVERLAYS: SHGFI_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_ATTR_SPECIFIED: SHGFI_FLAGS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_ATTRIBUTES: SHGFI_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_DISPLAYNAME: SHGFI_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_EXETYPE: SHGFI_FLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_ICON: SHGFI_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_ICONLOCATION: SHGFI_FLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_LARGEICON: SHGFI_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_LINKOVERLAY: SHGFI_FLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_OPENICON: SHGFI_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_OVERLAYINDEX: SHGFI_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_PIDL: SHGFI_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_SELECTED: SHGFI_FLAGS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_SHELLICONSIZE: SHGFI_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_SMALLICON: SHGFI_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_SYSICONINDEX: SHGFI_FLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_TYPENAME: SHGFI_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFI_USEFILEATTRIBUTES: SHGFI_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHGFP_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFP_TYPE_CURRENT: SHGFP_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGFP_TYPE_DEFAULT: SHGFP_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHGLOBALCOUNTER = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_SEARCHMANAGER: SHGLOBALCOUNTER = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_SEARCHOPTIONS: SHGLOBALCOUNTER = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_FOLDERSETTINGSCHANGE: SHGLOBALCOUNTER = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RATINGS: SHGLOBALCOUNTER = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_APPROVEDSITES: SHGLOBALCOUNTER = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RESTRICTIONS: SHGLOBALCOUNTER = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_SHELLSETTINGSCHANGED: SHGLOBALCOUNTER = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_SYSTEMPIDLCHANGE: SHGLOBALCOUNTER = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_OVERLAYMANAGER: SHGLOBALCOUNTER = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_QUERYASSOCIATIONS: SHGLOBALCOUNTER = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_IESESSIONS: SHGLOBALCOUNTER = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_IEONLY_SESSIONS: SHGLOBALCOUNTER = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_APPLICATION_DESTINATIONS: SHGLOBALCOUNTER = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const __UNUSED_RECYCLE_WAS_GLOBALCOUNTER_CSCSYNCINPROGRESS: SHGLOBALCOUNTER = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_BITBUCKETNUMDELETERS: SHGLOBALCOUNTER = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_SHARES: SHGLOBALCOUNTER = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_A: SHGLOBALCOUNTER = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_B: SHGLOBALCOUNTER = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_C: SHGLOBALCOUNTER = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_D: SHGLOBALCOUNTER = 19i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_E: SHGLOBALCOUNTER = 20i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_F: SHGLOBALCOUNTER = 21i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_G: SHGLOBALCOUNTER = 22i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_H: SHGLOBALCOUNTER = 23i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_I: SHGLOBALCOUNTER = 24i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_J: SHGLOBALCOUNTER = 25i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_K: SHGLOBALCOUNTER = 26i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_L: SHGLOBALCOUNTER = 27i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_M: SHGLOBALCOUNTER = 28i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_N: SHGLOBALCOUNTER = 29i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_O: SHGLOBALCOUNTER = 30i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_P: SHGLOBALCOUNTER = 31i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_Q: SHGLOBALCOUNTER = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_R: SHGLOBALCOUNTER = 33i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_S: SHGLOBALCOUNTER = 34i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_T: SHGLOBALCOUNTER = 35i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_U: SHGLOBALCOUNTER = 36i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_V: SHGLOBALCOUNTER = 37i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_W: SHGLOBALCOUNTER = 38i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_X: SHGLOBALCOUNTER = 39i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_Y: SHGLOBALCOUNTER = 40i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEDIRTYCOUNT_DRIVE_Z: SHGLOBALCOUNTER = 41i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const __UNUSED_RECYCLE_WAS_GLOBALCOUNTER_RECYCLEDIRTYCOUNT_SERVERDRIVE: SHGLOBALCOUNTER = 42i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const __UNUSED_RECYCLE_WAS_GLOBALCOUNTER_RECYCLEGLOBALDIRTYCOUNT: SHGLOBALCOUNTER = 43i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEBINENUM: SHGLOBALCOUNTER = 44i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RECYCLEBINCORRUPTED: SHGLOBALCOUNTER = 45i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_RATINGS_STATECOUNTER: SHGLOBALCOUNTER = 46i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_PRIVATE_PROFILE_CACHE: SHGLOBALCOUNTER = 47i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_INTERNETTOOLBAR_LAYOUT: SHGLOBALCOUNTER = 48i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_FOLDERDEFINITION_CACHE: SHGLOBALCOUNTER = 49i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_COMMONPLACES_LIST_CACHE: SHGLOBALCOUNTER = 50i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_PRIVATE_PROFILE_CACHE_MACHINEWIDE: SHGLOBALCOUNTER = 51i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_ASSOCCHANGED: SHGLOBALCOUNTER = 52i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_APP_ITEMS_STATE_STORE_CACHE: SHGLOBALCOUNTER = 53i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_SETTINGSYNC_ENABLED: SHGLOBALCOUNTER = 54i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_APPSFOLDER_FILETYPEASSOCIATION_COUNTER: SHGLOBALCOUNTER = 55i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_USERINFOCHANGED: SHGLOBALCOUNTER = 56i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_SYNC_ENGINE_INFORMATION_CACHE_MACHINEWIDE: SHGLOBALCOUNTER = 57i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_BANNERS_DATAMODEL_CACHE_MACHINEWIDE: SHGLOBALCOUNTER = 58i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const GLOBALCOUNTER_MAXIMUMVALUE: SHGLOBALCOUNTER = 59i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGNLI_NOLNK: u64 = 8u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGNLI_NOLOCNAME: u64 = 16u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGNLI_NOUNIQUE: u64 = 4u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGNLI_PIDL: u64 = 1u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGNLI_PREFIXNAME: u64 = 2u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGNLI_USEURLEXT: u64 = 32u64;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGSI_ICONLOCATION: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_ALLFOLDERS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_ALLUSERS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_INHERIT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_NOAUTODEFAULTS: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_PERFOLDER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_PERUSER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGVSPB_ROAM: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHHLNF_NOAUTOSELECT: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHHLNF_WRITENOHISTORY: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIL_EXTRALARGE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIL_JUMBO: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIL_LARGE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIL_LAST: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIL_SMALL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIL_SYSSMALL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMGDEC_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMGDEC_LOADFULL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMGDEC_THUMBNAIL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMGKEY_QUALITY: &str = "Compression";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMGKEY_RAWFORMAT: &str = "RawDataFormat";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMSTCAPFLAG_LOCKABLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHIMSTCAPFLAG_PURGEABLE: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct SHNAMEMAPPINGA {
    pub pszOldPath: ::windows_sys::core::PSTR,
    pub pszNewPath: ::windows_sys::core::PSTR,
    pub cchOldPath: i32,
    pub cchNewPath: i32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for SHNAMEMAPPINGA {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for SHNAMEMAPPINGA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
#[cfg(target_arch = "x86")]
pub struct SHNAMEMAPPINGA {
    pub pszOldPath: ::windows_sys::core::PSTR,
    pub pszNewPath: ::windows_sys::core::PSTR,
    pub cchOldPath: i32,
    pub cchNewPath: i32,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for SHNAMEMAPPINGA {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for SHNAMEMAPPINGA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct SHNAMEMAPPINGW {
    pub pszOldPath: ::windows_sys::core::PWSTR,
    pub pszNewPath: ::windows_sys::core::PWSTR,
    pub cchOldPath: i32,
    pub cchNewPath: i32,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for SHNAMEMAPPINGW {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for SHNAMEMAPPINGW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
#[cfg(target_arch = "x86")]
pub struct SHNAMEMAPPINGW {
    pub pszOldPath: ::windows_sys::core::PWSTR,
    pub pszNewPath: ::windows_sys::core::PWSTR,
    pub cchOldPath: i32,
    pub cchNewPath: i32,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for SHNAMEMAPPINGW {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for SHNAMEMAPPINGW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHOP_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHOP_PRINTERNAME: SHOP_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHOP_FILEPATH: SHOP_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHOP_VOLUMEGUID: SHOP_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPPFW_ASKDIRCREATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPPFW_DIRCREATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPPFW_IGNOREFILENAME: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPPFW_MEDIACHECKONLY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPPFW_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPPFW_NOWRITECHECK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPWHF_ANYLOCATION: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPWHF_NOFILESELECTOR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPWHF_NONETPLACECREATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPWHF_NORECOMPRESS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPWHF_USEMRU: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHPWHF_VALIDATEVIAWEBFOLDERS: u32 = 65536u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct SHQUERYRBINFO {
    pub cbSize: u32,
    pub i64Size: i64,
    pub i64NumItems: i64,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for SHQUERYRBINFO {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for SHQUERYRBINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
#[cfg(target_arch = "x86")]
pub struct SHQUERYRBINFO {
    pub cbSize: u32,
    pub i64Size: i64,
    pub i64NumItems: i64,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for SHQUERYRBINFO {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for SHQUERYRBINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHREGDEL_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGDEL_DEFAULT: SHREGDEL_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGDEL_HKCU: SHREGDEL_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGDEL_HKLM: SHREGDEL_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGDEL_BOTH: SHREGDEL_FLAGS = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHREGENUM_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGENUM_DEFAULT: SHREGENUM_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGENUM_HKCU: SHREGENUM_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGENUM_HKLM: SHREGENUM_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGENUM_BOTH: SHREGENUM_FLAGS = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGSET_FORCE_HKCU: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGSET_FORCE_HKLM: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGSET_HKCU: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHREGSET_HKLM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SHSTOCKICONID = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DOCNOASSOC: SHSTOCKICONID = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DOCASSOC: SHSTOCKICONID = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_APPLICATION: SHSTOCKICONID = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_FOLDER: SHSTOCKICONID = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_FOLDEROPEN: SHSTOCKICONID = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVE525: SHSTOCKICONID = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVE35: SHSTOCKICONID = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVEREMOVE: SHSTOCKICONID = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVEFIXED: SHSTOCKICONID = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVENET: SHSTOCKICONID = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVENETDISABLED: SHSTOCKICONID = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVECD: SHSTOCKICONID = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVERAM: SHSTOCKICONID = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_WORLD: SHSTOCKICONID = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SERVER: SHSTOCKICONID = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_PRINTER: SHSTOCKICONID = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MYNETWORK: SHSTOCKICONID = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_FIND: SHSTOCKICONID = 22i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_HELP: SHSTOCKICONID = 23i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SHARE: SHSTOCKICONID = 28i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_LINK: SHSTOCKICONID = 29i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SLOWFILE: SHSTOCKICONID = 30i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_RECYCLER: SHSTOCKICONID = 31i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_RECYCLERFULL: SHSTOCKICONID = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACDAUDIO: SHSTOCKICONID = 40i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_LOCK: SHSTOCKICONID = 47i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_AUTOLIST: SHSTOCKICONID = 49i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_PRINTERNET: SHSTOCKICONID = 50i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SERVERSHARE: SHSTOCKICONID = 51i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_PRINTERFAX: SHSTOCKICONID = 52i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_PRINTERFAXNET: SHSTOCKICONID = 53i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_PRINTERFILE: SHSTOCKICONID = 54i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_STACK: SHSTOCKICONID = 55i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIASVCD: SHSTOCKICONID = 56i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_STUFFEDFOLDER: SHSTOCKICONID = 57i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVEUNKNOWN: SHSTOCKICONID = 58i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVEDVD: SHSTOCKICONID = 59i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVD: SHSTOCKICONID = 60i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVDRAM: SHSTOCKICONID = 61i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVDRW: SHSTOCKICONID = 62i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVDR: SHSTOCKICONID = 63i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVDROM: SHSTOCKICONID = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACDAUDIOPLUS: SHSTOCKICONID = 65i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACDRW: SHSTOCKICONID = 66i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACDR: SHSTOCKICONID = 67i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACDBURN: SHSTOCKICONID = 68i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIABLANKCD: SHSTOCKICONID = 69i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACDROM: SHSTOCKICONID = 70i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_AUDIOFILES: SHSTOCKICONID = 71i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_IMAGEFILES: SHSTOCKICONID = 72i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_VIDEOFILES: SHSTOCKICONID = 73i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MIXEDFILES: SHSTOCKICONID = 74i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_FOLDERBACK: SHSTOCKICONID = 75i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_FOLDERFRONT: SHSTOCKICONID = 76i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SHIELD: SHSTOCKICONID = 77i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_WARNING: SHSTOCKICONID = 78i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_INFO: SHSTOCKICONID = 79i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_ERROR: SHSTOCKICONID = 80i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_KEY: SHSTOCKICONID = 81i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SOFTWARE: SHSTOCKICONID = 82i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_RENAME: SHSTOCKICONID = 83i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DELETE: SHSTOCKICONID = 84i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAAUDIODVD: SHSTOCKICONID = 85i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAMOVIEDVD: SHSTOCKICONID = 86i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAENHANCEDCD: SHSTOCKICONID = 87i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAENHANCEDDVD: SHSTOCKICONID = 88i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAHDDVD: SHSTOCKICONID = 89i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIABLURAY: SHSTOCKICONID = 90i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAVCD: SHSTOCKICONID = 91i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVDPLUSR: SHSTOCKICONID = 92i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIADVDPLUSRW: SHSTOCKICONID = 93i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DESKTOPPC: SHSTOCKICONID = 94i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MOBILEPC: SHSTOCKICONID = 95i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_USERS: SHSTOCKICONID = 96i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIASMARTMEDIA: SHSTOCKICONID = 97i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIACOMPACTFLASH: SHSTOCKICONID = 98i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DEVICECELLPHONE: SHSTOCKICONID = 99i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DEVICECAMERA: SHSTOCKICONID = 100i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DEVICEVIDEOCAMERA: SHSTOCKICONID = 101i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DEVICEAUDIOPLAYER: SHSTOCKICONID = 102i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_NETWORKCONNECT: SHSTOCKICONID = 103i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_INTERNET: SHSTOCKICONID = 104i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_ZIPFILE: SHSTOCKICONID = 105i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_SETTINGS: SHSTOCKICONID = 106i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVEHDDVD: SHSTOCKICONID = 132i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_DRIVEBD: SHSTOCKICONID = 133i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAHDDVDROM: SHSTOCKICONID = 134i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAHDDVDR: SHSTOCKICONID = 135i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIAHDDVDRAM: SHSTOCKICONID = 136i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIABDROM: SHSTOCKICONID = 137i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIABDR: SHSTOCKICONID = 138i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MEDIABDRE: SHSTOCKICONID = 139i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_CLUSTEREDDRIVE: SHSTOCKICONID = 140i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIID_MAX_ICONS: SHSTOCKICONID = 181i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct SHSTOCKICONINFO {
    pub cbSize: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub iSysImageIndex: i32,
    pub iIcon: i32,
    pub szPath: [u16; 260],
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for SHSTOCKICONINFO {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for SHSTOCKICONINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct SHSTOCKICONINFO {
    pub cbSize: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub iSysImageIndex: i32,
    pub iIcon: i32,
    pub szPath: [u16; 260],
}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for SHSTOCKICONINFO {}
#[cfg(target_arch = "x86")]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for SHSTOCKICONINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SIATTRIBFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIATTRIBFLAGS_AND: SIATTRIBFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIATTRIBFLAGS_OR: SIATTRIBFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIATTRIBFLAGS_APPCOMPAT: SIATTRIBFLAGS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIATTRIBFLAGS_MASK: SIATTRIBFLAGS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIATTRIBFLAGS_ALLITEMS: SIATTRIBFLAGS = 16384i32;
pub const SID_CommandsPropertyBag: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1845768784, data2: 17430, data3: 18524, data4: [177, 67, 230, 42, 118, 13, 159, 229] };
pub const SID_CtxQueryAssociations: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4205706304, data2: 46967, data3: 19305, data4: [170, 129, 119, 3, 94, 240, 230, 232] };
pub const SID_DefView: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1829961344, data2: 30993, data3: 4559, data4: [149, 52, 0, 0, 192, 91, 174, 11] };
pub const SID_LaunchSourceAppUserModelId: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 753369104, data2: 29915, data3: 18620, data4: [156, 106, 16, 243, 114, 73, 87, 35] };
pub const SID_LaunchSourceViewSizePreference: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2153796754, data2: 26585, data3: 16719, data4: [175, 137, 161, 205, 241, 36, 43, 193] };
pub const SID_LaunchTargetViewSizePreference: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 651895922, data2: 47031, data3: 16491, data4: [151, 2, 115, 10, 78, 32, 211, 191] };
pub const SID_MenuShellFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2797698740, data2: 11621, data3: 4562, data4: [131, 143, 0, 192, 79, 217, 24, 208] };
pub const SID_SCommDlgBrowser: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2163409459, data2: 47071, data3: 4562, data4: [163, 59, 0, 96, 151, 223, 91, 212] };
pub const SID_SCommandBarState: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3114183260, data2: 14416, data3: 17408, data4: [188, 51, 44, 229, 52, 4, 139, 248] };
pub const SID_SGetViewFromViewDual: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2291831645, data2: 38686, data3: 19218, data4: [185, 12, 36, 223, 201, 225, 229, 232] };
pub const SID_SInPlaceBrowser: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 489349163, data2: 13909, data3: 18124, data4: [182, 58, 40, 89, 136, 21, 59, 202] };
pub const SID_SMenuBandBKContextMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 374062470, data2: 7437, data3: 19936, data4: [154, 59, 217, 114, 150, 71, 194, 184] };
pub const SID_SMenuBandBottom: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1950131812, data2: 3563, data3: 4561, data4: [152, 37, 0, 192, 79, 217, 25, 114] };
pub const SID_SMenuBandBottomSelected: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 375306996, data2: 27985, data3: 4562, data4: [131, 173, 0, 192, 79, 217, 24, 208] };
pub const SID_SMenuBandChild: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3986472992, data2: 2233, data3: 4561, data4: [152, 35, 0, 192, 79, 217, 25, 114] };
pub const SID_SMenuBandContextMenuModifier: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 961828980, data2: 29026, data3: 18014, data4: [183, 131, 42, 161, 135, 79, 239, 129] };
pub const SID_SMenuBandParent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2351402732, data2: 16043, data3: 4561, data4: [140, 176, 0, 192, 79, 217, 24, 208] };
pub const SID_SMenuBandTop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2492704784, data2: 60472, data3: 4560, data4: [188, 70, 0, 170, 0, 108, 226, 245] };
pub const SID_SMenuPopup: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3521621995, data2: 27182, data3: 4560, data4: [140, 120, 0, 192, 79, 217, 24, 180] };
pub const SID_SSearchBoxInfo: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 338537057, data2: 20843, data3: 18195, data4: [180, 156, 251, 152, 94, 248, 41, 152] };
pub const SID_STopLevelBrowser: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1284947520, data2: 37212, data3: 4559, data4: [153, 211, 0, 170, 0, 74, 232, 55] };
pub const SID_STopWindow: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1239528704, data2: 17974, data3: 4563, data4: [151, 247, 0, 192, 79, 69, 208, 179] };
pub const SID_ShellExecuteNamedPropertyStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3951340962, data2: 255, data3: 18834, data4: [131, 36, 237, 92, 224, 97, 203, 41] };
pub const SID_URLExecutionContext: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4217343676, data2: 48054, data3: 19728, data4: [164, 97, 119, 114, 145, 160, 144, 48] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SIGDN = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_NORMALDISPLAY: SIGDN = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_PARENTRELATIVEPARSING: SIGDN = -2147385343i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_DESKTOPABSOLUTEPARSING: SIGDN = -2147319808i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_PARENTRELATIVEEDITING: SIGDN = -2147282943i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_DESKTOPABSOLUTEEDITING: SIGDN = -2147172352i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_FILESYSPATH: SIGDN = -2147123200i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_URL: SIGDN = -2147057664i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_PARENTRELATIVEFORADDRESSBAR: SIGDN = -2146975743i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_PARENTRELATIVE: SIGDN = -2146959359i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIGDN_PARENTRELATIVEFORUI: SIGDN = -2146877439i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SIIGBF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_RESIZETOFIT: SIIGBF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_BIGGERSIZEOK: SIIGBF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_MEMORYONLY: SIIGBF = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_ICONONLY: SIIGBF = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_THUMBNAILONLY: SIIGBF = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_INCACHEONLY: SIIGBF = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_CROPTOSQUARE: SIIGBF = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_WIDETHUMBNAILS: SIIGBF = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_ICONBACKGROUND: SIIGBF = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIIGBF_SCALEUP: SIIGBF = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIOM_ICONINDEX: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIOM_OVERLAYINDEX: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIOM_RESERVED_DEFAULT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIOM_RESERVED_LINK: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIOM_RESERVED_SHARED: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SIOM_RESERVED_SLOWFILE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SLGP_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLGP_SHORTPATH: SLGP_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLGP_UNCPRIORITY: SLGP_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLGP_RAWPATH: SLGP_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLGP_RELATIVEPRIORITY: SLGP_FLAGS = 8i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct SLOWAPPINFO {
    pub ullSize: u64,
    pub ftLastUsed: super::super::Foundation::FILETIME,
    pub iTimesUsed: i32,
    pub pszImage: ::windows_sys::core::PWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for SLOWAPPINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for SLOWAPPINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SLR_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NONE: SLR_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NO_UI: SLR_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_ANY_MATCH: SLR_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_UPDATE: SLR_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NOUPDATE: SLR_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NOSEARCH: SLR_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NOTRACK: SLR_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NOLINKINFO: SLR_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_INVOKE_MSI: SLR_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NO_UI_WITH_MSG_PUMP: SLR_FLAGS = 257i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_OFFER_DELETE_WITHOUT_FILE: SLR_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_KNOWNFOLDER: SLR_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_MACHINE_IN_LOCAL_TARGET: SLR_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_UPDATE_MACHINE_AND_SID: SLR_FLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SLR_NO_OBJECT_ID: SLR_FLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMAE_CONTRACTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMAE_EXPANDED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMAE_USER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMAE_VALID: u32 = 7u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(feature = "Win32_UI_Shell_Common")]
pub struct SMCSHCHANGENOTIFYSTRUCT {
    pub lEvent: i32,
    pub pidl1: *mut Common::ITEMIDLIST,
    pub pidl2: *mut Common::ITEMIDLIST,
}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::marker::Copy for SMCSHCHANGENOTIFYSTRUCT {}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::clone::Clone for SMCSHCHANGENOTIFYSTRUCT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_AUTOEXPANDCHANGE: u32 = 66u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_CHEVRONEXPAND: u32 = 25u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_CHEVRONGETTIP: u32 = 47u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_CREATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_DEFAULTICON: u32 = 22u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_DEMOTE: u32 = 17u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_DISPLAYCHEVRONTIP: u32 = 42u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_EXITMENU: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETAUTOEXPANDSTATE: u32 = 65u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETBKCONTEXTMENU: u32 = 68u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETCONTEXTMENUMODIFIER: u32 = 67u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETINFO: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETOBJECT: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETSFINFO: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_GETSFOBJECT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_INITMENU: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_NEWITEM: u32 = 23u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_OPEN: u32 = 69u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_PROMOTE: u32 = 18u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_REFRESH: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_SETSFOBJECT: u32 = 45u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_SFDDRESTRICTED: u32 = 48u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_SFEXEC: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_SFEXEC_MIDDLE: u32 = 49u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_SFSELECTITEM: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMC_SHCHANGENOTIFY: u32 = 46u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Shell_Common\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct SMDATA {
    pub dwMask: u32,
    pub dwFlags: u32,
    pub hmenu: super::WindowsAndMessaging::HMENU,
    pub hwnd: super::super::Foundation::HWND,
    pub uId: u32,
    pub uIdParent: u32,
    pub uIdAncestor: u32,
    pub punk: ::windows_sys::core::IUnknown,
    pub pidlFolder: *mut Common::ITEMIDLIST,
    pub pidlItem: *mut Common::ITEMIDLIST,
    pub psf: IShellFolder,
    pub pvUserData: *mut ::core::ffi::c_void,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for SMDATA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Shell_Common", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for SMDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMDM_HMENU: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMDM_SHELLFOLDER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMDM_TOOLBAR: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SMINFO {
    pub dwMask: u32,
    pub dwType: u32,
    pub dwFlags: u32,
    pub iIcon: i32,
}
impl ::core::marker::Copy for SMINFO {}
impl ::core::clone::Clone for SMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SMINFOFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_ICON: SMINFOFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_ACCELERATOR: SMINFOFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_DROPTARGET: SMINFOFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_SUBMENU: SMINFOFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_CHECKED: SMINFOFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_DROPCASCADE: SMINFOFLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_HIDDEN: SMINFOFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_DISABLED: SMINFOFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_TRACKPOPUP: SMINFOFLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_DEMOTED: SMINFOFLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_ALTSTATE: SMINFOFLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_DRAGNDROP: SMINFOFLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIF_NEW: SMINFOFLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SMINFOMASK = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIM_TYPE: SMINFOMASK = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIM_FLAGS: SMINFOMASK = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIM_ICON: SMINFOMASK = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SMINFOTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIT_SEPARATOR: SMINFOTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMIT_STRING: SMINFOTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_AUTOEXPAND: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_AUTOTOOLTIP: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_CACHED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_DROPONCONTAINER: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_HORIZONTAL: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_RESTRICT_DRAGDROP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_TOPLEVEL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINIT_VERTICAL: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINV_ID: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMINV_REFRESH: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMSET_BOTTOM: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMSET_DONTOWN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SMSET_TOP: u32 = 268435456u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_PropertiesSystem\"`*"]
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
pub struct SORTCOLUMN {
    pub propkey: PropertiesSystem::PROPERTYKEY,
    pub direction: SORTDIRECTION,
}
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
impl ::core::marker::Copy for SORTCOLUMN {}
#[cfg(feature = "Win32_UI_Shell_PropertiesSystem")]
impl ::core::clone::Clone for SORTCOLUMN {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SORTDIRECTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SORT_DESCENDING: SORTDIRECTION = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SORT_ASCENDING: SORTDIRECTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SORT_ORDER_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SOT_DEFAULT: SORT_ORDER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SOT_IGNORE_FOLDERNESS: SORT_ORDER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SPACTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_NONE: SPACTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_MOVING: SPACTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_COPYING: SPACTION = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_RECYCLING: SPACTION = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_APPLYINGATTRIBS: SPACTION = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_DOWNLOADING: SPACTION = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_SEARCHING_INTERNET: SPACTION = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_CALCULATING: SPACTION = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_UPLOADING: SPACTION = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_SEARCHING_FILES: SPACTION = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_DELETING: SPACTION = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_RENAMING: SPACTION = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_FORMATTING: SPACTION = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPACTION_COPY_MOVING: SPACTION = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_BROWSER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_DBMON: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_DEBUGBREAK: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_DEBUGOUT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_EVENT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_EVENTTRACE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_FLUSH: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_FORMATTEXT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_MEMWATCH: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_MSGTRACE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_MSVM: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_MULTISTOP: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_PERFTAGS: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_PROFILE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_SHELL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPMODE_TEST: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SPTEXT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPTEXT_ACTIONDESCRIPTION: SPTEXT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPTEXT_ACTIONDETAIL: SPTEXT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_NOEXPAND: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_NOVIRT: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RM_ANY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RM_NORMAL: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RM_SAFE: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RM_SAFENETWORK: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_ANY: u32 = 65535u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_BINARY: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_DWORD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_EXPAND_SZ: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_MULTI_SZ: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_NONE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_QWORD: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_RT_REG_SZ: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SRRF_ZEROONFAILURE: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SSF_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWALLOBJECTS: SSF_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWEXTENSIONS: SSF_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_HIDDENFILEEXTS: SSF_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SERVERADMINUI: SSF_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWCOMPCOLOR: SSF_MASK = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SORTCOLUMNS: SSF_MASK = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWSYSFILES: SSF_MASK = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_DOUBLECLICKINWEBVIEW: SSF_MASK = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWATTRIBCOL: SSF_MASK = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_DESKTOPHTML: SSF_MASK = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_WIN95CLASSIC: SSF_MASK = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_DONTPRETTYPATH: SSF_MASK = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWINFOTIP: SSF_MASK = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_MAPNETDRVBUTTON: SSF_MASK = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_NOCONFIRMRECYCLE: SSF_MASK = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_HIDEICONS: SSF_MASK = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_FILTER: SSF_MASK = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_WEBVIEW: SSF_MASK = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWSUPERHIDDEN: SSF_MASK = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SEPPROCESS: SSF_MASK = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_NONETCRAWLING: SSF_MASK = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_STARTPANELON: SSF_MASK = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWSTARTPAGE: SSF_MASK = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_AUTOCHECKSELECT: SSF_MASK = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_ICONSONLY: SSF_MASK = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWTYPEOVERLAY: SSF_MASK = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSF_SHOWSTATUSBAR: SSF_MASK = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSM_CLEAR: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSM_REFRESH: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSM_SET: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SSM_UPDATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type STGOP = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_MOVE: STGOP = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_COPY: STGOP = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_SYNC: STGOP = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_REMOVE: STGOP = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_RENAME: STGOP = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_APPLYPROPERTIES: STGOP = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STGOP_NEW: STGOP = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STIF_DEFAULT: i32 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STIF_SUPPORT_HEX: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type STORAGE_PROVIDER_FILE_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPFF_NONE: STORAGE_PROVIDER_FILE_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPFF_DOWNLOAD_BY_DEFAULT: STORAGE_PROVIDER_FILE_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPFF_CREATED_ON_THIS_DEVICE: STORAGE_PROVIDER_FILE_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STORE_E_NEWER_VERSION_AVAILABLE: ::windows_sys::core::HRESULT = -2144927484i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type STPFLAG = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STPF_NONE: STPFLAG = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STPF_USEAPPTHUMBNAILALWAYS: STPFLAG = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STPF_USEAPPTHUMBNAILWHENACTIVE: STPFLAG = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STPF_USEAPPPEEKALWAYS: STPFLAG = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STPF_USEAPPPEEKWHENACTIVE: STPFLAG = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_AVOID_DRIVE_RESTRICTION_POLICY: &str = "Avoid Drive Restriction Policy";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_BIND_DELEGATE_CREATE_OBJECT: &str = "Delegate Object Creation";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_BIND_FOLDERS_READ_ONLY: &str = "Folders As Read Only";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_BIND_FOLDER_ENUM_MODE: &str = "Folder Enum Mode";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_BIND_FORCE_FOLDER_SHORTCUT_RESOLVE: &str = "Force Folder Shortcut Resolve";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_DONT_PARSE_RELATIVE: &str = "Don\'t Parse Relative";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_DONT_RESOLVE_LINK: &str = "Don\'t Resolve Link";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_ENUM_ITEMS_FLAGS: &str = "SHCONTF";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_FILE_SYS_BIND_DATA: &str = "File System Bind Data";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_FILE_SYS_BIND_DATA_WIN7_FORMAT: &str = "Win7FileSystemIdList";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GET_ASYNC_HANDLER: &str = "GetAsyncHandler";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GPS_BESTEFFORT: &str = "GPS_BESTEFFORT";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GPS_DELAYCREATION: &str = "GPS_DELAYCREATION";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GPS_FASTPROPERTIESONLY: &str = "GPS_FASTPROPERTIESONLY";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GPS_HANDLERPROPERTIESONLY: &str = "GPS_HANDLERPROPERTIESONLY";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GPS_NO_OPLOCK: &str = "GPS_NO_OPLOCK";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_GPS_OPENSLOWITEM: &str = "GPS_OPENSLOWITEM";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_INTERNAL_NAVIGATE: &str = "Internal Navigation";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_INTERNETFOLDER_PARSE_ONLY_URLMON_BINDABLE: &str = "Validate URL";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_ITEM_CACHE_CONTEXT: &str = "ItemCacheContext";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_MYDOCS_CLSID: &str = "{450D8FBA-AD25-11D0-98A8-0800361B1103}";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_NO_VALIDATE_FILENAME_CHARS: &str = "NoValidateFilenameChars";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_ALLOW_INTERNET_SHELL_FOLDERS: &str = "Allow binding to Internet shell folder handlers and negate STR_PARSE_PREFER_WEB_BROWSING";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_AND_CREATE_ITEM: &str = "ParseAndCreateItem";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_DONT_REQUIRE_VALIDATED_URLS: &str = "Do not require validated URLs";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_EXPLICIT_ASSOCIATION_SUCCESSFUL: &str = "ExplicitAssociationSuccessful";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_PARTIAL_IDLIST: &str = "ParseOriginalItem";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_PREFER_FOLDER_BROWSING: &str = "Parse Prefer Folder Browsing";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_PREFER_WEB_BROWSING: &str = "Do not bind to Internet shell folder handlers";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_PROPERTYSTORE: &str = "DelegateNamedProperties";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_SHELL_PROTOCOL_TO_FILE_OBJECTS: &str = "Parse Shell Protocol To File Objects";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_SHOW_NET_DIAGNOSTICS_UI: &str = "Show network diagnostics UI";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_SKIP_NET_CACHE: &str = "Skip Net Resource Cache";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_TRANSLATE_ALIASES: &str = "Parse Translate Aliases";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_WITH_EXPLICIT_ASSOCAPP: &str = "ExplicitAssociationApp";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_WITH_EXPLICIT_PROGID: &str = "ExplicitProgid";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PARSE_WITH_PROPERTIES: &str = "ParseWithProperties";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_PROPERTYBAG_PARAM: &str = "SHBindCtxPropertyBag";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_REFERRER_IDENTIFIER: &str = "Referrer Identifier";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_SKIP_BINDING_CLSID: &str = "Skip Binding CLSID";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_STORAGEITEM_CREATION_FLAGS: &str = "SHGETSTORAGEITEM";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_TAB_REUSE_IDENTIFIER: &str = "Tab Reuse Identifier";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const STR_TRACK_CLSID: &str = "Track the CLSID";
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type SUBCLASSPROC = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM, uidsubclass: usize, dwrefdata: usize) -> super::super::Foundation::LRESULT>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole"))]
pub struct SV2CVW2_PARAMS {
    pub cbSize: u32,
    pub psvPrev: IShellView,
    pub pfs: *mut FOLDERSETTINGS,
    pub psbOwner: IShellBrowser,
    pub prcView: *mut super::super::Foundation::RECT,
    pub pvid: *const ::windows_sys::core::GUID,
    pub hwndView: super::super::Foundation::HWND,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for SV2CVW2_PARAMS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for SV2CVW2_PARAMS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SVUIA_STATUS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVUIA_DEACTIVATE: SVUIA_STATUS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVUIA_ACTIVATE_NOFOCUS: SVUIA_STATUS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVUIA_ACTIVATE_FOCUS: SVUIA_STATUS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVUIA_INPLACEACTIVATE: SVUIA_STATUS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRERRORFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRERRORFLAG_ENABLEJUMPTEXT: SYNCMGRERRORFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRFLAG = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_CONNECT: SYNCMGRFLAG = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_PENDINGDISCONNECT: SYNCMGRFLAG = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_MANUAL: SYNCMGRFLAG = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_IDLE: SYNCMGRFLAG = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_INVOKE: SYNCMGRFLAG = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_SCHEDULED: SYNCMGRFLAG = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_EVENTMASK: SYNCMGRFLAG = 255i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_SETTINGS: SYNCMGRFLAG = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRFLAG_MAYBOTHERUSER: SYNCMGRFLAG = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRHANDLERFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRHANDLER_HASPROPERTIES: SYNCMGRHANDLERFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRHANDLER_MAYESTABLISHCONNECTION: SYNCMGRHANDLERFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRHANDLER_ALWAYSLISTHANDLER: SYNCMGRHANDLERFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRHANDLER_HIDDEN: SYNCMGRHANDLERFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRHANDLERFLAG_MASK: u32 = 15u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct SYNCMGRHANDLERINFO {
    pub cbSize: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub SyncMgrHandlerFlags: u32,
    pub wszHandlerName: [u16; 32],
}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for SYNCMGRHANDLERINFO {}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for SYNCMGRHANDLERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRINVOKEFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRINVOKE_STARTSYNC: SYNCMGRINVOKEFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRINVOKE_MINIMIZED: SYNCMGRINVOKEFLAGS = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct SYNCMGRITEM {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub ItemID: ::windows_sys::core::GUID,
    pub dwItemState: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub wszItemName: [u16; 128],
    pub ftLastUpdate: super::super::Foundation::FILETIME,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for SYNCMGRITEM {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for SYNCMGRITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRITEMFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_HASPROPERTIES: SYNCMGRITEMFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_TEMPORARY: SYNCMGRITEMFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_ROAMINGUSER: SYNCMGRITEMFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_LASTUPDATETIME: SYNCMGRITEMFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_MAYDELETEITEM: SYNCMGRITEMFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_HIDDEN: SYNCMGRITEMFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRITEMSTATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEMSTATE_UNCHECKED: SYNCMGRITEMSTATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEMSTATE_CHECKED: SYNCMGRITEMSTATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRITEM_ITEMFLAGMASK: u32 = 127u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SYNCMGRLOGERRORINFO {
    pub cbSize: u32,
    pub mask: u32,
    pub dwSyncMgrErrorFlags: u32,
    pub ErrorID: ::windows_sys::core::GUID,
    pub ItemID: ::windows_sys::core::GUID,
}
impl ::core::marker::Copy for SYNCMGRLOGERRORINFO {}
impl ::core::clone::Clone for SYNCMGRLOGERRORINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGERROR_ERRORFLAGS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGERROR_ERRORID: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGERROR_ITEMID: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRLOGLEVEL = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGLEVEL_INFORMATION: SYNCMGRLOGLEVEL = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGLEVEL_WARNING: SYNCMGRLOGLEVEL = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGLEVEL_ERROR: SYNCMGRLOGLEVEL = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRLOGLEVEL_LOGLEVELMAX: SYNCMGRLOGLEVEL = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct SYNCMGRPROGRESSITEM {
    pub cbSize: u32,
    pub mask: u32,
    pub lpcStatusText: ::windows_sys::core::PCWSTR,
    pub dwStatusType: u32,
    pub iProgValue: i32,
    pub iMaxValue: i32,
}
impl ::core::marker::Copy for SYNCMGRPROGRESSITEM {}
impl ::core::clone::Clone for SYNCMGRPROGRESSITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRPROGRESSITEM_MAXVALUE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRPROGRESSITEM_PROGVALUE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRPROGRESSITEM_STATUSTEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRPROGRESSITEM_STATUSTYPE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRREGISTERFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRREGISTERFLAG_CONNECT: SYNCMGRREGISTERFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRREGISTERFLAG_PENDINGDISCONNECT: SYNCMGRREGISTERFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRREGISTERFLAG_IDLE: SYNCMGRREGISTERFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRREGISTERFLAGS_MASK: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGRSTATUS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_STOPPED: SYNCMGRSTATUS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_SKIPPED: SYNCMGRSTATUS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_PENDING: SYNCMGRSTATUS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_UPDATING: SYNCMGRSTATUS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_SUCCEEDED: SYNCMGRSTATUS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_FAILED: SYNCMGRSTATUS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_PAUSED: SYNCMGRSTATUS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_RESUMING: SYNCMGRSTATUS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_UPDATING_INDETERMINATE: SYNCMGRSTATUS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGRSTATUS_DELETED: SYNCMGRSTATUS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_CANCEL_REQUEST = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CR_NONE: SYNCMGR_CANCEL_REQUEST = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CR_CANCEL_ITEM: SYNCMGR_CANCEL_REQUEST = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CR_CANCEL_ALL: SYNCMGR_CANCEL_REQUEST = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CR_MAX: SYNCMGR_CANCEL_REQUEST = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_System_Com\"`*"]
#[cfg(feature = "Win32_System_Com")]
pub struct SYNCMGR_CONFLICT_ID_INFO {
    pub pblobID: *mut super::super::System::Com::BYTE_BLOB,
    pub pblobExtra: *mut super::super::System::Com::BYTE_BLOB,
}
#[cfg(feature = "Win32_System_Com")]
impl ::core::marker::Copy for SYNCMGR_CONFLICT_ID_INFO {}
#[cfg(feature = "Win32_System_Com")]
impl ::core::clone::Clone for SYNCMGR_CONFLICT_ID_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_CONFLICT_ITEM_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CIT_UPDATED: SYNCMGR_CONFLICT_ITEM_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CIT_DELETED: SYNCMGR_CONFLICT_ITEM_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_CONTROL_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CF_NONE: SYNCMGR_CONTROL_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CF_NOWAIT: SYNCMGR_CONTROL_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CF_WAIT: SYNCMGR_CONTROL_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CF_NOUI: SYNCMGR_CONTROL_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_CF_VALID: SYNCMGR_CONTROL_FLAGS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_EVENT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_EF_NONE: SYNCMGR_EVENT_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_EF_VALID: SYNCMGR_EVENT_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_EVENT_LEVEL = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_EL_INFORMATION: SYNCMGR_EVENT_LEVEL = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_EL_WARNING: SYNCMGR_EVENT_LEVEL = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_EL_ERROR: SYNCMGR_EVENT_LEVEL = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_EL_MAX: SYNCMGR_EVENT_LEVEL = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_HANDLER_CAPABILITIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_NONE: SYNCMGR_HANDLER_CAPABILITIES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_PROVIDES_ICON: SYNCMGR_HANDLER_CAPABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_EVENT_STORE: SYNCMGR_HANDLER_CAPABILITIES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_CONFLICT_STORE: SYNCMGR_HANDLER_CAPABILITIES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_SUPPORTS_CONCURRENT_SESSIONS: SYNCMGR_HANDLER_CAPABILITIES = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_CAN_BROWSE_CONTENT: SYNCMGR_HANDLER_CAPABILITIES = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_CAN_SHOW_SCHEDULE: SYNCMGR_HANDLER_CAPABILITIES = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_QUERY_BEFORE_ACTIVATE: SYNCMGR_HANDLER_CAPABILITIES = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_QUERY_BEFORE_DEACTIVATE: SYNCMGR_HANDLER_CAPABILITIES = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_QUERY_BEFORE_ENABLE: SYNCMGR_HANDLER_CAPABILITIES = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_QUERY_BEFORE_DISABLE: SYNCMGR_HANDLER_CAPABILITIES = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HCM_VALID_MASK: SYNCMGR_HANDLER_CAPABILITIES = 15925271i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_HANDLER_POLICIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_NONE: SYNCMGR_HANDLER_POLICIES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_PREVENT_ACTIVATE: SYNCMGR_HANDLER_POLICIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_PREVENT_DEACTIVATE: SYNCMGR_HANDLER_POLICIES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_PREVENT_ENABLE: SYNCMGR_HANDLER_POLICIES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_PREVENT_DISABLE: SYNCMGR_HANDLER_POLICIES = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_PREVENT_START_SYNC: SYNCMGR_HANDLER_POLICIES = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_PREVENT_STOP_SYNC: SYNCMGR_HANDLER_POLICIES = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_DISABLE_ENABLE: SYNCMGR_HANDLER_POLICIES = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_DISABLE_DISABLE: SYNCMGR_HANDLER_POLICIES = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_DISABLE_START_SYNC: SYNCMGR_HANDLER_POLICIES = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_DISABLE_STOP_SYNC: SYNCMGR_HANDLER_POLICIES = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_DISABLE_BROWSE: SYNCMGR_HANDLER_POLICIES = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_DISABLE_SCHEDULE: SYNCMGR_HANDLER_POLICIES = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_HIDDEN_BY_DEFAULT: SYNCMGR_HANDLER_POLICIES = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_BACKGROUND_SYNC_ONLY: SYNCMGR_HANDLER_POLICIES = 48i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HPM_VALID_MASK: SYNCMGR_HANDLER_POLICIES = 77631i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_HANDLER_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_UNSPECIFIED: SYNCMGR_HANDLER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_APPLICATION: SYNCMGR_HANDLER_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_DEVICE: SYNCMGR_HANDLER_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_FOLDER: SYNCMGR_HANDLER_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_SERVICE: SYNCMGR_HANDLER_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_COMPUTER: SYNCMGR_HANDLER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_MIN: SYNCMGR_HANDLER_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_HT_MAX: SYNCMGR_HANDLER_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_ITEM_CAPABILITIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_NONE: SYNCMGR_ITEM_CAPABILITIES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_PROVIDES_ICON: SYNCMGR_ITEM_CAPABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_EVENT_STORE: SYNCMGR_ITEM_CAPABILITIES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_CONFLICT_STORE: SYNCMGR_ITEM_CAPABILITIES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_CAN_DELETE: SYNCMGR_ITEM_CAPABILITIES = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_CAN_BROWSE_CONTENT: SYNCMGR_ITEM_CAPABILITIES = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_QUERY_BEFORE_ENABLE: SYNCMGR_ITEM_CAPABILITIES = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_QUERY_BEFORE_DISABLE: SYNCMGR_ITEM_CAPABILITIES = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_QUERY_BEFORE_DELETE: SYNCMGR_ITEM_CAPABILITIES = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_ICM_VALID_MASK: SYNCMGR_ITEM_CAPABILITIES = 7405591i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_ITEM_POLICIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_NONE: SYNCMGR_ITEM_POLICIES = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_PREVENT_ENABLE: SYNCMGR_ITEM_POLICIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_PREVENT_DISABLE: SYNCMGR_ITEM_POLICIES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_PREVENT_START_SYNC: SYNCMGR_ITEM_POLICIES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_PREVENT_STOP_SYNC: SYNCMGR_ITEM_POLICIES = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_DISABLE_ENABLE: SYNCMGR_ITEM_POLICIES = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_DISABLE_DISABLE: SYNCMGR_ITEM_POLICIES = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_DISABLE_START_SYNC: SYNCMGR_ITEM_POLICIES = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_DISABLE_STOP_SYNC: SYNCMGR_ITEM_POLICIES = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_DISABLE_BROWSE: SYNCMGR_ITEM_POLICIES = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_DISABLE_DELETE: SYNCMGR_ITEM_POLICIES = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_HIDDEN_BY_DEFAULT: SYNCMGR_ITEM_POLICIES = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_IPM_VALID_MASK: SYNCMGR_ITEM_POLICIES = 66303i32;
pub const SYNCMGR_OBJECTID_BrowseContent: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1472968068, data2: 59828, data3: 18350, data4: [161, 32, 196, 223, 51, 53, 222, 226] };
pub const SYNCMGR_OBJECTID_ConflictStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3615588852, data2: 9097, data3: 18404, data4: [169, 96, 96, 188, 194, 237, 147, 11] };
pub const SYNCMGR_OBJECTID_EventLinkClick: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 570670529, data2: 6897, data3: 16514, data4: [140, 48, 40, 57, 159, 65, 56, 76] };
pub const SYNCMGR_OBJECTID_EventStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1273967801, data2: 42886, data3: 16501, data4: [186, 136, 12, 43, 157, 137, 169, 143] };
pub const SYNCMGR_OBJECTID_Icon: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1841071555, data2: 23815, data3: 19570, data4: [167, 119, 127, 236, 120, 7, 44, 6] };
pub const SYNCMGR_OBJECTID_QueryBeforeActivate: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3632453643, data2: 59306, data3: 18925, data4: [134, 183, 230, 225, 247, 20, 205, 254] };
pub const SYNCMGR_OBJECTID_QueryBeforeDeactivate: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2700067458, data2: 24800, data3: 17934, data4: [147, 116, 234, 136, 81, 60, 252, 128] };
pub const SYNCMGR_OBJECTID_QueryBeforeDelete: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4151063447, data2: 44979, data3: 17879, data4: [165, 159, 90, 73, 233, 5, 67, 126] };
pub const SYNCMGR_OBJECTID_QueryBeforeDisable: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3143591082, data2: 61444, data3: 20149, data4: [142, 77, 38, 117, 25, 102, 52, 76] };
pub const SYNCMGR_OBJECTID_QueryBeforeEnable: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 80476144, data2: 23531, data3: 19937, data4: [188, 144, 144, 131, 69, 196, 128, 246] };
pub const SYNCMGR_OBJECTID_ShowSchedule: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3989238755, data2: 33857, data3: 16649, data4: [173, 243, 108, 28, 160, 183, 222, 71] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_PRESENTER_CHOICE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PC_NO_CHOICE: SYNCMGR_PRESENTER_CHOICE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PC_KEEP_ONE: SYNCMGR_PRESENTER_CHOICE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PC_KEEP_MULTIPLE: SYNCMGR_PRESENTER_CHOICE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PC_KEEP_RECENT: SYNCMGR_PRESENTER_CHOICE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PC_REMOVE_FROM_SYNC_SET: SYNCMGR_PRESENTER_CHOICE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PC_SKIP: SYNCMGR_PRESENTER_CHOICE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_PRESENTER_NEXT_STEP = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PNS_CONTINUE: SYNCMGR_PRESENTER_NEXT_STEP = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PNS_DEFAULT: SYNCMGR_PRESENTER_NEXT_STEP = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PNS_CANCEL: SYNCMGR_PRESENTER_NEXT_STEP = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_PROGRESS_STATUS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_UPDATING: SYNCMGR_PROGRESS_STATUS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_UPDATING_INDETERMINATE: SYNCMGR_PROGRESS_STATUS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_SUCCEEDED: SYNCMGR_PROGRESS_STATUS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_FAILED: SYNCMGR_PROGRESS_STATUS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_CANCELED: SYNCMGR_PROGRESS_STATUS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_DISCONNECTED: SYNCMGR_PROGRESS_STATUS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_PS_MAX: SYNCMGR_PROGRESS_STATUS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_RESOLUTION_ABILITIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RA_KEEPOTHER: SYNCMGR_RESOLUTION_ABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RA_KEEPRECENT: SYNCMGR_RESOLUTION_ABILITIES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RA_REMOVEFROMSYNCSET: SYNCMGR_RESOLUTION_ABILITIES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RA_KEEP_SINGLE: SYNCMGR_RESOLUTION_ABILITIES = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RA_KEEP_MULTIPLE: SYNCMGR_RESOLUTION_ABILITIES = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RA_VALID: SYNCMGR_RESOLUTION_ABILITIES = 31i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_RESOLUTION_FEEDBACK = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RF_CONTINUE: SYNCMGR_RESOLUTION_FEEDBACK = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RF_REFRESH: SYNCMGR_RESOLUTION_FEEDBACK = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_RF_CANCEL: SYNCMGR_RESOLUTION_FEEDBACK = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_SYNC_CONTROL_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_SCF_NONE: SYNCMGR_SYNC_CONTROL_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_SCF_IGNORE_IF_ALREADY_SYNCING: SYNCMGR_SYNC_CONTROL_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_SCF_VALID: SYNCMGR_SYNC_CONTROL_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SYNCMGR_UPDATE_REASON = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_UR_ADDED: SYNCMGR_UPDATE_REASON = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_UR_CHANGED: SYNCMGR_UPDATE_REASON = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_UR_REMOVED: SYNCMGR_UPDATE_REASON = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SYNCMGR_UR_MAX: SYNCMGR_UPDATE_REASON = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SZ_CONTENTTYPE_CDF: &str = "application/x-cdf";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SZ_CONTENTTYPE_CDFA: &str = "application/x-cdf";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SZ_CONTENTTYPE_CDFW: &str = "application/x-cdf";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SZ_CONTENTTYPE_HTML: &str = "text/html";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SZ_CONTENTTYPE_HTMLA: &str = "text/html";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SZ_CONTENTTYPE_HTMLW: &str = "text/html";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const S_SYNCMGR_CANCELALL: ::windows_sys::core::HRESULT = 262660i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const S_SYNCMGR_CANCELITEM: ::windows_sys::core::HRESULT = 262659i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const S_SYNCMGR_ENUMITEMS: ::windows_sys::core::HRESULT = 262673i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const S_SYNCMGR_ITEMDELETED: ::windows_sys::core::HRESULT = 262672i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const S_SYNCMGR_MISSINGITEMS: ::windows_sys::core::HRESULT = 262657i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const S_SYNCMGR_RETRYSYNC: ::windows_sys::core::HRESULT = 262658i32;
pub const ScheduledTasks: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3592911248, data2: 19562, data3: 4559, data4: [141, 135, 0, 170, 0, 96, 245, 191] };
pub const SearchFolderItemFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 335613442, data2: 48061, data3: 16880, data4: [136, 227, 237, 163, 113, 33, 101, 132] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type SecureLockIconConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconUnsecure: SecureLockIconConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconMixed: SecureLockIconConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconSecureUnknownBits: SecureLockIconConstants = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconSecure40Bit: SecureLockIconConstants = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconSecure56Bit: SecureLockIconConstants = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconSecureFortezza: SecureLockIconConstants = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const secureLockIconSecure128Bit: SecureLockIconConstants = 6i32;
pub const SelectedItemCount_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2414024402, data2: 3666, data3: 17930, data4: [156, 30, 72, 242, 115, 212, 112, 163] };
pub type ShFindChangeNotificationHandle = isize;
pub const SharedBitmap: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1303536758, data2: 26503, data3: 16454, data4: [184, 54, 232, 65, 42, 158, 138, 39] };
pub const SharingConfigurationManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1240691169, data2: 35932, data3: 19868, data4: [154, 59, 84, 166, 130, 127, 81, 60] };
pub const Shell: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 326145568, data2: 49785, data3: 4558, data4: [164, 158, 68, 69, 83, 84, 0, 0] };
pub const ShellBrowserWindow: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3230334352, data2: 62113, data3: 4561, data4: [132, 85, 0, 160, 201, 31, 56, 128] };
pub const ShellDesktop: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136192, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const ShellDispatchInproc: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 176793696, data2: 55217, data3: 4558, data4: [131, 80, 68, 69, 83, 84, 0, 0] };
pub const ShellFSFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4080421792, data2: 26041, data3: 4558, data4: [169, 186, 0, 170, 0, 74, 232, 55] };
pub const ShellFolderItem: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 803427050, data2: 64799, data3: 4562, data4: [177, 244, 0, 192, 79, 142, 235, 62] };
pub const ShellFolderView: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1645292193, data2: 60388, data3: 4559, data4: [165, 251, 0, 32, 175, 231, 41, 45] };
pub const ShellFolderViewOC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2610977137, data2: 63144, data3: 4559, data4: [164, 66, 0, 160, 201, 10, 143, 57] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ShellFolderViewOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_SHOWALLOBJECTS: ShellFolderViewOptions = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_SHOWEXTENSIONS: ShellFolderViewOptions = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_SHOWCOMPCOLOR: ShellFolderViewOptions = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_SHOWSYSFILES: ShellFolderViewOptions = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_WIN95CLASSIC: ShellFolderViewOptions = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_DOUBLECLICKINWEBVIEW: ShellFolderViewOptions = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SFVVO_DESKTOPHTML: ShellFolderViewOptions = 512i32;
pub const ShellImageDataFactory: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1726276859, data2: 62341, data3: 19920, data4: [141, 116, 162, 239, 209, 188, 97, 120] };
pub const ShellItem: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2596928481, data2: 57506, data3: 19158, data4: [180, 238, 226, 18, 1, 62, 169, 23] };
pub const ShellLibrary: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3652395293, data2: 58751, data3: 17446, data4: [170, 239, 48, 168, 6, 173, 211, 151] };
pub const ShellLink: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 136193, data2: 0, data3: 0, data4: [192, 0, 0, 0, 0, 0, 0, 70] };
pub const ShellLinkObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 287413280, data2: 5992, data3: 4561, data4: [149, 190, 0, 96, 151, 151, 234, 79] };
pub const ShellNameSpace: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1427335173, data2: 45790, data3: 4561, data4: [185, 242, 0, 160, 201, 139, 197, 71] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ShellSpecialFolderConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfDESKTOP: ShellSpecialFolderConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPROGRAMS: ShellSpecialFolderConstants = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCONTROLS: ShellSpecialFolderConstants = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPRINTERS: ShellSpecialFolderConstants = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPERSONAL: ShellSpecialFolderConstants = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfFAVORITES: ShellSpecialFolderConstants = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfSTARTUP: ShellSpecialFolderConstants = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfRECENT: ShellSpecialFolderConstants = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfSENDTO: ShellSpecialFolderConstants = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfBITBUCKET: ShellSpecialFolderConstants = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfSTARTMENU: ShellSpecialFolderConstants = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfDESKTOPDIRECTORY: ShellSpecialFolderConstants = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfDRIVES: ShellSpecialFolderConstants = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfNETWORK: ShellSpecialFolderConstants = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfNETHOOD: ShellSpecialFolderConstants = 19i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfFONTS: ShellSpecialFolderConstants = 20i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfTEMPLATES: ShellSpecialFolderConstants = 21i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONSTARTMENU: ShellSpecialFolderConstants = 22i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONPROGRAMS: ShellSpecialFolderConstants = 23i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONSTARTUP: ShellSpecialFolderConstants = 24i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONDESKTOPDIR: ShellSpecialFolderConstants = 25i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfAPPDATA: ShellSpecialFolderConstants = 26i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPRINTHOOD: ShellSpecialFolderConstants = 27i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfLOCALAPPDATA: ShellSpecialFolderConstants = 28i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfALTSTARTUP: ShellSpecialFolderConstants = 29i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONALTSTARTUP: ShellSpecialFolderConstants = 30i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONFAVORITES: ShellSpecialFolderConstants = 31i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfINTERNETCACHE: ShellSpecialFolderConstants = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOOKIES: ShellSpecialFolderConstants = 33i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfHISTORY: ShellSpecialFolderConstants = 34i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfCOMMONAPPDATA: ShellSpecialFolderConstants = 35i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfWINDOWS: ShellSpecialFolderConstants = 36i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfSYSTEM: ShellSpecialFolderConstants = 37i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPROGRAMFILES: ShellSpecialFolderConstants = 38i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfMYPICTURES: ShellSpecialFolderConstants = 39i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPROFILE: ShellSpecialFolderConstants = 40i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfSYSTEMx86: ShellSpecialFolderConstants = 41i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ssfPROGRAMFILESx86: ShellSpecialFolderConstants = 48i32;
pub const ShellUIHelper: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1688947639, data2: 4382, data3: 4561, data4: [143, 121, 0, 192, 79, 194, 251, 225] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ShellWindowFindWindowOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWFO_NEEDDISPATCH: ShellWindowFindWindowOptions = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWFO_INCLUDEPENDING: ShellWindowFindWindowOptions = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWFO_COOKIEPASSED: ShellWindowFindWindowOptions = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ShellWindowTypeConstants = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWC_EXPLORER: ShellWindowTypeConstants = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWC_BROWSER: ShellWindowTypeConstants = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWC_3RDPARTY: ShellWindowTypeConstants = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWC_CALLBACK: ShellWindowTypeConstants = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SWC_DESKTOP: ShellWindowTypeConstants = 8i32;
pub const ShellWindows: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2610977138, data2: 63144, data3: 4559, data4: [164, 66, 0, 160, 201, 10, 143, 57] };
pub const ShowInputPaneAnimationCoordinator: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 520383167, data2: 12802, data3: 19905, data4: [140, 181, 60, 103, 97, 124, 225, 250] };
pub const SimpleConflictPresenter: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2047830711, data2: 60804, data3: 18102, data4: [180, 126, 2, 170, 21, 154, 21, 43] };
pub const SizeCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1440200786, data2: 63185, data3: 17138, data4: [170, 117, 135, 40, 161, 178, 210, 100] };
pub const SmartcardCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2413289884, data2: 15351, data3: 18587, data4: [167, 44, 132, 106, 179, 103, 140, 150] };
pub const SmartcardPinProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2488888446, data2: 14148, data3: 16846, data4: [137, 62, 187, 240, 145, 34, 247, 106] };
pub const SmartcardReaderSelectionProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 455620705, data2: 30031, data3: 16418, data4: [173, 71, 165, 234, 170, 97, 136, 148] };
pub const SmartcardWinRTProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 518468479, data2: 34220, data3: 17890, data4: [162, 60, 55, 199, 83, 32, 151, 105] };
pub const StartMenuPin: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2729006173, data2: 41154, data3: 17076, data4: [151, 8, 160, 178, 186, 221, 119, 200] };
pub const StorageProviderBanners: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2093873652, data2: 58742, data3: 17754, data4: [139, 199, 246, 236, 104, 214, 240, 99] };
pub const SuspensionDependencyManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1797734341, data2: 25085, data3: 18712, data4: [149, 162, 195, 181, 233, 215, 245, 129] };
pub const SyncMgr: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1653989159, data2: 13806, data3: 4561, data4: [135, 7, 0, 192, 79, 217, 51, 39] };
pub const SyncMgrClient: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 302177120, data2: 7596, data3: 17093, data4: [174, 213, 26, 189, 212, 50, 36, 142] };
pub const SyncMgrControl: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 438256134, data2: 1672, data3: 20095, data4: [190, 3, 216, 46, 198, 157, 249, 165] };
pub const SyncMgrFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2624845285, data2: 31463, data3: 20018, data4: [168, 232, 141, 35, 184, 82, 85, 191] };
pub const SyncMgrScheduleWizard: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2374733360, data2: 50257, data3: 16923, data4: [133, 83, 210, 151, 106, 250, 100, 140] };
pub const SyncResultsFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1910084708, data2: 15211, data3: 18268, data4: [178, 65, 225, 88, 131, 32, 117, 41] };
pub const SyncSetupFolder: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 782129600, data2: 46135, data3: 18817, data4: [166, 71, 156, 52, 185, 185, 8, 145] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_APPEND: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_INTERNETBAR: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_NOTOOLBAR: u32 = 196608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_PREPEND: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_REPLACE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBIF_STANDARDTOOLBAR: u32 = 131072u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct TBINFO {
    pub cbuttons: u32,
    pub uFlags: u32,
}
impl ::core::marker::Copy for TBINFO {}
impl ::core::clone::Clone for TBINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type TBPFLAG = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBPF_NOPROGRESS: TBPFLAG = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBPF_INDETERMINATE: TBPFLAG = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBPF_NORMAL: TBPFLAG = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBPF_ERROR: TBPFLAG = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TBPF_PAUSED: TBPFLAG = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBN_CLICKED: u32 = 6144u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
pub struct THUMBBUTTON {
    pub dwMask: THUMBBUTTONMASK,
    pub iId: u32,
    pub iBitmap: u32,
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub szTip: [u16; 260],
    pub dwFlags: THUMBBUTTONFLAGS,
}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::marker::Copy for THUMBBUTTON {}
#[cfg(feature = "Win32_UI_WindowsAndMessaging")]
impl ::core::clone::Clone for THUMBBUTTON {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type THUMBBUTTONFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBF_ENABLED: THUMBBUTTONFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBF_DISABLED: THUMBBUTTONFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBF_DISMISSONCLICK: THUMBBUTTONFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBF_NOBACKGROUND: THUMBBUTTONFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBF_HIDDEN: THUMBBUTTONFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THBF_NONINTERACTIVE: THUMBBUTTONFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type THUMBBUTTONMASK = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THB_BITMAP: THUMBBUTTONMASK = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THB_ICON: THUMBBUTTONMASK = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THB_TOOLTIP: THUMBBUTTONMASK = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const THB_FLAGS: THUMBBUTTONMASK = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TITLEBARNAMELEN: u32 = 40u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type TI_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TI_BITMAP: TI_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TI_JPEG: TI_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type TLENUMF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_RELATIVE_INCLUDE_CURRENT: TLENUMF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_RELATIVE_BACK: TLENUMF = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_RELATIVE_FORE: TLENUMF = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_INCLUDE_UNINVOKEABLE: TLENUMF = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_ABSOLUTE: TLENUMF = 49i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_EXCLUDE_SUBFRAME_ENTRIES: TLENUMF = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLEF_EXCLUDE_ABOUT_PAGES: TLENUMF = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLMENUF_BACK: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLMENUF_FORE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLMENUF_INCLUDECURRENT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLOG_BACK: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLOG_CURRENT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TLOG_FORE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_System_Ole"))]
pub struct TOOLBARITEM {
    pub ptbar: IDockingWindow,
    pub rcBorderTool: super::super::Foundation::RECT,
    pub pwszItem: ::windows_sys::core::PWSTR,
    pub fShow: super::super::Foundation::BOOL,
    pub hMon: super::super::Graphics::Gdi::HMONITOR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for TOOLBARITEM {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for TOOLBARITEM {
    fn clone(&self) -> Self {
        *self
    }
}
pub const TaskbarList: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1459483460, data2: 64877, data3: 4560, data4: [149, 138, 0, 96, 151, 201, 160, 144] };
pub const ThumbnailStreamCache: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3420520147, data2: 19345, data3: 20112, data4: [131, 84, 138, 140, 132, 236, 104, 114] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type ThumbnailStreamCacheOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ExtractIfNotCached: ThumbnailStreamCacheOptions = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ReturnOnlyIfCached: ThumbnailStreamCacheOptions = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ResizeThumbnail: ThumbnailStreamCacheOptions = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const AllowSmallerSize: ThumbnailStreamCacheOptions = 4i32;
pub const TimeCategorizer: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1001656719, data2: 56829, data3: 19760, data4: [163, 72, 159, 181, 214, 191, 26, 254] };
pub const TrackShellMenu: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2188966193, data2: 10814, data3: 4562, data4: [131, 143, 0, 192, 79, 217, 24, 208] };
pub const TrayBandSiteService: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4127903904, data2: 58849, data3: 17867, data4: [181, 26, 225, 91, 159, 139, 41, 52] };
pub const TrayDeskBand: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3863225399, data2: 27752, data3: 20306, data4: [148, 221, 44, 254, 210, 103, 239, 185] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type UNDOCK_REASON = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const UR_RESOLUTION_CHANGE: UNDOCK_REASON = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const UR_MONITOR_DISCONNECT: UNDOCK_REASON = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type URLIS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_URL: URLIS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_OPAQUE: URLIS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_NOHISTORY: URLIS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_FILEURL: URLIS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_APPLIABLE: URLIS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_DIRECTORY: URLIS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLIS_HASQUERY: URLIS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_APPLY_DEFAULT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_APPLY_FORCEAPPLY: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_APPLY_GUESSFILE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_APPLY_GUESSSCHEME: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_BROWSER_MODE: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_CONVERT_IF_DOSPATH: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_DONT_ESCAPE_EXTRA_INFO: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_DONT_SIMPLIFY: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_DONT_UNESCAPE: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_DONT_UNESCAPE_EXTRA_INFO: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_ESCAPE_ASCII_URI_COMPONENT: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_ESCAPE_AS_UTF8: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_ESCAPE_PERCENT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_ESCAPE_SEGMENT_ONLY: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_ESCAPE_SPACES_ONLY: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_ESCAPE_UNSAFE: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_E_INVALID_SYNTAX: ::windows_sys::core::HRESULT = -2147217407i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_E_UNREGISTERED_PROTOCOL: ::windows_sys::core::HRESULT = -2147217406i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_FILE_USE_PATHURL: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_INTERNAL_PATH: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_NO_META: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type URL_PART = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_NONE: URL_PART = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_SCHEME: URL_PART = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_HOSTNAME: URL_PART = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_USERNAME: URL_PART = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_PASSWORD: URL_PART = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_PORT: URL_PART = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PART_QUERY: URL_PART = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PARTFLAG_KEEPSCHEME: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_PLUGGABLE_PROTOCOL: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type URL_SCHEME = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_INVALID: URL_SCHEME = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_UNKNOWN: URL_SCHEME = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_FTP: URL_SCHEME = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_HTTP: URL_SCHEME = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_GOPHER: URL_SCHEME = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MAILTO: URL_SCHEME = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_NEWS: URL_SCHEME = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_NNTP: URL_SCHEME = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_TELNET: URL_SCHEME = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_WAIS: URL_SCHEME = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_FILE: URL_SCHEME = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MK: URL_SCHEME = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_HTTPS: URL_SCHEME = 11i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_SHELL: URL_SCHEME = 12i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_SNEWS: URL_SCHEME = 13i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_LOCAL: URL_SCHEME = 14i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_JAVASCRIPT: URL_SCHEME = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_VBSCRIPT: URL_SCHEME = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_ABOUT: URL_SCHEME = 17i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_RES: URL_SCHEME = 18i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MSSHELLROOTED: URL_SCHEME = 19i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MSSHELLIDLIST: URL_SCHEME = 20i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MSHELP: URL_SCHEME = 21i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MSSHELLDEVICE: URL_SCHEME = 22i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_WILDCARD: URL_SCHEME = 23i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_SEARCH_MS: URL_SCHEME = 24i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_SEARCH: URL_SCHEME = 25i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_KNOWNFOLDER: URL_SCHEME = 26i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_SCHEME_MAXVALUE: URL_SCHEME = 27i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_UNESCAPE: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_UNESCAPE_AS_UTF8: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_UNESCAPE_HIGH_ANSI_ONLY: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_UNESCAPE_INPLACE: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_UNESCAPE_URI_COMPONENT: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URL_WININET_COMPATIBILITY: u32 = 2147483648u32;
pub const UserNotification: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1083662, data2: 34697, data3: 16700, data4: [173, 188, 72, 245, 181, 17, 179, 175] };
pub const V1PasswordCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1866849310, data2: 21380, data3: 17786, data4: [188, 19, 44, 216, 27, 13, 40, 237] };
pub const V1SmartcardCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2348394768, data2: 43263, data3: 17791, data4: [153, 159, 165, 202, 16, 180, 168, 133] };
pub const V1WinBioCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2889531977, data2: 59424, data3: 17219, data4: [166, 91, 55, 122, 198, 52, 220, 9] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type VALIDATEUNC_OPTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VALIDATEUNC_CONNECT: VALIDATEUNC_OPTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VALIDATEUNC_NOUI: VALIDATEUNC_OPTION = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VALIDATEUNC_PRINT: VALIDATEUNC_OPTION = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VALIDATEUNC_PERSIST: VALIDATEUNC_OPTION = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VALIDATEUNC_VALID: VALIDATEUNC_OPTION = 15i32;
pub const VID_Content: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 818070580, data2: 2185, data3: 19597, data4: [152, 93, 169, 247, 24, 48, 176, 169] };
pub const VID_Details: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 327055104, data2: 13683, data3: 4559, data4: [174, 105, 8, 0, 43, 46, 18, 98] };
pub const VID_LargeIcons: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 5755104, data2: 13683, data3: 4559, data4: [174, 105, 8, 0, 43, 46, 18, 98] };
pub const VID_List: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 236955104, data2: 13683, data3: 4559, data4: [174, 105, 8, 0, 43, 46, 18, 98] };
pub const VID_SmallIcons: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 143655104, data2: 13683, data3: 4559, data4: [174, 105, 8, 0, 43, 46, 18, 98] };
pub const VID_ThumbStrip: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2398070308, data2: 53737, data3: 17499, data4: [148, 183, 116, 251, 206, 46, 161, 26] };
pub const VID_Thumbnails: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2347479696, data2: 21200, data3: 4560, data4: [183, 244, 0, 192, 79, 215, 6, 236] };
pub const VID_Tile: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1710302693, data2: 31713, data3: 18448, data4: [186, 157, 210, 113, 200, 67, 44, 227] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_CACHEHIT: u32 = 80u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_CACHEMISS: u32 = 48u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_DESPERATE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_INHERIT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_RESTRICTED: u32 = 112u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_SHELLEXT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_SHELLEXT_ASBACKUP: u32 = 21u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_STALECACHEHIT: u32 = 69u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VIEW_PRIORITY_USEASDEFAULT: u32 = 67u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VOLUME_PREFIX: &str = "\\\\?\\Volume";
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type VPCOLORFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPCF_TEXT: VPCOLORFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPCF_BACKGROUND: VPCOLORFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPCF_SORTCOLUMN: VPCOLORFLAGS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPCF_SUBTEXT: VPCOLORFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPCF_TEXTBACKGROUND: VPCOLORFLAGS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type VPWATERMARKFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPWF_DEFAULT: VPWATERMARKFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const VPWF_ALPHABLEND: VPWATERMARKFLAGS = 1i32;
pub const VaultProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1345796560, data2: 19550, data3: 19709, data4: [179, 186, 216, 129, 51, 79, 13, 242] };
pub const VirtualDesktopManager: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2857406598, data2: 23721, data3: 19493, data4: [143, 149, 88, 157, 60, 7, 180, 138] };
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WC_NETADDRESS: &str = "msctls_netaddress";
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_UI_Shell_Common\"`*"]
#[cfg(feature = "Win32_UI_Shell_Common")]
pub struct WINDOWDATA {
    pub dwWindowID: u32,
    pub uiCP: u32,
    pub pidl: *mut Common::ITEMIDLIST,
    pub lpszUrl: ::windows_sys::core::PWSTR,
    pub lpszUrlLocation: ::windows_sys::core::PWSTR,
    pub lpszTitle: ::windows_sys::core::PWSTR,
}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::marker::Copy for WINDOWDATA {}
#[cfg(feature = "Win32_UI_Shell_Common")]
impl ::core::clone::Clone for WINDOWDATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WM_CPL_LAUNCH: u32 = 2024u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WM_CPL_LAUNCHED: u32 = 2025u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_CENTER: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_CROPTOFIT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_KEEPASPECT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_MAX: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_SPAN: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_STRETCH: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WPSTYLE_TILE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type WTS_ALPHATYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSAT_UNKNOWN: WTS_ALPHATYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSAT_RGB: WTS_ALPHATYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSAT_ARGB: WTS_ALPHATYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type WTS_CACHEFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_DEFAULT: WTS_CACHEFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_LOWQUALITY: WTS_CACHEFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_CACHED: WTS_CACHEFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type WTS_CONTEXTFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSCF_DEFAULT: WTS_CONTEXTFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSCF_APPSTYLE: WTS_CONTEXTFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSCF_SQUARE: WTS_CONTEXTFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSCF_WIDE: WTS_CONTEXTFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTSCF_FAST: WTS_CONTEXTFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_DATAFILEUNAVAILABLE: ::windows_sys::core::HRESULT = -2147175932i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_EXTRACTIONBLOCKED: ::windows_sys::core::HRESULT = -2147175930i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_EXTRACTIONPENDING: ::windows_sys::core::HRESULT = -2147175931i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_EXTRACTIONTIMEDOUT: ::windows_sys::core::HRESULT = -2147175935i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_FAILEDEXTRACTION: ::windows_sys::core::HRESULT = -2147175936i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_FASTEXTRACTIONNOTSUPPORTED: ::windows_sys::core::HRESULT = -2147175933i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_NOSTORAGEPROVIDERTHUMBNAILHANDLER: ::windows_sys::core::HRESULT = -2147175929i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_E_SURROGATEUNAVAILABLE: ::windows_sys::core::HRESULT = -2147175934i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type WTS_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_NONE: WTS_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_EXTRACT: WTS_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_INCACHEONLY: WTS_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_FASTEXTRACT: WTS_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_FORCEEXTRACTION: WTS_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_SLOWRECLAIM: WTS_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_EXTRACTDONOTCACHE: WTS_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_SCALETOREQUESTEDSIZE: WTS_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_SKIPFASTEXTRACT: WTS_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_EXTRACTINPROC: WTS_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_CROPTOSQUARE: WTS_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_INSTANCESURROGATE: WTS_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_REQUIRESURROGATE: WTS_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_APPSTYLE: WTS_FLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_WIDETHUMBNAILS: WTS_FLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_IDEALCACHESIZEONLY: WTS_FLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const WTS_SCALEUP: WTS_FLAGS = 65536i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub struct WTS_THUMBNAILID {
    pub rgbKey: [u8; 16],
}
impl ::core::marker::Copy for WTS_THUMBNAILID {}
impl ::core::clone::Clone for WTS_THUMBNAILID {
    fn clone(&self) -> Self {
        *self
    }
}
pub const WebBrowser: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2287401313, data2: 13322, data3: 4560, data4: [169, 107, 0, 192, 79, 215, 5, 162] };
pub const WebBrowser_V1: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3937544899, data2: 12481, data3: 4559, data4: [167, 235, 0, 0, 192, 91, 174, 11] };
pub const WebWizardHost: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3358060873, data2: 21953, data3: 19752, data4: [147, 94, 87, 228, 124, 174, 217, 115] };
pub const WinBioCredentialProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3200291363, data2: 45080, data3: 16749, data4: [160, 172, 82, 57, 113, 182, 57, 245] };
#[repr(C)]
pub struct _APPCONSTRAIN_REGISTRATION(pub u8);
#[repr(C)]
pub struct _APPSTATE_REGISTRATION(pub u8);
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _BROWSERFRAMEOPTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_NONE: _BROWSERFRAMEOPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_BROWSER_PERSIST_SETTINGS: _BROWSERFRAMEOPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_RENAME_FOLDER_OPTIONS_TOINTERNET: _BROWSERFRAMEOPTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_BOTH_OPTIONS: _BROWSERFRAMEOPTIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BIF_PREFER_INTERNET_SHORTCUT: _BROWSERFRAMEOPTIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_BROWSE_NO_IN_NEW_PROCESS: _BROWSERFRAMEOPTIONS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_ENABLE_HYPERLINK_TRACKING: _BROWSERFRAMEOPTIONS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_USE_IE_OFFLINE_SUPPORT: _BROWSERFRAMEOPTIONS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_SUBSTITUE_INTERNET_START_PAGE: _BROWSERFRAMEOPTIONS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_USE_IE_LOGOBANDING: _BROWSERFRAMEOPTIONS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_ADD_IE_TOCAPTIONBAR: _BROWSERFRAMEOPTIONS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_USE_DIALUP_REF: _BROWSERFRAMEOPTIONS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_USE_IE_TOOLBAR: _BROWSERFRAMEOPTIONS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_NO_PARENT_FOLDER_SUPPORT: _BROWSERFRAMEOPTIONS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_NO_REOPEN_NEXT_RESTART: _BROWSERFRAMEOPTIONS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_GO_HOME_PAGE: _BROWSERFRAMEOPTIONS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_PREFER_IEPROCESS: _BROWSERFRAMEOPTIONS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_SHOW_NAVIGATION_CANCELLED: _BROWSERFRAMEOPTIONS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_USE_IE_STATUSBAR: _BROWSERFRAMEOPTIONS = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const BFO_QUERY_ALL: _BROWSERFRAMEOPTIONS = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _CDBE_ACTIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBE_TYPE_MUSIC: _CDBE_ACTIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBE_TYPE_DATA: _CDBE_ACTIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const CDBE_TYPE_ALL: _CDBE_ACTIONS = -1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _EXPCMDFLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_DEFAULT: _EXPCMDFLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_HASSUBCOMMANDS: _EXPCMDFLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_HASSPLITBUTTON: _EXPCMDFLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_HIDELABEL: _EXPCMDFLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_ISSEPARATOR: _EXPCMDFLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_HASLUASHIELD: _EXPCMDFLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_SEPARATORBEFORE: _EXPCMDFLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_SEPARATORAFTER: _EXPCMDFLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_ISDROPDOWN: _EXPCMDFLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_TOGGLEABLE: _EXPCMDFLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECF_AUTOMENUICONS: _EXPCMDFLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _EXPCMDSTATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECS_ENABLED: _EXPCMDSTATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECS_DISABLED: _EXPCMDSTATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECS_HIDDEN: _EXPCMDSTATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECS_CHECKBOX: _EXPCMDSTATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECS_CHECKED: _EXPCMDSTATE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const ECS_RADIOCHECK: _EXPCMDSTATE = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _EXPLORERPANESTATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EPS_DONTCARE: _EXPLORERPANESTATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EPS_DEFAULT_ON: _EXPLORERPANESTATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EPS_DEFAULT_OFF: _EXPLORERPANESTATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EPS_STATEMASK: _EXPLORERPANESTATE = 65535i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EPS_INITIALSTATE: _EXPLORERPANESTATE = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EPS_FORCE: _EXPLORERPANESTATE = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _EXPPS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const EXPPS_FILETYPES: _EXPPS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _HLSHORTCUTF__NOREDEF10 = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSHORTCUTF_DEFAULT: _HLSHORTCUTF__NOREDEF10 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSHORTCUTF_DONTACTUALLYCREATE: _HLSHORTCUTF__NOREDEF10 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSHORTCUTF_USEFILENAMEFROMFRIENDLYNAME: _HLSHORTCUTF__NOREDEF10 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSHORTCUTF_USEUNIQUEFILENAME: _HLSHORTCUTF__NOREDEF10 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSHORTCUTF_MAYUSEEXISTINGSHORTCUT: _HLSHORTCUTF__NOREDEF10 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _HLSR_NOREDEF10 = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSR_HOME: _HLSR_NOREDEF10 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSR_SEARCHPAGE: _HLSR_NOREDEF10 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLSR_HISTORYFOLDER: _HLSR_NOREDEF10 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _HLTRANSLATEF_NOREDEF10 = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTRANSLATEF_DEFAULT: _HLTRANSLATEF_NOREDEF10 = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const HLTRANSLATEF_DONTAPPLYDEFAULTPREFIX: _HLTRANSLATEF_NOREDEF10 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _KF_DEFINITION_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KFDF_LOCAL_REDIRECT_ONLY: _KF_DEFINITION_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KFDF_ROAMABLE: _KF_DEFINITION_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KFDF_PRECREATE: _KF_DEFINITION_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KFDF_STREAM: _KF_DEFINITION_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KFDF_PUBLISHEXPANDEDPATH: _KF_DEFINITION_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KFDF_NO_REDIRECT_UI: _KF_DEFINITION_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _KF_REDIRECTION_CAPABILITIES = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECTION_CAPABILITIES_ALLOW_ALL: _KF_REDIRECTION_CAPABILITIES = 255i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECTION_CAPABILITIES_REDIRECTABLE: _KF_REDIRECTION_CAPABILITIES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECTION_CAPABILITIES_DENY_ALL: _KF_REDIRECTION_CAPABILITIES = 1048320i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECTION_CAPABILITIES_DENY_POLICY_REDIRECTED: _KF_REDIRECTION_CAPABILITIES = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECTION_CAPABILITIES_DENY_POLICY: _KF_REDIRECTION_CAPABILITIES = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECTION_CAPABILITIES_DENY_PERMISSIONS: _KF_REDIRECTION_CAPABILITIES = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _KF_REDIRECT_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_USER_EXCLUSIVE: _KF_REDIRECT_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_COPY_SOURCE_DACL: _KF_REDIRECT_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_OWNER_USER: _KF_REDIRECT_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_SET_OWNER_EXPLICIT: _KF_REDIRECT_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_CHECK_ONLY: _KF_REDIRECT_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_WITH_UI: _KF_REDIRECT_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_UNPIN: _KF_REDIRECT_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_PIN: _KF_REDIRECT_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_COPY_CONTENTS: _KF_REDIRECT_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_DEL_SOURCE_CONTENTS: _KF_REDIRECT_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const KF_REDIRECT_EXCLUDE_ALL_KNOWN_SUBFOLDERS: _KF_REDIRECT_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NMCII_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NMCII_NONE: _NMCII_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NMCII_ITEMS: _NMCII_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NMCII_FOLDERS: _NMCII_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NMCSAEI_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NMCSAEI_SELECT: _NMCSAEI_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NMCSAEI_EDIT: _NMCSAEI_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NSTCECLICKTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCECT_LBUTTON: _NSTCECLICKTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCECT_MBUTTON: _NSTCECLICKTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCECT_RBUTTON: _NSTCECLICKTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCECT_BUTTON: _NSTCECLICKTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCECT_DBLCLICK: _NSTCECLICKTYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NSTCEHITTEST = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_NOWHERE: _NSTCEHITTEST = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMICON: _NSTCEHITTEST = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMLABEL: _NSTCEHITTEST = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMINDENT: _NSTCEHITTEST = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMBUTTON: _NSTCEHITTEST = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMRIGHT: _NSTCEHITTEST = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMSTATEICON: _NSTCEHITTEST = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEM: _NSTCEHITTEST = 70i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCEHT_ONITEMTABBUTTON: _NSTCEHITTEST = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NSTCITEMSTATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCIS_NONE: _NSTCITEMSTATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCIS_SELECTED: _NSTCITEMSTATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCIS_EXPANDED: _NSTCITEMSTATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCIS_BOLD: _NSTCITEMSTATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCIS_DISABLED: _NSTCITEMSTATE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCIS_SELECTEDNOEXPAND: _NSTCITEMSTATE = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NSTCROOTSTYLE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCRS_VISIBLE: _NSTCROOTSTYLE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCRS_HIDDEN: _NSTCROOTSTYLE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCRS_EXPANDED: _NSTCROOTSTYLE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _NSTCSTYLE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_HASEXPANDOS: _NSTCSTYLE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_HASLINES: _NSTCSTYLE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_SINGLECLICKEXPAND: _NSTCSTYLE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_FULLROWSELECT: _NSTCSTYLE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_SPRINGEXPAND: _NSTCSTYLE = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_HORIZONTALSCROLL: _NSTCSTYLE = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_ROOTHASEXPANDO: _NSTCSTYLE = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_SHOWSELECTIONALWAYS: _NSTCSTYLE = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_NOINFOTIP: _NSTCSTYLE = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_EVENHEIGHT: _NSTCSTYLE = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_NOREPLACEOPEN: _NSTCSTYLE = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_DISABLEDRAGDROP: _NSTCSTYLE = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_NOORDERSTREAM: _NSTCSTYLE = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_RICHTOOLTIP: _NSTCSTYLE = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_BORDER: _NSTCSTYLE = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_NOEDITLABELS: _NSTCSTYLE = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_TABSTOP: _NSTCSTYLE = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_FAVORITESMODE: _NSTCSTYLE = 524288i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_AUTOHSCROLL: _NSTCSTYLE = 1048576i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_FADEINOUTEXPANDOS: _NSTCSTYLE = 2097152i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_EMPTYTEXT: _NSTCSTYLE = 4194304i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_CHECKBOXES: _NSTCSTYLE = 8388608i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_PARTIALCHECKBOXES: _NSTCSTYLE = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_EXCLUSIONCHECKBOXES: _NSTCSTYLE = 33554432i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_DIMMEDCHECKBOXES: _NSTCSTYLE = 67108864i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_NOINDENTCHECKS: _NSTCSTYLE = 134217728i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_ALLOWJUNCTIONS: _NSTCSTYLE = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_SHOWTABSBUTTON: _NSTCSTYLE = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_SHOWDELETEBUTTON: _NSTCSTYLE = 1073741824i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const NSTCS_SHOWREFRESHBUTTON: _NSTCSTYLE = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _OPPROGDLGF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_DEFAULT: _OPPROGDLGF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_ENABLEPAUSE: _OPPROGDLGF = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_ALLOWUNDO: _OPPROGDLGF = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_DONTDISPLAYSOURCEPATH: _OPPROGDLGF = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_DONTDISPLAYDESTPATH: _OPPROGDLGF = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_NOMULTIDAYESTIMATES: _OPPROGDLGF = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const OPPROGDLG_DONTDISPLAYLOCATIONS: _OPPROGDLGF = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _PDMODE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDM_DEFAULT: _PDMODE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDM_RUN: _PDMODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDM_PREFLIGHT: _PDMODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDM_UNDOING: _PDMODE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDM_ERRORSBLOCKING: _PDMODE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const PDM_INDETERMINATE: _PDMODE = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SHCONTF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_CHECKING_FOR_CHILDREN: _SHCONTF = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_FOLDERS: _SHCONTF = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_NONFOLDERS: _SHCONTF = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_INCLUDEHIDDEN: _SHCONTF = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_INIT_ON_FIRST_NEXT: _SHCONTF = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_NETPRINTERSRCH: _SHCONTF = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_SHAREABLE: _SHCONTF = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_STORAGE: _SHCONTF = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_NAVIGATION_ENUM: _SHCONTF = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_FASTITEMS: _SHCONTF = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_FLATLIST: _SHCONTF = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_ENABLE_ASYNC: _SHCONTF = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHCONTF_INCLUDESUPERHIDDEN: _SHCONTF = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SHGDNF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDN_NORMAL: _SHGDNF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDN_INFOLDER: _SHGDNF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDN_FOREDITING: _SHGDNF = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDN_FORADDRESSBAR: _SHGDNF = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SHGDN_FORPARSING: _SHGDNF = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SICHINTF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SICHINT_DISPLAY: _SICHINTF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SICHINT_ALLFIELDS: _SICHINTF = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SICHINT_CANONICAL: _SICHINTF = 268435456i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SICHINT_TEST_FILESYSPATH_IF_NOT_EQUAL: _SICHINTF = 536870912i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SPBEGINF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPBEGINF_NORMAL: _SPBEGINF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPBEGINF_AUTOTIME: _SPBEGINF = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPBEGINF_NOPROGRESSBAR: _SPBEGINF = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPBEGINF_MARQUEEPROGRESS: _SPBEGINF = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPBEGINF_NOCANCELBUTTON: _SPBEGINF = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SPINITF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPINITF_NORMAL: _SPINITF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPINITF_MODAL: _SPINITF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SPINITF_NOMINIMIZE: _SPINITF = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SV3CVW3_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SV3CVW3_DEFAULT: _SV3CVW3_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SV3CVW3_NONINTERACTIVE: _SV3CVW3_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SV3CVW3_FORCEVIEWMODE: _SV3CVW3_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SV3CVW3_FORCEFOLDERFLAGS: _SV3CVW3_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SVGIO = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVGIO_BACKGROUND: _SVGIO = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVGIO_SELECTION: _SVGIO = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVGIO_ALLVIEW: _SVGIO = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVGIO_CHECKED: _SVGIO = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVGIO_TYPE_MASK: _SVGIO = 15i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVGIO_FLAG_VIEWORDER: _SVGIO = -2147483648i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _SVSIF = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_DESELECT: _SVSIF = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_SELECT: _SVSIF = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_EDIT: _SVSIF = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_DESELECTOTHERS: _SVSIF = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_ENSUREVISIBLE: _SVSIF = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_FOCUSED: _SVSIF = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_TRANSLATEPT: _SVSIF = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_SELECTIONMARK: _SVSIF = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_POSITIONITEM: _SVSIF = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_CHECK: _SVSIF = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_CHECK2: _SVSIF = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_KEYBOARDSELECT: _SVSIF = 1025i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const SVSI_NOTAKEFOCUS: _SVSIF = 1073741824i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _TRANSFER_ADVISE_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TS_NONE: _TRANSFER_ADVISE_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TS_PERFORMING: _TRANSFER_ADVISE_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TS_PREPARING: _TRANSFER_ADVISE_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TS_INDETERMINATE: _TRANSFER_ADVISE_STATE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type _TRANSFER_SOURCE_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_NORMAL: _TRANSFER_SOURCE_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_FAIL_EXIST: _TRANSFER_SOURCE_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_RENAME_EXIST: _TRANSFER_SOURCE_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_OVERWRITE_EXIST: _TRANSFER_SOURCE_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_ALLOW_DECRYPTION: _TRANSFER_SOURCE_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_NO_SECURITY: _TRANSFER_SOURCE_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_COPY_CREATION_TIME: _TRANSFER_SOURCE_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_COPY_WRITE_TIME: _TRANSFER_SOURCE_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_USE_FULL_ACCESS: _TRANSFER_SOURCE_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_DELETE_RECYCLE_IF_POSSIBLE: _TRANSFER_SOURCE_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_COPY_HARD_LINK: _TRANSFER_SOURCE_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_COPY_LOCALIZED_NAME: _TRANSFER_SOURCE_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_MOVE_AS_COPY_DELETE: _TRANSFER_SOURCE_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TSF_SUSPEND_SHELLEVENTS: _TRANSFER_SOURCE_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsAppName: u32 = 1007u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsBadOldPW: u32 = 1006u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsChangePW: u32 = 1005u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsDefKeyword: u32 = 1010u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsDifferentPW: u32 = 1004u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsHelpFile: u32 = 1009u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsIniFile: u32 = 1001u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsIsPassword: u32 = 1000u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsNoHelpMemory: u32 = 1008u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsPassword: u32 = 1003u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const idsScreenSaver: u32 = 1002u32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type iurl_invokecommand_flags = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_INVOKECOMMAND_FL_ALLOW_UI: iurl_invokecommand_flags = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_INVOKECOMMAND_FL_USE_DEFAULT_VERB: iurl_invokecommand_flags = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_INVOKECOMMAND_FL_DDEWAIT: iurl_invokecommand_flags = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_INVOKECOMMAND_FL_ASYNCOK: iurl_invokecommand_flags = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_INVOKECOMMAND_FL_LOG_USAGE: iurl_invokecommand_flags = 16i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type iurl_seturl_flags = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_SETURL_FL_GUESS_PROTOCOL: iurl_seturl_flags = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const IURL_SETURL_FL_USE_DEFAULT_PROTOCOL: iurl_seturl_flags = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type mimeassociationdialog_in_flags = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const MIMEASSOCDLG_FL_REGISTER_ASSOC: mimeassociationdialog_in_flags = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type translateurl_in_flags = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TRANSLATEURL_FL_GUESS_PROTOCOL: translateurl_in_flags = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const TRANSLATEURL_FL_USE_DEFAULT_PROTOCOL: translateurl_in_flags = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub type urlassociationdialog_in_flags = i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLASSOCDLG_FL_USE_DEFAULT_NAME: urlassociationdialog_in_flags = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell\"`*"]
pub const URLASSOCDLG_FL_REGISTER_ASSOC: urlassociationdialog_in_flags = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct urlinvokecommandinfoA {
    pub dwcbSize: u32,
    pub dwFlags: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub pcszVerb: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for urlinvokecommandinfoA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for urlinvokecommandinfoA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct urlinvokecommandinfoW {
    pub dwcbSize: u32,
    pub dwFlags: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub pcszVerb: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for urlinvokecommandinfoW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for urlinvokecommandinfoW {
    fn clone(&self) -> Self {
        *self
    }
}
