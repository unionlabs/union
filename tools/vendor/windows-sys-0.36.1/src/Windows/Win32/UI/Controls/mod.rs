#[cfg(feature = "Win32_UI_Controls_Dialogs")]
pub mod Dialogs;
#[cfg(feature = "Win32_UI_Controls_RichEdit")]
pub mod RichEdit;
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn BeginBufferedAnimation(hwnd: super::super::Foundation::HWND, hdctarget: super::super::Graphics::Gdi::HDC, prctarget: *const super::super::Foundation::RECT, dwformat: BP_BUFFERFORMAT, ppaintparams: *const BP_PAINTPARAMS, panimationparams: *const BP_ANIMATIONPARAMS, phdcfrom: *mut super::super::Graphics::Gdi::HDC, phdcto: *mut super::super::Graphics::Gdi::HDC) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn BeginBufferedPaint(hdctarget: super::super::Graphics::Gdi::HDC, prctarget: *const super::super::Foundation::RECT, dwformat: BP_BUFFERFORMAT, ppaintparams: *const BP_PAINTPARAMS, phdc: *mut super::super::Graphics::Gdi::HDC) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BeginPanningFeedback(hwnd: super::super::Foundation::HWND) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BufferedPaintClear(hbufferedpaint: isize, prc: *const super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn BufferedPaintInit() -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn BufferedPaintRenderAnimation(hwnd: super::super::Foundation::HWND, hdctarget: super::super::Graphics::Gdi::HDC) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BufferedPaintSetAlpha(hbufferedpaint: isize, prc: *const super::super::Foundation::RECT, alpha: u8) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn BufferedPaintStopAllAnimations(hwnd: super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn BufferedPaintUnInit() -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CheckDlgButton(hdlg: super::super::Foundation::HWND, nidbutton: i32, ucheck: DLG_BUTTON_CHECK_STATE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CheckRadioButton(hdlg: super::super::Foundation::HWND, nidfirstbutton: i32, nidlastbutton: i32, nidcheckbutton: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn CloseThemeData(htheme: isize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn CreateMappedBitmap(hinstance: super::super::Foundation::HINSTANCE, idbitmap: isize, wflags: u32, lpcolormap: *const COLORMAP, inummaps: i32) -> super::super::Graphics::Gdi::HBITMAP;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn CreatePropertySheetPageA(constpropsheetpagepointer: *mut PROPSHEETPAGEA) -> HPROPSHEETPAGE;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn CreatePropertySheetPageW(constpropsheetpagepointer: *mut PROPSHEETPAGEW) -> HPROPSHEETPAGE;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateStatusWindowA(style: i32, lpsztext: ::windows_sys::core::PCSTR, hwndparent: super::super::Foundation::HWND, wid: u32) -> super::super::Foundation::HWND;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateStatusWindowW(style: i32, lpsztext: ::windows_sys::core::PCWSTR, hwndparent: super::super::Foundation::HWND, wid: u32) -> super::super::Foundation::HWND;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn CreateSyntheticPointerDevice(pointertype: super::WindowsAndMessaging::POINTER_INPUT_TYPE, maxcount: u32, mode: POINTER_FEEDBACK_MODE) -> HSYNTHETICPOINTERDEVICE;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateToolbarEx(hwnd: super::super::Foundation::HWND, ws: u32, wid: u32, nbitmaps: i32, hbminst: super::super::Foundation::HINSTANCE, wbmid: usize, lpbuttons: *mut TBBUTTON, inumbuttons: i32, dxbutton: i32, dybutton: i32, dxbitmap: i32, dybitmap: i32, ustructsize: u32) -> super::super::Foundation::HWND;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateUpDownControl(dwstyle: u32, x: i32, y: i32, cx: i32, cy: i32, hparent: super::super::Foundation::HWND, nid: i32, hinst: super::super::Foundation::HINSTANCE, hbuddy: super::super::Foundation::HWND, nupper: i32, nlower: i32, npos: i32) -> super::super::Foundation::HWND;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_Clone(hdpa: HDPA, hdpanew: HDPA) -> HDPA;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_Create(citemgrow: i32) -> HDPA;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_CreateEx(cpgrow: i32, hheap: super::super::Foundation::HANDLE) -> HDPA;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_DeleteAllPtrs(hdpa: HDPA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_DeletePtr(hdpa: HDPA, i: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_Destroy(hdpa: HDPA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_DestroyCallback(hdpa: HDPA, pfncb: PFNDAENUMCALLBACK, pdata: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_EnumCallback(hdpa: HDPA, pfncb: PFNDAENUMCALLBACK, pdata: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_GetPtr(hdpa: HDPA, i: isize) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_GetPtrIndex(hdpa: HDPA, p: *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_GetSize(hdpa: HDPA) -> u64;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_Grow(pdpa: HDPA, cp: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DPA_InsertPtr(hdpa: HDPA, i: i32, p: *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn DPA_LoadStream(phdpa: *mut HDPA, pfn: PFNDPASTREAM, pstream: super::super::System::Com::IStream, pvinstdata: *const ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_Merge(hdpadest: HDPA, hdpasrc: HDPA, dwflags: u32, pfncompare: PFNDACOMPARE, pfnmerge: PFNDPAMERGE, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn DPA_SaveStream(hdpa: HDPA, pfn: PFNDPASTREAM, pstream: super::super::System::Com::IStream, pvinstdata: *const ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_Search(hdpa: HDPA, pfind: *const ::core::ffi::c_void, istart: i32, pfncompare: PFNDACOMPARE, lparam: super::super::Foundation::LPARAM, options: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_SetPtr(hdpa: HDPA, i: i32, p: *const ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DPA_Sort(hdpa: HDPA, pfncompare: PFNDACOMPARE, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_Clone(hdsa: HDSA) -> HDSA;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_Create(cbitem: i32, citemgrow: i32) -> HDSA;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DSA_DeleteAllItems(hdsa: HDSA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DSA_DeleteItem(hdsa: HDSA, i: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DSA_Destroy(hdsa: HDSA) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_DestroyCallback(hdsa: HDSA, pfncb: PFNDAENUMCALLBACK, pdata: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_EnumCallback(hdsa: HDSA, pfncb: PFNDAENUMCALLBACK, pdata: *const ::core::ffi::c_void);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DSA_GetItem(hdsa: HDSA, i: i32, pitem: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_GetItemPtr(hdsa: HDSA, i: i32) -> *mut ::core::ffi::c_void;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_GetSize(hdsa: HDSA) -> u64;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DSA_InsertItem(hdsa: HDSA, i: i32, pitem: *const ::core::ffi::c_void) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DSA_SetItem(hdsa: HDSA, i: i32, pitem: *const ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DSA_Sort(pdsa: HDSA, pfncompare: PFNDACOMPARE, lparam: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DestroyPropertySheetPage(param0: HPROPSHEETPAGE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn DestroySyntheticPointerDevice(device: HSYNTHETICPOINTERDEVICE);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirListA(hdlg: super::super::Foundation::HWND, lppathspec: ::windows_sys::core::PSTR, nidlistbox: i32, nidstaticpath: i32, ufiletype: DLG_DIR_LIST_FILE_TYPE) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirListComboBoxA(hdlg: super::super::Foundation::HWND, lppathspec: ::windows_sys::core::PSTR, nidcombobox: i32, nidstaticpath: i32, ufiletype: DLG_DIR_LIST_FILE_TYPE) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirListComboBoxW(hdlg: super::super::Foundation::HWND, lppathspec: ::windows_sys::core::PWSTR, nidcombobox: i32, nidstaticpath: i32, ufiletype: DLG_DIR_LIST_FILE_TYPE) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirListW(hdlg: super::super::Foundation::HWND, lppathspec: ::windows_sys::core::PWSTR, nidlistbox: i32, nidstaticpath: i32, ufiletype: DLG_DIR_LIST_FILE_TYPE) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirSelectComboBoxExA(hwnddlg: super::super::Foundation::HWND, lpstring: ::windows_sys::core::PSTR, cchout: i32, idcombobox: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirSelectComboBoxExW(hwnddlg: super::super::Foundation::HWND, lpstring: ::windows_sys::core::PWSTR, cchout: i32, idcombobox: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirSelectExA(hwnddlg: super::super::Foundation::HWND, lpstring: ::windows_sys::core::PSTR, chcount: i32, idlistbox: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DlgDirSelectExW(hwnddlg: super::super::Foundation::HWND, lpstring: ::windows_sys::core::PWSTR, chcount: i32, idlistbox: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn DrawInsert(handparent: super::super::Foundation::HWND, hlb: super::super::Foundation::HWND, nitem: i32);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawShadowText(hdc: super::super::Graphics::Gdi::HDC, psztext: ::windows_sys::core::PCWSTR, cch: u32, prc: *const super::super::Foundation::RECT, dwflags: u32, crtext: u32, crshadow: u32, ixoffset: i32, iyoffset: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawStatusTextA(hdc: super::super::Graphics::Gdi::HDC, lprc: *mut super::super::Foundation::RECT, psztext: ::windows_sys::core::PCSTR, uflags: u32);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawStatusTextW(hdc: super::super::Graphics::Gdi::HDC, lprc: *mut super::super::Foundation::RECT, psztext: ::windows_sys::core::PCWSTR, uflags: u32);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeBackground(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, prect: *const super::super::Foundation::RECT, pcliprect: *const super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeBackgroundEx(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, prect: *const super::super::Foundation::RECT, poptions: *const DTBGOPTS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeEdge(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, pdestrect: *const super::super::Foundation::RECT, uedge: u32, uflags: u32, pcontentrect: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeIcon(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, prect: *const super::super::Foundation::RECT, himl: HIMAGELIST, iimageindex: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeParentBackground(hwnd: super::super::Foundation::HWND, hdc: super::super::Graphics::Gdi::HDC, prc: *const super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeParentBackgroundEx(hwnd: super::super::Foundation::HWND, hdc: super::super::Graphics::Gdi::HDC, dwflags: DRAW_THEME_PARENT_BACKGROUND_FLAGS, prc: *const super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeText(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, psztext: ::windows_sys::core::PCWSTR, cchtext: i32, dwtextflags: u32, dwtextflags2: u32, prect: *const super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn DrawThemeTextEx(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, psztext: ::windows_sys::core::PCWSTR, cchtext: i32, dwtextflags: u32, prect: *mut super::super::Foundation::RECT, poptions: *const DTTOPTS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn EnableScrollBar(hwnd: super::super::Foundation::HWND, wsbflags: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, warrows: ENABLE_SCROLL_BAR_ARROWS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnableThemeDialogTexture(hwnd: super::super::Foundation::HWND, dwflags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EnableTheming(fenable: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EndBufferedAnimation(hbpanimation: isize, fupdatetarget: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EndBufferedPaint(hbufferedpaint: isize, fupdatetarget: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EndPanningFeedback(hwnd: super::super::Foundation::HWND, fanimateback: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EvaluateProximityToPolygon(numvertices: u32, controlpolygon: *const super::super::Foundation::POINT, phittestinginput: *const TOUCH_HIT_TESTING_INPUT, pproximityeval: *mut TOUCH_HIT_TESTING_PROXIMITY_EVALUATION) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn EvaluateProximityToRect(controlboundingbox: *const super::super::Foundation::RECT, phittestinginput: *const TOUCH_HIT_TESTING_INPUT, pproximityeval: *mut TOUCH_HIT_TESTING_PROXIMITY_EVALUATION) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FlatSB_EnableScrollBar(param0: super::super::Foundation::HWND, param1: i32, param2: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_GetScrollInfo(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, param2: *mut super::WindowsAndMessaging::SCROLLINFO) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_GetScrollPos(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FlatSB_GetScrollProp(param0: super::super::Foundation::HWND, propindex: WSB_PROP, param2: *mut i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_GetScrollRange(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, param2: *mut i32, param3: *mut i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_SetScrollInfo(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, psi: *mut super::WindowsAndMessaging::SCROLLINFO, fredraw: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_SetScrollPos(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, pos: i32, fredraw: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn FlatSB_SetScrollProp(param0: super::super::Foundation::HWND, index: WSB_PROP, newvalue: isize, param3: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_SetScrollRange(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, min: i32, max: i32, fredraw: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn FlatSB_ShowScrollBar(param0: super::super::Foundation::HWND, code: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, param2: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetBufferedPaintBits(hbufferedpaint: isize, ppbbuffer: *mut *mut super::super::Graphics::Gdi::RGBQUAD, pcxrow: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetBufferedPaintDC(hbufferedpaint: isize) -> super::super::Graphics::Gdi::HDC;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetBufferedPaintTargetDC(hbufferedpaint: isize) -> super::super::Graphics::Gdi::HDC;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetBufferedPaintTargetRect(hbufferedpaint: isize, prc: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetComboBoxInfo(hwndcombo: super::super::Foundation::HWND, pcbi: *mut COMBOBOXINFO) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetCurrentThemeName(pszthemefilename: ::windows_sys::core::PWSTR, cchmaxnamechars: i32, pszcolorbuff: ::windows_sys::core::PWSTR, cchmaxcolorchars: i32, pszsizebuff: ::windows_sys::core::PWSTR, cchmaxsizechars: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetEffectiveClientRect(hwnd: super::super::Foundation::HWND, lprc: *mut super::super::Foundation::RECT, lpinfo: *const i32);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetListBoxInfo(hwnd: super::super::Foundation::HWND) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetMUILanguage() -> u16;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeAnimationProperty(htheme: isize, istoryboardid: i32, itargetid: i32, eproperty: TA_PROPERTY, pvproperty: *mut ::core::ffi::c_void, cbsize: u32, pcbsizeout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeAnimationTransform(htheme: isize, istoryboardid: i32, itargetid: i32, dwtransformindex: u32, ptransform: *mut TA_TRANSFORM, cbsize: u32, pcbsizeout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeAppProperties() -> u32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn GetThemeBackgroundContentRect(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, pboundingrect: *const super::super::Foundation::RECT, pcontentrect: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn GetThemeBackgroundExtent(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, pcontentrect: *const super::super::Foundation::RECT, pextentrect: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn GetThemeBackgroundRegion(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, prect: *const super::super::Foundation::RECT, pregion: *mut super::super::Graphics::Gdi::HRGN) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetThemeBitmap(htheme: isize, ipartid: i32, istateid: i32, ipropid: THEME_PROPERTY_SYMBOL_ID, dwflags: GET_THEME_BITMAP_FLAGS, phbitmap: *mut super::super::Graphics::Gdi::HBITMAP) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetThemeBool(htheme: isize, ipartid: i32, istateid: i32, ipropid: THEME_PROPERTY_SYMBOL_ID, pfval: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeColor(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, pcolor: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeDocumentationProperty(pszthemename: ::windows_sys::core::PCWSTR, pszpropertyname: ::windows_sys::core::PCWSTR, pszvaluebuff: ::windows_sys::core::PWSTR, cchmaxvalchars: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeEnumValue(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, pival: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeFilename(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, pszthemefilename: ::windows_sys::core::PWSTR, cchmaxbuffchars: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetThemeFont(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, ipropid: i32, pfont: *mut super::super::Graphics::Gdi::LOGFONTW) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeInt(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, pival: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeIntList(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, pintlist: *mut INTLIST) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn GetThemeMargins(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, ipropid: i32, prc: *const super::super::Foundation::RECT, pmargins: *mut MARGINS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetThemeMetric(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, ipropid: THEME_PROPERTY_SYMBOL_ID, pival: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn GetThemePartSize(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, prc: *const super::super::Foundation::RECT, esize: THEMESIZE, psz: *mut super::super::Foundation::SIZE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetThemePosition(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, ppoint: *mut super::super::Foundation::POINT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemePropertyOrigin(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, porigin: *mut PROPERTYORIGIN) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetThemeRect(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, prect: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetThemeStream(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, ppvstream: *mut *mut ::core::ffi::c_void, pcbstream: *mut u32, hinst: super::super::Foundation::HINSTANCE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeString(htheme: isize, ipartid: i32, istateid: i32, ipropid: i32, pszbuff: ::windows_sys::core::PWSTR, cchmaxbuffchars: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetThemeSysBool(htheme: isize, iboolid: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeSysColor(htheme: isize, icolorid: i32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetThemeSysColorBrush(htheme: isize, icolorid: THEME_PROPERTY_SYMBOL_ID) -> super::super::Graphics::Gdi::HBRUSH;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetThemeSysFont(htheme: isize, ifontid: THEME_PROPERTY_SYMBOL_ID, plf: *mut super::super::Graphics::Gdi::LOGFONTW) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeSysInt(htheme: isize, iintid: i32, pivalue: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeSysSize(htheme: isize, isizeid: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeSysString(htheme: isize, istringid: THEME_PROPERTY_SYMBOL_ID, pszstringbuff: ::windows_sys::core::PWSTR, cchmaxstringchars: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn GetThemeTextExtent(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, psztext: ::windows_sys::core::PCWSTR, cchcharcount: i32, dwtextflags: u32, pboundingrect: *const super::super::Foundation::RECT, pextentrect: *mut super::super::Foundation::RECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn GetThemeTextMetrics(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, ptm: *mut super::super::Graphics::Gdi::TEXTMETRICW) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeTimingFunction(htheme: isize, itimingfunctionid: i32, ptimingfunction: *mut TA_TIMINGFUNCTION, cbsize: u32, pcbsizeout: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn GetThemeTransitionDuration(htheme: isize, ipartid: i32, istateidfrom: i32, istateidto: i32, ipropid: i32, pdwduration: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetWindowFeedbackSetting(hwnd: super::super::Foundation::HWND, feedback: FEEDBACK_TYPE, dwflags: u32, psize: *mut u32, config: *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn GetWindowTheme(hwnd: super::super::Foundation::HWND) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn HIMAGELIST_QueryInterface(himl: HIMAGELIST, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn HitTestThemeBackground(htheme: isize, hdc: super::super::Graphics::Gdi::HDC, ipartid: i32, istateid: i32, dwoptions: u32, prect: *const super::super::Foundation::RECT, hrgn: super::super::Graphics::Gdi::HRGN, pttest: super::super::Foundation::POINT, pwhittestcode: *mut u16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn ImageList_Add(himl: HIMAGELIST, hbmimage: super::super::Graphics::Gdi::HBITMAP, hbmmask: super::super::Graphics::Gdi::HBITMAP) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(feature = "Win32_Graphics_Gdi")]
    pub fn ImageList_AddMasked(himl: HIMAGELIST, hbmimage: super::super::Graphics::Gdi::HBITMAP, crmask: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_BeginDrag(himltrack: HIMAGELIST, itrack: i32, dxhotspot: i32, dyhotspot: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_CoCreateInstance(rclsid: *const ::windows_sys::core::GUID, punkouter: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_Copy(himldst: HIMAGELIST, idst: i32, himlsrc: HIMAGELIST, isrc: i32, uflags: IMAGE_LIST_COPY_FLAGS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_Create(cx: i32, cy: i32, flags: IMAGELIST_CREATION_FLAGS, cinitial: i32, cgrow: i32) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_Destroy(himl: HIMAGELIST) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_DragEnter(hwndlock: super::super::Foundation::HWND, x: i32, y: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_DragLeave(hwndlock: super::super::Foundation::HWND) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_DragMove(x: i32, y: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_DragShowNolock(fshow: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn ImageList_Draw(himl: HIMAGELIST, i: i32, hdcdst: super::super::Graphics::Gdi::HDC, x: i32, y: i32, fstyle: IMAGE_LIST_DRAW_STYLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn ImageList_DrawEx(himl: HIMAGELIST, i: i32, hdcdst: super::super::Graphics::Gdi::HDC, x: i32, y: i32, dx: i32, dy: i32, rgbbk: u32, rgbfg: u32, fstyle: IMAGE_LIST_DRAW_STYLE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn ImageList_DrawIndirect(pimldp: *const IMAGELISTDRAWPARAMS) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_Duplicate(himl: HIMAGELIST) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_EndDrag();
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_GetBkColor(himl: HIMAGELIST) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_GetDragImage(ppt: *mut super::super::Foundation::POINT, ppthotspot: *mut super::super::Foundation::POINT) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn ImageList_GetIcon(himl: HIMAGELIST, i: i32, flags: u32) -> super::WindowsAndMessaging::HICON;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_GetIconSize(himl: HIMAGELIST, cx: *mut i32, cy: *mut i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_GetImageCount(himl: HIMAGELIST) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn ImageList_GetImageInfo(himl: HIMAGELIST, i: i32, pimageinfo: *mut IMAGEINFO) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ImageList_LoadImageA(hi: super::super::Foundation::HINSTANCE, lpbmp: ::windows_sys::core::PCSTR, cx: i32, cgrow: i32, crmask: u32, utype: u32, uflags: super::WindowsAndMessaging::IMAGE_FLAGS) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ImageList_LoadImageW(hi: super::super::Foundation::HINSTANCE, lpbmp: ::windows_sys::core::PCWSTR, cx: i32, cgrow: i32, crmask: u32, utype: u32, uflags: super::WindowsAndMessaging::IMAGE_FLAGS) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_Merge(himl1: HIMAGELIST, i1: i32, himl2: HIMAGELIST, i2: i32, dx: i32, dy: i32) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn ImageList_Read(pstm: super::super::System::Com::IStream) -> HIMAGELIST;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn ImageList_ReadEx(dwflags: u32, pstm: super::super::System::Com::IStream, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_Remove(himl: HIMAGELIST, i: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
    pub fn ImageList_Replace(himl: HIMAGELIST, i: i32, hbmimage: super::super::Graphics::Gdi::HBITMAP, hbmmask: super::super::Graphics::Gdi::HBITMAP) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(feature = "Win32_UI_WindowsAndMessaging")]
    pub fn ImageList_ReplaceIcon(himl: HIMAGELIST, i: i32, hicon: super::WindowsAndMessaging::HICON) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn ImageList_SetBkColor(himl: HIMAGELIST, clrbk: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_SetDragCursorImage(himldrag: HIMAGELIST, idrag: i32, dxhotspot: i32, dyhotspot: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_SetIconSize(himl: HIMAGELIST, cx: i32, cy: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_SetImageCount(himl: HIMAGELIST, unewcount: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ImageList_SetOverlayImage(himl: HIMAGELIST, iimage: i32, ioverlay: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn ImageList_Write(himl: HIMAGELIST, pstm: super::super::System::Com::IStream) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn ImageList_WriteEx(himl: HIMAGELIST, dwflags: u32, pstm: super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn InitCommonControls();
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn InitCommonControlsEx(picce: *const INITCOMMONCONTROLSEX) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn InitMUILanguage(uilang: u16);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn InitializeFlatSB(param0: super::super::Foundation::HWND) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsAppThemed() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsCharLowerW(ch: u16) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsCompositionActive() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsDlgButtonChecked(hdlg: super::super::Foundation::HWND, nidbutton: i32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsThemeActive() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsThemeBackgroundPartiallyTransparent(htheme: isize, ipartid: i32, istateid: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsThemeDialogTextureEnabled(hwnd: super::super::Foundation::HWND) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsThemePartDefined(htheme: isize, ipartid: i32, istateid: i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn LBItemFromPt(hlb: super::super::Foundation::HWND, pt: super::super::Foundation::POINT, bautoscroll: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn LoadIconMetric(hinst: super::super::Foundation::HINSTANCE, pszname: ::windows_sys::core::PCWSTR, lims: _LI_METRIC, phico: *mut super::WindowsAndMessaging::HICON) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn LoadIconWithScaleDown(hinst: super::super::Foundation::HINSTANCE, pszname: ::windows_sys::core::PCWSTR, cx: i32, cy: i32, phico: *mut super::WindowsAndMessaging::HICON) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn MakeDragList(hlb: super::super::Foundation::HWND) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn MenuHelp(umsg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM, hmainmenu: super::WindowsAndMessaging::HMENU, hinst: super::super::Foundation::HINSTANCE, hwndstatus: super::super::Foundation::HWND, lpwids: *const u32);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenThemeData(hwnd: super::super::Foundation::HWND, pszclasslist: ::windows_sys::core::PCWSTR) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OpenThemeDataEx(hwnd: super::super::Foundation::HWND, pszclasslist: ::windows_sys::core::PCWSTR, dwflags: OPEN_THEME_DATA_FLAGS) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PackTouchHitTestingProximityEvaluation(phittestinginput: *const TOUCH_HIT_TESTING_INPUT, pproximityeval: *const TOUCH_HIT_TESTING_PROXIMITY_EVALUATION) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn PropertySheetA(param0: *mut PROPSHEETHEADERA_V2) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn PropertySheetW(param0: *mut PROPSHEETHEADERW_V2) -> isize;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterPointerDeviceNotifications(window: super::super::Foundation::HWND, notifyrange: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterTouchHitTestingWindow(hwnd: super::super::Foundation::HWND, value: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SetScrollInfo(hwnd: super::super::Foundation::HWND, nbar: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, lpsi: *const super::WindowsAndMessaging::SCROLLINFO, redraw: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SetScrollPos(hwnd: super::super::Foundation::HWND, nbar: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, npos: i32, bredraw: super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn SetScrollRange(hwnd: super::super::Foundation::HWND, nbar: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, nminpos: i32, nmaxpos: i32, bredraw: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
    pub fn SetThemeAppProperties(dwflags: u32);
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetWindowFeedbackSetting(hwnd: super::super::Foundation::HWND, feedback: FEEDBACK_TYPE, dwflags: u32, size: u32, configuration: *const ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetWindowTheme(hwnd: super::super::Foundation::HWND, pszsubappname: ::windows_sys::core::PCWSTR, pszsubidlist: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetWindowThemeAttribute(hwnd: super::super::Foundation::HWND, eattribute: WINDOWTHEMEATTRIBUTETYPE, pvattribute: *const ::core::ffi::c_void, cbattribute: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ShowHideMenuCtl(hwnd: super::super::Foundation::HWND, uflags: usize, lpinfo: *const i32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn ShowScrollBar(hwnd: super::super::Foundation::HWND, wbar: super::WindowsAndMessaging::SCROLLBAR_CONSTANTS, bshow: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn Str_SetPtrW(ppsz: *mut ::windows_sys::core::PWSTR, psz: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TaskDialog(hwndowner: super::super::Foundation::HWND, hinstance: super::super::Foundation::HINSTANCE, pszwindowtitle: ::windows_sys::core::PCWSTR, pszmaininstruction: ::windows_sys::core::PCWSTR, pszcontent: ::windows_sys::core::PCWSTR, dwcommonbuttons: TASKDIALOG_COMMON_BUTTON_FLAGS, pszicon: ::windows_sys::core::PCWSTR, pnbutton: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn TaskDialogIndirect(ptaskconfig: *const TASKDIALOGCONFIG, pnbutton: *mut i32, pnradiobutton: *mut i32, pfverificationflagchecked: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UninitializeFlatSB(param0: super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UpdatePanningFeedback(hwnd: super::super::Foundation::HWND, ltotaloverpanoffsetx: i32, ltotaloverpanoffsety: i32, fininertia: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACM_ISPLAYING: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACM_OPEN: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACM_OPENA: u32 = 1124u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACM_OPENW: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACM_PLAY: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACM_STOP: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACN_START: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACN_STOP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACS_AUTOPLAY: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACS_CENTER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACS_TIMER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ACS_TRANSPARENT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ANIMATE_CLASS: &str = "SysAnimate32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ANIMATE_CLASSA: &str = "SysAnimate32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ANIMATE_CLASSW: &str = "SysAnimate32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_FIRST: u32 = 5632u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_GETIDEALSIZE: u32 = 5633u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_GETIMAGELIST: u32 = 5635u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_GETNOTE: u32 = 5642u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_GETNOTELENGTH: u32 = 5643u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_GETSPLITINFO: u32 = 5640u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_GETTEXTMARGIN: u32 = 5637u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_SETDROPDOWNSTATE: u32 = 5638u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_SETIMAGELIST: u32 = 5634u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_SETNOTE: u32 = 5641u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_SETSHIELD: u32 = 5644u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_SETSPLITINFO: u32 = 5639u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCM_SETTEXTMARGIN: u32 = 5636u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCN_DROPDOWN: u32 = 4294966048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCN_FIRST: u32 = 4294966046u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCN_HOTITEMCHANGE: u32 = 4294966047u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSIF_GLYPH: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSIF_IMAGE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSIF_SIZE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSIF_STYLE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSS_ALIGNLEFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSS_IMAGE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSS_NOSPLIT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BCSS_STRETCH: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type BGTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BT_IMAGEFILE: BGTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BT_BORDERFILL: BGTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BT_NONE: BGTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type BORDERTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BT_RECT: BORDERTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BT_ROUNDRECT: BORDERTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BT_ELLIPSE: BORDERTYPE = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct BP_ANIMATIONPARAMS {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub style: BP_ANIMATIONSTYLE,
    pub dwDuration: u32,
}
impl ::core::marker::Copy for BP_ANIMATIONPARAMS {}
impl ::core::clone::Clone for BP_ANIMATIONPARAMS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type BP_ANIMATIONSTYLE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPAS_NONE: BP_ANIMATIONSTYLE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPAS_LINEAR: BP_ANIMATIONSTYLE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPAS_CUBIC: BP_ANIMATIONSTYLE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPAS_SINE: BP_ANIMATIONSTYLE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type BP_BUFFERFORMAT = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPBF_COMPATIBLEBITMAP: BP_BUFFERFORMAT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPBF_DIB: BP_BUFFERFORMAT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPBF_TOPDOWNDIB: BP_BUFFERFORMAT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPBF_TOPDOWNMONODIB: BP_BUFFERFORMAT = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct BP_PAINTPARAMS {
    pub cbSize: u32,
    pub dwFlags: BP_PAINTPARAMS_FLAGS,
    pub prcExclude: *const super::super::Foundation::RECT,
    pub pBlendFunction: *const super::super::Graphics::Gdi::BLENDFUNCTION,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for BP_PAINTPARAMS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for BP_PAINTPARAMS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type BP_PAINTPARAMS_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPPF_ERASE: BP_PAINTPARAMS_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPPF_NOCLIP: BP_PAINTPARAMS_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BPPF_NONCLIENT: BP_PAINTPARAMS_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BST_DROPDOWNPUSHED: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BST_HOT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BS_COMMANDLINK: i32 = 14i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BS_DEFCOMMANDLINK: i32 = 15i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BS_DEFSPLITBUTTON: i32 = 13i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BS_SPLITBUTTON: i32 = 12i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_AUTOSIZE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_BUTTON: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_CHECK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_DROPDOWN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_GROUP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_NOPREFIX: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_SEP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_SHOWTEXT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BTNS_WHOLEDROPDOWN: u32 = 128u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct BUTTON_IMAGELIST {
    pub himl: HIMAGELIST,
    pub margin: super::super::Foundation::RECT,
    pub uAlign: BUTTON_IMAGELIST_ALIGN,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for BUTTON_IMAGELIST {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for BUTTON_IMAGELIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type BUTTON_IMAGELIST_ALIGN = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BUTTON_IMAGELIST_ALIGN_LEFT: BUTTON_IMAGELIST_ALIGN = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BUTTON_IMAGELIST_ALIGN_RIGHT: BUTTON_IMAGELIST_ALIGN = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BUTTON_IMAGELIST_ALIGN_TOP: BUTTON_IMAGELIST_ALIGN = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BUTTON_IMAGELIST_ALIGN_BOTTOM: BUTTON_IMAGELIST_ALIGN = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BUTTON_IMAGELIST_ALIGN_CENTER: BUTTON_IMAGELIST_ALIGN = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct BUTTON_SPLITINFO {
    pub mask: u32,
    pub himlGlyph: HIMAGELIST,
    pub uSplitStyle: u32,
    pub size: super::super::Foundation::SIZE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for BUTTON_SPLITINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for BUTTON_SPLITINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEMAXSTRLEN: u32 = 260u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETCOMBOCONTROL: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETEDITCONTROL: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETEXSTYLE: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETEXTENDEDSTYLE: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETIMAGELIST: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETITEM: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETITEMA: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETITEMW: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_HASEDITCHANGED: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_INSERTITEM: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_INSERTITEMA: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_INSERTITEMW: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETEXSTYLE: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETEXTENDEDSTYLE: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETIMAGELIST: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETITEM: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETITEMA: u32 = 1029u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETITEMW: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEM_SETWINDOWTHEME: u32 = 8203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBENF_DROPDOWN: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBENF_ESCAPE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBENF_KILLFOCUS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBENF_RETURN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBES_EX_CASESENSITIVE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBES_EX_NOEDITIMAGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBES_EX_NOEDITIMAGEINDENT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBES_EX_NOSIZELIMIT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBES_EX_PATHWORDBREAKPROC: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBES_EX_TEXTENDELLIPSIS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBM_FIRST: u32 = 5888u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CB_GETCUEBANNER: u32 = 5892u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CB_GETMINVISIBLE: u32 = 5890u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CB_SETCUEBANNER: u32 = 5891u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CB_SETMINVISIBLE: u32 = 5889u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCF_NOTEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCHCCCLASS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCHCCDESC: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCHCCTEXT: u32 = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct CCINFOA {
    pub szClass: [super::super::Foundation::CHAR; 32],
    pub flOptions: u32,
    pub szDesc: [super::super::Foundation::CHAR; 32],
    pub cxDefault: u32,
    pub cyDefault: u32,
    pub flStyleDefault: u32,
    pub flExtStyleDefault: u32,
    pub flCtrlTypeMask: u32,
    pub szTextDefault: [super::super::Foundation::CHAR; 256],
    pub cStyleFlags: i32,
    pub aStyleFlags: *mut CCSTYLEFLAGA,
    pub lpfnStyle: LPFNCCSTYLEA,
    pub lpfnSizeToText: LPFNCCSIZETOTEXTA,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for CCINFOA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for CCINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct CCINFOW {
    pub szClass: [u16; 32],
    pub flOptions: u32,
    pub szDesc: [u16; 32],
    pub cxDefault: u32,
    pub cyDefault: u32,
    pub flStyleDefault: u32,
    pub flExtStyleDefault: u32,
    pub flCtrlTypeMask: u32,
    pub cStyleFlags: i32,
    pub aStyleFlags: *mut CCSTYLEFLAGW,
    pub szTextDefault: [u16; 256],
    pub lpfnStyle: LPFNCCSTYLEW,
    pub lpfnSizeToText: LPFNCCSIZETOTEXTW,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for CCINFOW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for CCINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_DPISCALE: u32 = 8204u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_FIRST: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_GETCOLORSCHEME: u32 = 8195u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_GETDROPTARGET: u32 = 8196u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_GETVERSION: u32 = 8200u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_LAST: u32 = 8704u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_SETBKCOLOR: u32 = 8193u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_SETCOLORSCHEME: u32 = 8194u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_SETNOTIFYWINDOW: u32 = 8201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_SETVERSION: u32 = 8199u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCM_SETWINDOWTHEME: u32 = 8203u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct CCSTYLEA {
    pub flStyle: u32,
    pub flExtStyle: u32,
    pub szText: [super::super::Foundation::CHAR; 256],
    pub lgid: u16,
    pub wReserved1: u16,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for CCSTYLEA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for CCSTYLEA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct CCSTYLEFLAGA {
    pub flStyle: u32,
    pub flStyleMask: u32,
    pub pszStyle: ::windows_sys::core::PSTR,
}
impl ::core::marker::Copy for CCSTYLEFLAGA {}
impl ::core::clone::Clone for CCSTYLEFLAGA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct CCSTYLEFLAGW {
    pub flStyle: u32,
    pub flStyleMask: u32,
    pub pszStyle: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for CCSTYLEFLAGW {}
impl ::core::clone::Clone for CCSTYLEFLAGW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct CCSTYLEW {
    pub flStyle: u32,
    pub flExtStyle: u32,
    pub szText: [u16; 256],
    pub lgid: u16,
    pub wReserved1: u16,
}
impl ::core::marker::Copy for CCSTYLEW {}
impl ::core::clone::Clone for CCSTYLEW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_ADJUSTABLE: i32 = 32i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_BOTTOM: i32 = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_NODIVIDER: i32 = 64i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_NOMOVEY: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_NOPARENTALIGN: i32 = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_NORESIZE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_TOP: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CCS_VERT: i32 = 128i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_ITEM: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_POSTERASE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_CHECKED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_DEFAULT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_DISABLED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_DROPHILITED: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_FOCUS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_GRAYED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_HOT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_INDETERMINATE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_MARKED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_NEARHOT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_OTHERSIDEHOT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_SELECTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDIS_SHOWKEYBOARDCUES: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_DODEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_DOERASE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_NEWFONT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_NOTIFYITEMDRAW: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_NOTIFYPOSTERASE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_NOTIFYPOSTPAINT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_NOTIFYSUBITEMDRAW: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_SKIPDEFAULT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDRF_SKIPPOSTPAINT: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type CLOCKPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLP_TIME: CLOCKPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type CLOCKSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLS_NORMAL: CLOCKSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLS_HOT: CLOCKSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLS_PRESSED: CLOCKSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLR_DEFAULT: i32 = -16777216i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLR_HILIGHT: i32 = -16777216i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CLR_NONE: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CMB_MASKED: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct COLORMAP {
    pub from: u32,
    pub to: u32,
}
impl ::core::marker::Copy for COLORMAP {}
impl ::core::clone::Clone for COLORMAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const COLORMGMTDLGORD: u32 = 1551u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct COLORSCHEME {
    pub dwSize: u32,
    pub clrBtnHighlight: u32,
    pub clrBtnShadow: u32,
}
impl ::core::marker::Copy for COLORSCHEME {}
impl ::core::clone::Clone for COLORSCHEME {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct COMBOBOXEXITEMA {
    pub mask: COMBOBOX_EX_ITEM_FLAGS,
    pub iItem: isize,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub iOverlay: i32,
    pub iIndent: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for COMBOBOXEXITEMA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for COMBOBOXEXITEMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct COMBOBOXEXITEMW {
    pub mask: COMBOBOX_EX_ITEM_FLAGS,
    pub iItem: isize,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub iOverlay: i32,
    pub iIndent: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for COMBOBOXEXITEMW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for COMBOBOXEXITEMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct COMBOBOXINFO {
    pub cbSize: u32,
    pub rcItem: super::super::Foundation::RECT,
    pub rcButton: super::super::Foundation::RECT,
    pub stateButton: COMBOBOXINFO_BUTTON_STATE,
    pub hwndCombo: super::super::Foundation::HWND,
    pub hwndItem: super::super::Foundation::HWND,
    pub hwndList: super::super::Foundation::HWND,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for COMBOBOXINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for COMBOBOXINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type COMBOBOXINFO_BUTTON_STATE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATE_SYSTEM_INVISIBLE: COMBOBOXINFO_BUTTON_STATE = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATE_SYSTEM_PRESSED: COMBOBOXINFO_BUTTON_STATE = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATE_SYSTEM_FOCUSABLE: COMBOBOXINFO_BUTTON_STATE = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATE_SYSTEM_OFFSCREEN: COMBOBOXINFO_BUTTON_STATE = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATE_SYSTEM_UNAVAILABLE: COMBOBOXINFO_BUTTON_STATE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type COMBOBOX_EX_ITEM_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_DI_SETITEM: COMBOBOX_EX_ITEM_FLAGS = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_IMAGE: COMBOBOX_EX_ITEM_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_INDENT: COMBOBOX_EX_ITEM_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_LPARAM: COMBOBOX_EX_ITEM_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_OVERLAY: COMBOBOX_EX_ITEM_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_SELECTEDIMAGE: COMBOBOX_EX_ITEM_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CBEIF_TEXT: COMBOBOX_EX_ITEM_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const COMCTL32_VERSION: u32 = 6u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct COMPAREITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub hwndItem: super::super::Foundation::HWND,
    pub itemID1: u32,
    pub itemData1: usize,
    pub itemID2: u32,
    pub itemData2: usize,
    pub dwLocaleId: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for COMPAREITEMSTRUCT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for COMPAREITEMSTRUCT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type CONTENTALIGNMENT = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CA_LEFT: CONTENTALIGNMENT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CA_CENTER: CONTENTALIGNMENT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CA_RIGHT: CONTENTALIGNMENT = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DATETIMEPICKERINFO {
    pub cbSize: u32,
    pub rcCheck: super::super::Foundation::RECT,
    pub stateCheck: u32,
    pub rcButton: super::super::Foundation::RECT,
    pub stateButton: u32,
    pub hwndEdit: super::super::Foundation::HWND,
    pub hwndUD: super::super::Foundation::HWND,
    pub hwndDropDown: super::super::Foundation::HWND,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DATETIMEPICKERINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DATETIMEPICKERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DATETIMEPICK_CLASS: &str = "SysDateTimePick32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DATETIMEPICK_CLASSA: &str = "SysDateTimePick32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DATETIMEPICK_CLASSW: &str = "SysDateTimePick32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DA_ERR: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DA_LAST: u32 = 2147483647u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DELETEITEMSTRUCT {
    pub CtlType: DRAWITEMSTRUCT_CTL_TYPE,
    pub CtlID: u32,
    pub itemID: u32,
    pub hwndItem: super::super::Foundation::HWND,
    pub itemData: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DELETEITEMSTRUCT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DELETEITEMSTRUCT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type DLG_BUTTON_CHECK_STATE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BST_CHECKED: DLG_BUTTON_CHECK_STATE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BST_INDETERMINATE: DLG_BUTTON_CHECK_STATE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const BST_UNCHECKED: DLG_BUTTON_CHECK_STATE = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type DLG_DIR_LIST_FILE_TYPE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_ARCHIVE: DLG_DIR_LIST_FILE_TYPE = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_DIRECTORY: DLG_DIR_LIST_FILE_TYPE = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_DRIVES: DLG_DIR_LIST_FILE_TYPE = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_EXCLUSIVE: DLG_DIR_LIST_FILE_TYPE = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_HIDDEN: DLG_DIR_LIST_FILE_TYPE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_READONLY: DLG_DIR_LIST_FILE_TYPE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_READWRITE: DLG_DIR_LIST_FILE_TYPE = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_SYSTEM: DLG_DIR_LIST_FILE_TYPE = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DDL_POSTMSGS: DLG_DIR_LIST_FILE_TYPE = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_COPYCURSOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_CURSORSET: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_MOVECURSOR: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_STOPCURSOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type DPAMM_MESSAGE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAMM_MERGE: DPAMM_MESSAGE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAMM_DELETE: DPAMM_MESSAGE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAMM_INSERT: DPAMM_MESSAGE = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAM_INTERSECT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAM_NORMAL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAM_SORTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAM_UNION: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct DPASTREAMINFO {
    pub iPos: i32,
    pub pvItem: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for DPASTREAMINFO {}
impl ::core::clone::Clone for DPASTREAMINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAS_INSERTAFTER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAS_INSERTBEFORE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPAS_SORTED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPA_APPEND: u32 = 2147483647u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DPA_ERR: i32 = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DRAGLISTINFO {
    pub uNotification: DRAGLISTINFO_NOTIFICATION_FLAGS,
    pub hWnd: super::super::Foundation::HWND,
    pub ptCursor: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DRAGLISTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DRAGLISTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type DRAGLISTINFO_NOTIFICATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_BEGINDRAG: DRAGLISTINFO_NOTIFICATION_FLAGS = 1157u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_CANCELDRAG: DRAGLISTINFO_NOTIFICATION_FLAGS = 1160u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_DRAGGING: DRAGLISTINFO_NOTIFICATION_FLAGS = 1158u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DL_DROPPED: DRAGLISTINFO_NOTIFICATION_FLAGS = 1159u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DRAGLISTMSGSTRING: &str = "commctrl_DragListMsg";
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DRAWITEMSTRUCT {
    pub CtlType: DRAWITEMSTRUCT_CTL_TYPE,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemAction: u32,
    pub itemState: u32,
    pub hwndItem: super::super::Foundation::HWND,
    pub hDC: super::super::Graphics::Gdi::HDC,
    pub rcItem: super::super::Foundation::RECT,
    pub itemData: usize,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DRAWITEMSTRUCT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DRAWITEMSTRUCT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type DRAWITEMSTRUCT_CTL_TYPE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_BUTTON: DRAWITEMSTRUCT_CTL_TYPE = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_COMBOBOX: DRAWITEMSTRUCT_CTL_TYPE = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_LISTBOX: DRAWITEMSTRUCT_CTL_TYPE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_LISTVIEW: DRAWITEMSTRUCT_CTL_TYPE = 102u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_MENU: DRAWITEMSTRUCT_CTL_TYPE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_STATIC: DRAWITEMSTRUCT_CTL_TYPE = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_TAB: DRAWITEMSTRUCT_CTL_TYPE = 101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type DRAW_THEME_PARENT_BACKGROUND_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTPB_WINDOWDC: DRAW_THEME_PARENT_BACKGROUND_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTPB_USECTLCOLORSTATIC: DRAW_THEME_PARENT_BACKGROUND_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTPB_USEERASEBKGND: DRAW_THEME_PARENT_BACKGROUND_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DSA_APPEND: u32 = 2147483647u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DSA_ERR: i32 = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct DTBGOPTS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub rcClip: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for DTBGOPTS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for DTBGOPTS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_CLIPRECT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_COMPUTINGREGION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_DRAWSOLID: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_MIRRORDC: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_NOMIRROR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_OMITBORDER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTBG_OMITCONTENT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_CLOSEMONTHCAL: u32 = 4109u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_FIRST: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETDATETIMEPICKERINFO: u32 = 4110u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETIDEALSIZE: u32 = 4111u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETMCCOLOR: u32 = 4103u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETMCFONT: u32 = 4106u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETMCSTYLE: u32 = 4108u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETMONTHCAL: u32 = 4104u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETRANGE: u32 = 4099u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_GETSYSTEMTIME: u32 = 4097u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETFORMAT: u32 = 4146u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETFORMATA: u32 = 4101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETFORMATW: u32 = 4146u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETMCCOLOR: u32 = 4102u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETMCFONT: u32 = 4105u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETMCSTYLE: u32 = 4107u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETRANGE: u32 = 4100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTM_SETSYSTEMTIME: u32 = 4098u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_APPCANPARSE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_LONGDATEFORMAT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_RIGHTALIGN: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_SHORTDATECENTURYFORMAT: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_SHORTDATEFORMAT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_SHOWNONE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_TIMEFORMAT: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTS_UPDOWN: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct DTTOPTS {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub crText: u32,
    pub crBorder: u32,
    pub crShadow: u32,
    pub iTextShadowType: i32,
    pub ptShadowOffset: super::super::Foundation::POINT,
    pub iBorderSize: i32,
    pub iFontPropId: i32,
    pub iColorPropId: i32,
    pub iStateId: i32,
    pub fApplyOverlay: super::super::Foundation::BOOL,
    pub iGlowSize: i32,
    pub pfnDrawTextCallback: DTT_CALLBACK_PROC,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for DTTOPTS {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for DTTOPTS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type DTT_CALLBACK_PROC = ::core::option::Option<unsafe extern "system" fn(hdc: super::super::Graphics::Gdi::HDC, psztext: ::windows_sys::core::PWSTR, cchtext: i32, prc: *mut super::super::Foundation::RECT, dwflags: u32, lparam: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTT_FLAGS2VALIDBITS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const DTT_GRAYED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ECM_FIRST: u32 = 5376u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type EC_ENDOFLINE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EC_ENDOFLINE_DETECTFROMCONTENT: EC_ENDOFLINE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EC_ENDOFLINE_CRLF: EC_ENDOFLINE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EC_ENDOFLINE_CR: EC_ENDOFLINE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EC_ENDOFLINE_LF: EC_ENDOFLINE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type EC_SEARCHWEB_ENTRYPOINT = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EC_SEARCHWEB_ENTRYPOINT_EXTERNAL: EC_SEARCHWEB_ENTRYPOINT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EC_SEARCHWEB_ENTRYPOINT_CONTEXTMENU: EC_SEARCHWEB_ENTRYPOINT = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct EDITBALLOONTIP {
    pub cbStruct: u32,
    pub pszTitle: ::windows_sys::core::PCWSTR,
    pub pszText: ::windows_sys::core::PCWSTR,
    pub ttiIcon: EDITBALLOONTIP_ICON,
}
impl ::core::marker::Copy for EDITBALLOONTIP {}
impl ::core::clone::Clone for EDITBALLOONTIP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type EDITBALLOONTIP_ICON = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_ERROR: EDITBALLOONTIP_ICON = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_INFO: EDITBALLOONTIP_ICON = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_NONE: EDITBALLOONTIP_ICON = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_WARNING: EDITBALLOONTIP_ICON = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_INFO_LARGE: EDITBALLOONTIP_ICON = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_WARNING_LARGE: EDITBALLOONTIP_ICON = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTI_ERROR_LARGE: EDITBALLOONTIP_ICON = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type EDITWORDBREAKPROCA = ::core::option::Option<unsafe extern "system" fn(lpch: ::windows_sys::core::PCSTR, ichcurrent: i32, cch: i32, code: WORD_BREAK_ACTION) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type EDITWORDBREAKPROCW = ::core::option::Option<unsafe extern "system" fn(lpch: ::windows_sys::core::PCWSTR, ichcurrent: i32, cch: i32, code: WORD_BREAK_ACTION) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type EMPTYMARKUPPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EMP_MARKUPTEXT: EMPTYMARKUPPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_CANUNDO: u32 = 198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_CHARFROMPOS: u32 = 215u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_EMPTYUNDOBUFFER: u32 = 205u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_ENABLEFEATURE: u32 = 218u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_ENABLESEARCHWEB: u32 = 5390u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_FILELINEFROMCHAR: u32 = 5395u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_FILELINEINDEX: u32 = 5396u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_FILELINELENGTH: u32 = 5397u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_FMTLINES: u32 = 200u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETCARETINDEX: u32 = 5394u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETCUEBANNER: u32 = 5378u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETENDOFLINE: u32 = 5389u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETEXTENDEDSTYLE: u32 = 5387u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETFILELINE: u32 = 5398u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETFILELINECOUNT: u32 = 5399u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETFIRSTVISIBLELINE: u32 = 206u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETHANDLE: u32 = 189u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETHILITE: u32 = 5382u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETIMESTATUS: u32 = 217u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETLIMITTEXT: u32 = 213u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETLINE: u32 = 196u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETLINECOUNT: u32 = 186u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETMARGINS: u32 = 212u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETMODIFY: u32 = 184u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETPASSWORDCHAR: u32 = 210u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETRECT: u32 = 178u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETSEL: u32 = 176u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETTHUMB: u32 = 190u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_GETWORDBREAKPROC: u32 = 209u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_HIDEBALLOONTIP: u32 = 5380u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_LIMITTEXT: u32 = 197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_LINEFROMCHAR: u32 = 201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_LINEINDEX: u32 = 187u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_LINELENGTH: u32 = 193u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_LINESCROLL: u32 = 182u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_NOSETFOCUS: u32 = 5383u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_POSFROMCHAR: u32 = 214u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_REPLACESEL: u32 = 194u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SCROLL: u32 = 181u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SCROLLCARET: u32 = 183u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SEARCHWEB: u32 = 5391u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETCARETINDEX: u32 = 5393u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETCUEBANNER: u32 = 5377u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETENDOFLINE: u32 = 5388u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETEXTENDEDSTYLE: u32 = 5386u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETHANDLE: u32 = 188u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETHILITE: u32 = 5381u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETIMESTATUS: u32 = 216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETLIMITTEXT: u32 = 197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETMARGINS: u32 = 211u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETMODIFY: u32 = 185u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETPASSWORDCHAR: u32 = 204u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETREADONLY: u32 = 207u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETRECT: u32 = 179u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETRECTNP: u32 = 180u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETSEL: u32 = 177u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETTABSTOPS: u32 = 203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SETWORDBREAKPROC: u32 = 208u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_SHOWBALLOONTIP: u32 = 5379u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_TAKEFOCUS: u32 = 5384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EM_UNDO: u32 = 199u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type ENABLE_SCROLL_BAR_ARROWS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_BOTH: ENABLE_SCROLL_BAR_ARROWS = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_DOWN: ENABLE_SCROLL_BAR_ARROWS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_LEFT: ENABLE_SCROLL_BAR_ARROWS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_LTUP: ENABLE_SCROLL_BAR_ARROWS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_RIGHT: ENABLE_SCROLL_BAR_ARROWS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_RTDN: ENABLE_SCROLL_BAR_ARROWS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_DISABLE_UP: ENABLE_SCROLL_BAR_ARROWS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ESB_ENABLE_BOTH: ENABLE_SCROLL_BAR_ARROWS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ES_EX_ALLOWEOL_CR: i32 = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ES_EX_ALLOWEOL_LF: i32 = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ES_EX_CONVERT_EOL_ON_PASTE: i32 = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ES_EX_ZOOMABLE: i32 = 16i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ETDT_DISABLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ETDT_ENABLE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ETDT_USEAEROWIZARDTABTEXTURE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ETDT_USETABTEXTURE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type FEEDBACK_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_TOUCH_CONTACTVISUALIZATION: FEEDBACK_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_PEN_BARRELVISUALIZATION: FEEDBACK_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_PEN_TAP: FEEDBACK_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_PEN_DOUBLETAP: FEEDBACK_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_PEN_PRESSANDHOLD: FEEDBACK_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_PEN_RIGHTTAP: FEEDBACK_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_TOUCH_TAP: FEEDBACK_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_TOUCH_DOUBLETAP: FEEDBACK_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_TOUCH_PRESSANDHOLD: FEEDBACK_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_TOUCH_RIGHTTAP: FEEDBACK_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_GESTURE_PRESSANDTAP: FEEDBACK_TYPE = 11i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FEEDBACK_MAX: FEEDBACK_TYPE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FILEOPENORD: u32 = 1536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type FILLTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FT_SOLID: FILLTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FT_VERTGRADIENT: FILLTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FT_HORZGRADIENT: FILLTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FT_RADIALGRADIENT: FILLTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FT_TILEIMAGE: FILLTYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FINDDLGORD: u32 = 1540u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FONTDLGORD: u32 = 1542u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FORMATDLGORD30: u32 = 1544u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FORMATDLGORD31: u32 = 1543u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FSB_ENCARTA_MODE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FSB_FLAT_MODE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const FSB_REGULAR_MODE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GDTR_MAX: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GDTR_MIN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GDT_ERROR: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GDT_NONE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GDT_VALID: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type GET_THEME_BITMAP_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GBF_DIRECT: GET_THEME_BITMAP_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GBF_COPY: GET_THEME_BITMAP_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GBF_VALIDBITS: GET_THEME_BITMAP_FLAGS = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type GLYPHFONTSIZINGTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GFST_NONE: GLYPHFONTSIZINGTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GFST_SIZE: GLYPHFONTSIZINGTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GFST_DPI: GLYPHFONTSIZINGTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type GLYPHTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GT_NONE: GLYPHTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GT_IMAGEGLYPH: GLYPHTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GT_FONTGLYPH: GLYPHTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GMR_DAYSTATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const GMR_VISIBLE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type GRIDCELLBACKGROUNDSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCB_SELECTED: GRIDCELLBACKGROUNDSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCB_HOT: GRIDCELLBACKGROUNDSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCB_SELECTEDHOT: GRIDCELLBACKGROUNDSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCB_SELECTEDNOTFOCUSED: GRIDCELLBACKGROUNDSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCB_TODAY: GRIDCELLBACKGROUNDSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCB_TODAYSELECTED: GRIDCELLBACKGROUNDSTATES = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type GRIDCELLSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_HOT: GRIDCELLSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_HASSTATE: GRIDCELLSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_HASSTATEHOT: GRIDCELLSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_TODAY: GRIDCELLSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_TODAYSELECTED: GRIDCELLSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_SELECTED: GRIDCELLSTATES = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGC_SELECTEDHOT: GRIDCELLSTATES = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type GRIDCELLUPPERSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCU_HOT: GRIDCELLUPPERSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCU_HASSTATE: GRIDCELLUPPERSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCU_HASSTATEHOT: GRIDCELLUPPERSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCU_SELECTED: GRIDCELLUPPERSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGCU_SELECTEDHOT: GRIDCELLUPPERSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type HALIGN = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HA_LEFT: HALIGN = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HA_CENTER: HALIGN = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HA_RIGHT: HALIGN = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDFT_HASNOVALUE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDFT_ISDATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDFT_ISNUMBER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDFT_ISSTRING: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_BITMAP: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_BITMAP_ON_RIGHT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_CENTER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_CHECKBOX: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_CHECKED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_FIXEDWIDTH: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_IMAGE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_JUSTIFYMASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_LEFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_OWNERDRAW: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_RIGHT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_RTLREADING: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_SORTDOWN: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_SORTUP: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_SPLITBUTTON: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDF_STRING: u32 = 16384u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct HDHITTESTINFO {
    pub pt: super::super::Foundation::POINT,
    pub flags: u32,
    pub iItem: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for HDHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for HDHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDIS_FOCUSED: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct HDITEMA {
    pub mask: HDI_MASK,
    pub cxy: i32,
    pub pszText: ::windows_sys::core::PSTR,
    pub hbm: super::super::Graphics::Gdi::HBITMAP,
    pub cchTextMax: i32,
    pub fmt: i32,
    pub lParam: super::super::Foundation::LPARAM,
    pub iImage: i32,
    pub iOrder: i32,
    pub r#type: u32,
    pub pvFilter: *mut ::core::ffi::c_void,
    pub state: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for HDITEMA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for HDITEMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct HDITEMW {
    pub mask: HDI_MASK,
    pub cxy: i32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub hbm: super::super::Graphics::Gdi::HBITMAP,
    pub cchTextMax: i32,
    pub fmt: i32,
    pub lParam: super::super::Foundation::LPARAM,
    pub iImage: i32,
    pub iOrder: i32,
    pub r#type: u32,
    pub pvFilter: *mut ::core::ffi::c_void,
    pub state: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for HDITEMW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for HDITEMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type HDI_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_WIDTH: HDI_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_HEIGHT: HDI_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_TEXT: HDI_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_FORMAT: HDI_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_LPARAM: HDI_MASK = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_BITMAP: HDI_MASK = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_IMAGE: HDI_MASK = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_DI_SETITEM: HDI_MASK = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_ORDER: HDI_MASK = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_FILTER: HDI_MASK = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDI_STATE: HDI_MASK = 512u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct HDLAYOUT {
    pub prc: *mut super::super::Foundation::RECT,
    pub pwpos: *mut super::WindowsAndMessaging::WINDOWPOS,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for HDLAYOUT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for HDLAYOUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_CLEARFILTER: u32 = 4632u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_CREATEDRAGIMAGE: u32 = 4624u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_DELETEITEM: u32 = 4610u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_EDITFILTER: u32 = 4631u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_FIRST: u32 = 4608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETBITMAPMARGIN: u32 = 4629u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETFOCUSEDITEM: u32 = 4635u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETIMAGELIST: u32 = 4617u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETITEM: u32 = 4619u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETITEMA: u32 = 4611u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETITEMCOUNT: u32 = 4608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETITEMDROPDOWNRECT: u32 = 4633u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETITEMRECT: u32 = 4615u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETITEMW: u32 = 4619u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETORDERARRAY: u32 = 4625u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETOVERFLOWRECT: u32 = 4634u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_HITTEST: u32 = 4614u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_INSERTITEM: u32 = 4618u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_INSERTITEMA: u32 = 4609u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_INSERTITEMW: u32 = 4618u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_LAYOUT: u32 = 4613u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_ORDERTOINDEX: u32 = 4623u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETBITMAPMARGIN: u32 = 4628u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETFILTERCHANGETIMEOUT: u32 = 4630u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETFOCUSEDITEM: u32 = 4636u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETHOTDIVIDER: u32 = 4627u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETIMAGELIST: u32 = 4616u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETITEM: u32 = 4620u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETITEMA: u32 = 4612u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETITEMW: u32 = 4620u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETORDERARRAY: u32 = 4626u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDM_SETUNICODEFORMAT: u32 = 8197u32;
pub type HDPA = isize;
pub type HDSA = isize;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDSIL_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDSIL_STATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_BUTTONS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_CHECKBOXES: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_DRAGDROP: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_FILTERBAR: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_FLAT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_FULLDRAG: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_HIDDEN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_HORZ: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_HOTTRACK: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_NOSIZING: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HDS_OVERFLOW: u32 = 4096u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct HD_TEXTFILTERA {
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
}
impl ::core::marker::Copy for HD_TEXTFILTERA {}
impl ::core::clone::Clone for HD_TEXTFILTERA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct HD_TEXTFILTERW {
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
}
impl ::core::marker::Copy for HD_TEXTFILTERW {}
impl ::core::clone::Clone for HD_TEXTFILTERW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type HEADER_CONTROL_NOTIFICATION_BUTTON = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HEADER_CONTROL_NOTIFICATION_BUTTON_LEFT: HEADER_CONTROL_NOTIFICATION_BUTTON = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HEADER_CONTROL_NOTIFICATION_BUTTON_RIGHT: HEADER_CONTROL_NOTIFICATION_BUTTON = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HEADER_CONTROL_NOTIFICATION_BUTTON_MIDDLE: HEADER_CONTROL_NOTIFICATION_BUTTON = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ABOVE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_BELOW: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_NOWHERE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONDIVIDER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONDIVOPEN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONDROPDOWN: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONFILTER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONFILTERBUTTON: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONHEADER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONITEMSTATEICON: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_ONOVERFLOW: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_TOLEFT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HHT_TORIGHT: u32 = 1024u32;
pub type HIMAGELIST = isize;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HIST_ADDTOFAVORITES: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HIST_BACK: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HIST_FAVORITES: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HIST_FORWARD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HIST_VIEWTREE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_A: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_C: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_CA: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_NONE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_S: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_SA: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_SC: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKCOMB_SCA: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKM_GETHOTKEY: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKM_SETHOTKEY: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HKM_SETRULES: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEYF_ALT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEYF_CONTROL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEYF_EXT: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEYF_SHIFT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEY_CLASS: &str = "msctls_hotkey32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEY_CLASSA: &str = "msctls_hotkey32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOTKEY_CLASSW: &str = "msctls_hotkey32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HOVER_DEFAULT: u32 = 4294967295u32;
pub type HPROPSHEETPAGE = isize;
pub type HSYNTHETICPOINTERDEVICE = isize;
pub type HTREEITEM = isize;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_BACKGROUNDSEG: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_CAPTION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_FIXEDBORDER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_RESIZINGBORDER_BOTTOM: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_RESIZINGBORDER_LEFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_RESIZINGBORDER_RIGHT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_RESIZINGBORDER_TOP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_SIZINGTEMPLATE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HTTB_SYSTEMSIZINGMARGINS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type HYPERLINKSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HLS_NORMALTEXT: HYPERLINKSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HLS_LINKTEXT: HYPERLINKSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type ICONEFFECT = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICE_NONE: ICONEFFECT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICE_GLOW: ICONEFFECT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICE_SHADOW: ICONEFFECT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICE_PULSE: ICONEFFECT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICE_ALPHA: ICONEFFECT = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_HIST_DISABLED: u32 = 14u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_HIST_HOT: u32 = 13u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_HIST_LARGE_COLOR: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_HIST_NORMAL: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_HIST_PRESSED: u32 = 15u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_HIST_SMALL_COLOR: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_STD_LARGE_COLOR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_STD_SMALL_COLOR: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_VIEW_LARGE_COLOR: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDB_VIEW_SMALL_COLOR: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IDC_MANAGE_LINK: u32 = 1592u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ID_PSRESTARTWINDOWS: u32 = 2u32;
pub type IImageList = *mut ::core::ffi::c_void;
pub type IImageList2 = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILDI_PURGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILDI_QUERYACCESS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILDI_RESETACCESS: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILDI_STANDBY: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILDRF_IMAGELOWQUALITY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILDRF_OVERLAYLOWQUALITY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_ASYNC: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_BLEND25: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_DPISCALE: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_IMAGE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_OVERLAYMASK: u32 = 3840u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_PRESERVEALPHA: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_ROP: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_SCALE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_TRANSPARENT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILFIP_ALWAYS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILFIP_FROMSTANDBY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILGOS_ALWAYS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILGOS_FROMSTANDBY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILGT_ASYNC: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILGT_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILP_DOWNLEVEL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILP_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_HORIZONTAL_CENTER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_HORIZONTAL_LEFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_HORIZONTAL_RIGHT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_SCALE_ASPECTRATIO: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_SCALE_CLIP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_VERTICAL_BOTTOM: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_VERTICAL_CENTER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILR_VERTICAL_TOP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILS_ALPHA: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILS_GLOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILS_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILS_SATURATE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILS_SHADOW: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct IMAGEINFO {
    pub hbmImage: super::super::Graphics::Gdi::HBITMAP,
    pub hbmMask: super::super::Graphics::Gdi::HBITMAP,
    pub Unused1: i32,
    pub Unused2: i32,
    pub rcImage: super::super::Foundation::RECT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for IMAGEINFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for IMAGEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type IMAGELAYOUT = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IL_VERTICAL: IMAGELAYOUT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IL_HORIZONTAL: IMAGELAYOUT = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct IMAGELISTDRAWPARAMS {
    pub cbSize: u32,
    pub himl: HIMAGELIST,
    pub i: i32,
    pub hdcDst: super::super::Graphics::Gdi::HDC,
    pub x: i32,
    pub y: i32,
    pub cx: i32,
    pub cy: i32,
    pub xBitmap: i32,
    pub yBitmap: i32,
    pub rgbBk: u32,
    pub rgbFg: u32,
    pub fStyle: u32,
    pub dwRop: u32,
    pub fState: u32,
    pub Frame: u32,
    pub crEffect: u32,
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for IMAGELISTDRAWPARAMS {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for IMAGELISTDRAWPARAMS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct IMAGELISTSTATS {
    pub cbSize: u32,
    pub cAlloc: i32,
    pub cUsed: i32,
    pub cStandby: i32,
}
impl ::core::marker::Copy for IMAGELISTSTATS {}
impl ::core::clone::Clone for IMAGELISTSTATS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type IMAGELIST_CREATION_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_MASK: IMAGELIST_CREATION_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLOR: IMAGELIST_CREATION_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLORDDB: IMAGELIST_CREATION_FLAGS = 254u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLOR4: IMAGELIST_CREATION_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLOR8: IMAGELIST_CREATION_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLOR16: IMAGELIST_CREATION_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLOR24: IMAGELIST_CREATION_FLAGS = 24u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_COLOR32: IMAGELIST_CREATION_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_PALETTE: IMAGELIST_CREATION_FLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_MIRROR: IMAGELIST_CREATION_FLAGS = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_PERITEMMIRROR: IMAGELIST_CREATION_FLAGS = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_ORIGINALSIZE: IMAGELIST_CREATION_FLAGS = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILC_HIGHQUALITYSCALE: IMAGELIST_CREATION_FLAGS = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type IMAGESELECTTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IST_NONE: IMAGESELECTTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IST_SIZE: IMAGESELECTTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IST_DPI: IMAGESELECTTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type IMAGE_LIST_COPY_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILCF_MOVE: IMAGE_LIST_COPY_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILCF_SWAP: IMAGE_LIST_COPY_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type IMAGE_LIST_DRAW_STYLE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_BLEND: IMAGE_LIST_DRAW_STYLE = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_BLEND50: IMAGE_LIST_DRAW_STYLE = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_FOCUS: IMAGE_LIST_DRAW_STYLE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_MASK: IMAGE_LIST_DRAW_STYLE = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_NORMAL: IMAGE_LIST_DRAW_STYLE = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILD_SELECTED: IMAGE_LIST_DRAW_STYLE = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type IMAGE_LIST_ITEM_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILIF_ALPHA: IMAGE_LIST_ITEM_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ILIF_LOWQUALITY: IMAGE_LIST_ITEM_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const INFOTIPSIZE: u32 = 1024u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct INITCOMMONCONTROLSEX {
    pub dwSize: u32,
    pub dwICC: INITCOMMONCONTROLSEX_ICC,
}
impl ::core::marker::Copy for INITCOMMONCONTROLSEX {}
impl ::core::clone::Clone for INITCOMMONCONTROLSEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type INITCOMMONCONTROLSEX_ICC = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_ANIMATE_CLASS: INITCOMMONCONTROLSEX_ICC = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_BAR_CLASSES: INITCOMMONCONTROLSEX_ICC = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_COOL_CLASSES: INITCOMMONCONTROLSEX_ICC = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_DATE_CLASSES: INITCOMMONCONTROLSEX_ICC = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_HOTKEY_CLASS: INITCOMMONCONTROLSEX_ICC = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_INTERNET_CLASSES: INITCOMMONCONTROLSEX_ICC = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_LINK_CLASS: INITCOMMONCONTROLSEX_ICC = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_LISTVIEW_CLASSES: INITCOMMONCONTROLSEX_ICC = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_NATIVEFNTCTL_CLASS: INITCOMMONCONTROLSEX_ICC = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_PAGESCROLLER_CLASS: INITCOMMONCONTROLSEX_ICC = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_PROGRESS_CLASS: INITCOMMONCONTROLSEX_ICC = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_STANDARD_CLASSES: INITCOMMONCONTROLSEX_ICC = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_TAB_CLASSES: INITCOMMONCONTROLSEX_ICC = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_TREEVIEW_CLASSES: INITCOMMONCONTROLSEX_ICC = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_UPDOWN_CLASS: INITCOMMONCONTROLSEX_ICC = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_USEREX_CLASSES: INITCOMMONCONTROLSEX_ICC = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ICC_WIN95_CLASSES: INITCOMMONCONTROLSEX_ICC = 255u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct INTLIST {
    pub iValueCount: i32,
    pub iValues: [i32; 402],
}
impl ::core::marker::Copy for INTLIST {}
impl ::core::clone::Clone for INTLIST {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const INVALID_LINK_INDEX: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IPM_CLEARADDRESS: u32 = 1124u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IPM_GETADDRESS: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IPM_ISBLANK: u32 = 1129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IPM_SETADDRESS: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IPM_SETFOCUS: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const IPM_SETRANGE: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_IMAGECALLBACK: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_IMAGENONE: i32 = -2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_INDENTCALLBACK: i32 = -1i32;
pub const ImageList: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2085055394, data2: 689, data3: 18676, data4: [128, 72, 178, 70, 25, 221, 192, 88] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LHITTESTINFO {
    pub pt: super::super::Foundation::POINT,
    pub item: LITEM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIF_ITEMID: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIF_ITEMINDEX: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIF_STATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIF_URL: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LINKPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LP_HYPERLINK: LINKPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIS_DEFAULTCOLORS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIS_ENABLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIS_FOCUSED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIS_HOTTRACK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIS_VISITED: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LITEM {
    pub mask: u32,
    pub iLink: i32,
    pub state: u32,
    pub stateMask: u32,
    pub szID: [u16; 48],
    pub szUrl: [u16; 2084],
}
impl ::core::marker::Copy for LITEM {}
impl ::core::clone::Clone for LITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LM_GETIDEALHEIGHT: u32 = 1793u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LM_GETIDEALSIZE: u32 = 1793u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LM_GETITEM: u32 = 1795u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LM_HITTEST: u32 = 1792u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LM_SETITEM: u32 = 1794u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LOGOFFBUTTONSSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPLS_NORMAL: LOGOFFBUTTONSSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPLS_HOT: LOGOFFBUTTONSSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPLS_PRESSED: LOGOFFBUTTONSSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNADDPROPSHEETPAGES = ::core::option::Option<unsafe extern "system" fn(param0: *mut ::core::ffi::c_void, param1: LPFNSVADDPROPSHEETPAGE, param2: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPFNCCINFOA = ::core::option::Option<unsafe extern "system" fn(acci: *mut CCINFOA) -> u32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub type LPFNCCINFOW = ::core::option::Option<unsafe extern "system" fn(acci: *mut CCINFOW) -> u32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub type LPFNCCSIZETOTEXTA = ::core::option::Option<unsafe extern "system" fn(flstyle: u32, flextstyle: u32, hfont: super::super::Graphics::Gdi::HFONT, psztext: ::windows_sys::core::PCSTR) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub type LPFNCCSIZETOTEXTW = ::core::option::Option<unsafe extern "system" fn(flstyle: u32, flextstyle: u32, hfont: super::super::Graphics::Gdi::HFONT, psztext: ::windows_sys::core::PCWSTR) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNCCSTYLEA = ::core::option::Option<unsafe extern "system" fn(hwndparent: super::super::Foundation::HWND, pccs: *mut CCSTYLEA) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNCCSTYLEW = ::core::option::Option<unsafe extern "system" fn(hwndparent: super::super::Foundation::HWND, pccs: *mut CCSTYLEW) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub type LPFNPSPCALLBACKA = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, umsg: PSPCB_MESSAGE, ppsp: *mut PROPSHEETPAGEA) -> u32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub type LPFNPSPCALLBACKW = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, umsg: PSPCB_MESSAGE, ppsp: *mut PROPSHEETPAGEW) -> u32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNSVADDPROPSHEETPAGE = ::core::option::Option<unsafe extern "system" fn(param0: HPROPSHEETPAGE, param1: super::super::Foundation::LPARAM) -> super::super::Foundation::BOOL>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVA_ALIGNLEFT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVA_ALIGNTOP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVA_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVA_SNAPTOGRID: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_FLAG_ALPHABLEND: u32 = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_FLAG_TILEOFFSET: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_SOURCE_HBITMAP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_SOURCE_MASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_SOURCE_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_SOURCE_URL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_STYLE_MASK: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_STYLE_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_STYLE_TILE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVBKIF_TYPE_WATERMARK: u32 = 268435456u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct LVBKIMAGEA {
    pub ulFlags: u32,
    pub hbm: super::super::Graphics::Gdi::HBITMAP,
    pub pszImage: ::windows_sys::core::PSTR,
    pub cchImageMax: u32,
    pub xOffsetPercent: i32,
    pub yOffsetPercent: i32,
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for LVBKIMAGEA {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for LVBKIMAGEA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(feature = "Win32_Graphics_Gdi")]
pub struct LVBKIMAGEW {
    pub ulFlags: u32,
    pub hbm: super::super::Graphics::Gdi::HBITMAP,
    pub pszImage: ::windows_sys::core::PWSTR,
    pub cchImageMax: u32,
    pub xOffsetPercent: i32,
    pub yOffsetPercent: i32,
}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::marker::Copy for LVBKIMAGEW {}
#[cfg(feature = "Win32_Graphics_Gdi")]
impl ::core::clone::Clone for LVBKIMAGEW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCDRF_NOGROUPFRAME: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCDRF_NOSELECT: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_FILL: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_LINE_BREAK: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_NO_TITLE: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_WRAP: u32 = 4194304u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVCOLUMNA {
    pub mask: LVCOLUMNW_MASK,
    pub fmt: LVCOLUMNW_FORMAT,
    pub cx: i32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iSubItem: i32,
    pub iImage: i32,
    pub iOrder: i32,
    pub cxMin: i32,
    pub cxDefault: i32,
    pub cxIdeal: i32,
}
impl ::core::marker::Copy for LVCOLUMNA {}
impl ::core::clone::Clone for LVCOLUMNA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVCOLUMNW {
    pub mask: LVCOLUMNW_MASK,
    pub fmt: LVCOLUMNW_FORMAT,
    pub cx: i32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iSubItem: i32,
    pub iImage: i32,
    pub iOrder: i32,
    pub cxMin: i32,
    pub cxDefault: i32,
    pub cxIdeal: i32,
}
impl ::core::marker::Copy for LVCOLUMNW {}
impl ::core::clone::Clone for LVCOLUMNW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVCOLUMNW_FORMAT = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_LEFT: LVCOLUMNW_FORMAT = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_RIGHT: LVCOLUMNW_FORMAT = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_CENTER: LVCOLUMNW_FORMAT = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_JUSTIFYMASK: LVCOLUMNW_FORMAT = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_IMAGE: LVCOLUMNW_FORMAT = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_BITMAP_ON_RIGHT: LVCOLUMNW_FORMAT = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_COL_HAS_IMAGES: LVCOLUMNW_FORMAT = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_FIXED_WIDTH: LVCOLUMNW_FORMAT = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_NO_DPI_SCALE: LVCOLUMNW_FORMAT = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_FIXED_RATIO: LVCOLUMNW_FORMAT = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCFMT_SPLITBUTTON: LVCOLUMNW_FORMAT = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVCOLUMNW_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_FMT: LVCOLUMNW_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_WIDTH: LVCOLUMNW_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_TEXT: LVCOLUMNW_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_SUBITEM: LVCOLUMNW_MASK = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_IMAGE: LVCOLUMNW_MASK = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_ORDER: LVCOLUMNW_MASK = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_MINWIDTH: LVCOLUMNW_MASK = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_DEFAULTWIDTH: LVCOLUMNW_MASK = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCF_IDEALWIDTH: LVCOLUMNW_MASK = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFF_ITEMCOUNT: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LVFINDINFOA {
    pub flags: LVFINDINFOW_FLAGS,
    pub psz: ::windows_sys::core::PCSTR,
    pub lParam: super::super::Foundation::LPARAM,
    pub pt: super::super::Foundation::POINT,
    pub vkDirection: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LVFINDINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LVFINDINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LVFINDINFOW {
    pub flags: LVFINDINFOW_FLAGS,
    pub psz: ::windows_sys::core::PCWSTR,
    pub lParam: super::super::Foundation::LPARAM,
    pub pt: super::super::Foundation::POINT,
    pub vkDirection: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LVFINDINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LVFINDINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVFINDINFOW_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFI_PARAM: LVFINDINFOW_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFI_PARTIAL: LVFINDINFOW_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFI_STRING: LVFINDINFOW_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFI_SUBSTRING: LVFINDINFOW_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFI_WRAP: LVFINDINFOW_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFI_NEARESTXY: LVFINDINFOW_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFIS_FOCUSED: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVFOOTERINFO {
    pub mask: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub cItems: u32,
}
impl ::core::marker::Copy for LVFOOTERINFO {}
impl ::core::clone::Clone for LVFOOTERINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVFOOTERITEM {
    pub mask: LVFOOTERITEM_MASK,
    pub iItem: i32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub state: u32,
    pub stateMask: u32,
}
impl ::core::marker::Copy for LVFOOTERITEM {}
impl ::core::clone::Clone for LVFOOTERITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVFOOTERITEM_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFIF_TEXT: LVFOOTERITEM_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVFIF_STATE: LVFOOTERITEM_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGA_FOOTER_CENTER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGA_FOOTER_LEFT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGA_FOOTER_RIGHT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_ALIGN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_DESCRIPTIONBOTTOM: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_DESCRIPTIONTOP: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_EXTENDEDIMAGE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_GROUPID: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_ITEMS: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_SUBSET: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_SUBSETITEMS: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_SUBTITLE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_TASK: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_TITLEIMAGE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGGR_GROUP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGGR_HEADER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGGR_LABEL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGGR_SUBSETLINK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGIT_UNFOLDED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGMF_BORDERCOLOR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGMF_BORDERSIZE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGMF_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGMF_TEXTCOLOR: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVGROUP {
    pub cbSize: u32,
    pub mask: LVGROUP_MASK,
    pub pszHeader: ::windows_sys::core::PWSTR,
    pub cchHeader: i32,
    pub pszFooter: ::windows_sys::core::PWSTR,
    pub cchFooter: i32,
    pub iGroupId: i32,
    pub stateMask: u32,
    pub state: u32,
    pub uAlign: u32,
    pub pszSubtitle: ::windows_sys::core::PWSTR,
    pub cchSubtitle: u32,
    pub pszTask: ::windows_sys::core::PWSTR,
    pub cchTask: u32,
    pub pszDescriptionTop: ::windows_sys::core::PWSTR,
    pub cchDescriptionTop: u32,
    pub pszDescriptionBottom: ::windows_sys::core::PWSTR,
    pub cchDescriptionBottom: u32,
    pub iTitleImage: i32,
    pub iExtendedImage: i32,
    pub iFirstItem: i32,
    pub cItems: u32,
    pub pszSubsetTitle: ::windows_sys::core::PWSTR,
    pub cchSubsetTitle: u32,
}
impl ::core::marker::Copy for LVGROUP {}
impl ::core::clone::Clone for LVGROUP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVGROUPMETRICS {
    pub cbSize: u32,
    pub mask: u32,
    pub Left: u32,
    pub Top: u32,
    pub Right: u32,
    pub Bottom: u32,
    pub crLeft: u32,
    pub crTop: u32,
    pub crRight: u32,
    pub crBottom: u32,
    pub crHeader: u32,
    pub crFooter: u32,
}
impl ::core::marker::Copy for LVGROUPMETRICS {}
impl ::core::clone::Clone for LVGROUPMETRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVGROUP_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_NONE: LVGROUP_MASK = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_HEADER: LVGROUP_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_FOOTER: LVGROUP_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGF_STATE: LVGROUP_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_COLLAPSED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_COLLAPSIBLE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_FOCUSED: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_HIDDEN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_NOHEADER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_SELECTED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_SUBSETED: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGS_SUBSETLINKFOCUSED: u32 = 128u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LVHITTESTINFO {
    pub pt: super::super::Foundation::POINT,
    pub flags: LVHITTESTINFO_FLAGS,
    pub iItem: i32,
    pub iSubItem: i32,
    pub iGroup: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LVHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LVHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVHITTESTINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_ABOVE: LVHITTESTINFO_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_BELOW: LVHITTESTINFO_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_NOWHERE: LVHITTESTINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_ONITEMICON: LVHITTESTINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_ONITEMLABEL: LVHITTESTINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_ONITEMSTATEICON: LVHITTESTINFO_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_TOLEFT: LVHITTESTINFO_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_TORIGHT: LVHITTESTINFO_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP_HEADER: LVHITTESTINFO_FLAGS = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP_FOOTER: LVHITTESTINFO_FLAGS = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP_COLLAPSE: LVHITTESTINFO_FLAGS = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP_BACKGROUND: LVHITTESTINFO_FLAGS = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP_STATEICON: LVHITTESTINFO_FLAGS = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP_SUBSETLINK: LVHITTESTINFO_FLAGS = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_GROUP: LVHITTESTINFO_FLAGS = 4076863488u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_ONCONTENTS: LVHITTESTINFO_FLAGS = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVHT_EX_FOOTER: LVHITTESTINFO_FLAGS = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_COLFMT: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_COLUMNS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_DI_SETITEM: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_GROUPID: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_IMAGE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_INDENT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_NORECOMPUTE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_PARAM: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_STATE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIF_TEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIM_AFTER: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVINSERTGROUPSORTED {
    pub pfnGroupCompare: PFNLVGROUPCOMPARE,
    pub pvData: *mut ::core::ffi::c_void,
    pub lvGroup: LVGROUP,
}
impl ::core::marker::Copy for LVINSERTGROUPSORTED {}
impl ::core::clone::Clone for LVINSERTGROUPSORTED {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVINSERTMARK {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub iItem: i32,
    pub dwReserved: u32,
}
impl ::core::marker::Copy for LVINSERTMARK {}
impl ::core::clone::Clone for LVINSERTMARK {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIR_BOUNDS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIR_ICON: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIR_LABEL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIR_SELECTBOUNDS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_ACTIVATING: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_CUT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_DROPHILITED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_FOCUSED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_GLOW: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_OVERLAYMASK: u32 = 3840u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_SELECTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVIS_STATEIMAGEMASK: u32 = 61440u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LVITEMA {
    pub mask: u32,
    pub iItem: i32,
    pub iSubItem: i32,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: super::super::Foundation::LPARAM,
    pub iIndent: i32,
    pub iGroupId: LVITEMA_GROUP_ID,
    pub cColumns: u32,
    pub puColumns: *mut u32,
    pub piColFmt: *mut i32,
    pub iGroup: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LVITEMA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LVITEMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVITEMA_GROUP_ID = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_GROUPIDCALLBACK: LVITEMA_GROUP_ID = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_GROUPIDNONE: LVITEMA_GROUP_ID = -2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVITEMINDEX {
    pub iItem: i32,
    pub iGroup: i32,
}
impl ::core::marker::Copy for LVITEMINDEX {}
impl ::core::clone::Clone for LVITEMINDEX {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LVITEMW {
    pub mask: u32,
    pub iItem: i32,
    pub iSubItem: i32,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: super::super::Foundation::LPARAM,
    pub iIndent: i32,
    pub iGroupId: LVITEMA_GROUP_ID,
    pub cColumns: u32,
    pub puColumns: *mut u32,
    pub piColFmt: *mut i32,
    pub iGroup: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LVITEMW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LVITEMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVKF_ALT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVKF_CONTROL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVKF_SHIFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_APPROXIMATEVIEWRECT: u32 = 4160u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_ARRANGE: u32 = 4118u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_CANCELEDITLABEL: u32 = 4275u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_CREATEDRAGIMAGE: u32 = 4129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_DELETEALLITEMS: u32 = 4105u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_DELETECOLUMN: u32 = 4124u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_DELETEITEM: u32 = 4104u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_EDITLABEL: u32 = 4214u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_EDITLABELA: u32 = 4119u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_EDITLABELW: u32 = 4214u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_ENABLEGROUPVIEW: u32 = 4253u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_ENSUREVISIBLE: u32 = 4115u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_FINDITEM: u32 = 4179u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_FINDITEMA: u32 = 4109u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_FINDITEMW: u32 = 4179u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_FIRST: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETBKCOLOR: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETBKIMAGE: u32 = 4235u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETBKIMAGEA: u32 = 4165u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETBKIMAGEW: u32 = 4235u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCALLBACKMASK: u32 = 4106u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCOLUMN: u32 = 4191u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCOLUMNA: u32 = 4121u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCOLUMNORDERARRAY: u32 = 4155u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCOLUMNW: u32 = 4191u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCOLUMNWIDTH: u32 = 4125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETCOUNTPERPAGE: u32 = 4136u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETEDITCONTROL: u32 = 4120u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETEMPTYTEXT: u32 = 4300u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETEXTENDEDLISTVIEWSTYLE: u32 = 4151u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETFOCUSEDGROUP: u32 = 4189u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETFOOTERINFO: u32 = 4302u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETFOOTERITEM: u32 = 4304u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETFOOTERITEMRECT: u32 = 4303u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETFOOTERRECT: u32 = 4301u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETGROUPCOUNT: u32 = 4248u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETGROUPINFO: u32 = 4245u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETGROUPINFOBYINDEX: u32 = 4249u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETGROUPMETRICS: u32 = 4252u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETGROUPRECT: u32 = 4194u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETGROUPSTATE: u32 = 4188u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETHEADER: u32 = 4127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETHOTCURSOR: u32 = 4159u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETHOTITEM: u32 = 4157u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETHOVERTIME: u32 = 4168u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETIMAGELIST: u32 = 4098u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETINSERTMARK: u32 = 4263u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETINSERTMARKCOLOR: u32 = 4267u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETINSERTMARKRECT: u32 = 4265u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETISEARCHSTRING: u32 = 4213u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETISEARCHSTRINGA: u32 = 4148u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETISEARCHSTRINGW: u32 = 4213u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEM: u32 = 4171u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMA: u32 = 4101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMCOUNT: u32 = 4100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMINDEXRECT: u32 = 4305u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMPOSITION: u32 = 4112u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMRECT: u32 = 4110u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMSPACING: u32 = 4147u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMSTATE: u32 = 4140u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMTEXT: u32 = 4211u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMTEXTA: u32 = 4141u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMTEXTW: u32 = 4211u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETITEMW: u32 = 4171u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETNEXTITEM: u32 = 4108u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETNEXTITEMINDEX: u32 = 4307u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETNUMBEROFWORKAREAS: u32 = 4169u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETORIGIN: u32 = 4137u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETOUTLINECOLOR: u32 = 4272u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSELECTEDCOLUMN: u32 = 4270u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSELECTEDCOUNT: u32 = 4146u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSELECTIONMARK: u32 = 4162u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSTRINGWIDTH: u32 = 4183u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSTRINGWIDTHA: u32 = 4113u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSTRINGWIDTHW: u32 = 4183u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETSUBITEMRECT: u32 = 4152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETTEXTBKCOLOR: u32 = 4133u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETTEXTCOLOR: u32 = 4131u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETTILEINFO: u32 = 4261u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETTILEVIEWINFO: u32 = 4259u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETTOOLTIPS: u32 = 4174u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETTOPINDEX: u32 = 4135u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETVIEW: u32 = 4239u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETVIEWRECT: u32 = 4130u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_GETWORKAREAS: u32 = 4166u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_HASGROUP: u32 = 4257u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_HITTEST: u32 = 4114u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTCOLUMN: u32 = 4193u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTCOLUMNA: u32 = 4123u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTCOLUMNW: u32 = 4193u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTGROUP: u32 = 4241u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTGROUPSORTED: u32 = 4255u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTITEM: u32 = 4173u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTITEMA: u32 = 4103u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTITEMW: u32 = 4173u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_INSERTMARKHITTEST: u32 = 4264u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_ISGROUPVIEWENABLED: u32 = 4271u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_ISITEMVISIBLE: u32 = 4278u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_MAPIDTOINDEX: u32 = 4277u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_MAPINDEXTOID: u32 = 4276u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_MOVEGROUP: u32 = 4247u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_MOVEITEMTOGROUP: u32 = 4250u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_REDRAWITEMS: u32 = 4117u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_REMOVEALLGROUPS: u32 = 4256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_REMOVEGROUP: u32 = 4246u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SCROLL: u32 = 4116u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETBKCOLOR: u32 = 4097u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETBKIMAGE: u32 = 4234u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETBKIMAGEA: u32 = 4164u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETBKIMAGEW: u32 = 4234u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETCALLBACKMASK: u32 = 4107u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETCOLUMN: u32 = 4192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETCOLUMNA: u32 = 4122u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETCOLUMNORDERARRAY: u32 = 4154u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETCOLUMNW: u32 = 4192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETCOLUMNWIDTH: u32 = 4126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETEXTENDEDLISTVIEWSTYLE: u32 = 4150u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETGROUPINFO: u32 = 4243u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETGROUPMETRICS: u32 = 4251u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETHOTCURSOR: u32 = 4158u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETHOTITEM: u32 = 4156u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETHOVERTIME: u32 = 4167u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETICONSPACING: u32 = 4149u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETIMAGELIST: u32 = 4099u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETINFOTIP: u32 = 4269u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETINSERTMARK: u32 = 4262u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETINSERTMARKCOLOR: u32 = 4266u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEM: u32 = 4172u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMA: u32 = 4102u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMCOUNT: u32 = 4143u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMINDEXSTATE: u32 = 4306u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMPOSITION: u32 = 4111u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMPOSITION32: u32 = 4145u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMSTATE: u32 = 4139u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMTEXT: u32 = 4212u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMTEXTA: u32 = 4142u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMTEXTW: u32 = 4212u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETITEMW: u32 = 4172u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETOUTLINECOLOR: u32 = 4273u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETSELECTEDCOLUMN: u32 = 4236u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETSELECTIONMARK: u32 = 4163u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETTEXTBKCOLOR: u32 = 4134u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETTEXTCOLOR: u32 = 4132u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETTILEINFO: u32 = 4260u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETTILEVIEWINFO: u32 = 4258u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETTOOLTIPS: u32 = 4170u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETVIEW: u32 = 4238u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SETWORKAREAS: u32 = 4161u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SORTGROUPS: u32 = 4254u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SORTITEMS: u32 = 4144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SORTITEMSEX: u32 = 4177u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_SUBITEMHITTEST: u32 = 4153u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVM_UPDATE: u32 = 4138u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_ABOVE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_ALL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_BELOW: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_CUT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_DROPHILITED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_FOCUSED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_PREVIOUS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_SAMEGROUPONLY: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_SELECTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_TOLEFT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_TORIGHT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_VISIBLEONLY: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNI_VISIBLEORDER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNSCH_DEFAULT: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNSCH_ERROR: i32 = -2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVNSCH_IGNORE: i32 = -3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSCW_AUTOSIZE: i32 = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSCW_AUTOSIZE_USEHEADER: i32 = -2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVSETINFOTIP {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub iItem: i32,
    pub iSubItem: i32,
}
impl ::core::marker::Copy for LVSETINFOTIP {}
impl ::core::clone::Clone for LVSETINFOTIP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSICF_NOINVALIDATEALL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSICF_NOSCROLL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSIL_GROUPHEADER: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSIL_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSIL_SMALL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVSIL_STATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_ALIGNLEFT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_ALIGNMASK: u32 = 3072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_ALIGNTOP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_AUTOARRANGE: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EDITLABELS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_AUTOAUTOARRANGE: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_AUTOCHECKSELECT: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_AUTOSIZECOLUMNS: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_BORDERSELECT: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_CHECKBOXES: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_COLUMNOVERFLOW: u32 = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_COLUMNSNAPPOINTS: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_DOUBLEBUFFER: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_FLATSB: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_FULLROWSELECT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_GRIDLINES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_HEADERDRAGDROP: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_HEADERINALLVIEWS: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_HIDELABELS: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_INFOTIP: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_JUSTIFYCOLUMNS: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_LABELTIP: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_MULTIWORKAREAS: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_ONECLICKACTIVATE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_REGIONAL: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_SIMPLESELECT: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_SINGLEROW: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_SNAPTOGRID: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_SUBITEMIMAGES: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_TRACKSELECT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_TRANSPARENTBKGND: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_TRANSPARENTSHADOWTEXT: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_TWOCLICKACTIVATE: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_UNDERLINECOLD: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_EX_UNDERLINEHOT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_ICON: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_LIST: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_NOCOLUMNHEADER: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_NOLABELWRAP: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_NOSCROLL: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_NOSORTHEADER: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_OWNERDATA: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_OWNERDRAWFIXED: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_REPORT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_SHAREIMAGELISTS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_SHOWSELALWAYS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_SINGLESEL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_SMALLICON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_SORTASCENDING: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_SORTDESCENDING: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_TYPEMASK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVS_TYPESTYLEMASK: u32 = 64512u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct LVTILEINFO {
    pub cbSize: u32,
    pub iItem: i32,
    pub cColumns: u32,
    pub puColumns: *mut u32,
    pub piColFmt: *mut i32,
}
impl ::core::marker::Copy for LVTILEINFO {}
impl ::core::clone::Clone for LVTILEINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct LVTILEVIEWINFO {
    pub cbSize: u32,
    pub dwMask: u32,
    pub dwFlags: LVTILEVIEWINFO_FLAGS,
    pub sizeTile: super::super::Foundation::SIZE,
    pub cLines: i32,
    pub rcLabelMargin: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for LVTILEVIEWINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for LVTILEVIEWINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type LVTILEVIEWINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIF_EXTENDED: LVTILEVIEWINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIF_AUTOSIZE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIF_FIXEDHEIGHT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIF_FIXEDSIZE: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIF_FIXEDWIDTH: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIM_COLUMNS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIM_LABELMARGIN: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVTVIM_TILESIZE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_MAX_WORKAREAS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_VIEW_DETAILS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_VIEW_ICON: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_VIEW_LIST: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_VIEW_MAX: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_VIEW_SMALLICON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LV_VIEW_TILE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LWS_IGNORERETURN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LWS_NOPREFIX: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LWS_RIGHT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LWS_TRANSPARENT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LWS_USECUSTOMTEXT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LWS_USEVISUALSTYLE: u32 = 8u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct MARGINS {
    pub cxLeftWidth: i32,
    pub cxRightWidth: i32,
    pub cyTopHeight: i32,
    pub cyBottomHeight: i32,
}
impl ::core::marker::Copy for MARGINS {}
impl ::core::clone::Clone for MARGINS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MARKUPTEXTSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EMT_NORMALTEXT: MARKUPTEXTSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EMT_LINKTEXT: MARKUPTEXTSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MAXPROPPAGES: u32 = 100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MAX_INTLIST_COUNT: u32 = 402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MAX_LINKID_TEXT: u32 = 48u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MAX_THEMECOLOR: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MAX_THEMESIZE: u32 = 64u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MCGRIDINFO {
    pub cbSize: u32,
    pub dwPart: MCGRIDINFO_PART,
    pub dwFlags: MCGRIDINFO_FLAGS,
    pub iCalendar: i32,
    pub iRow: i32,
    pub iCol: i32,
    pub bSelected: super::super::Foundation::BOOL,
    pub stStart: super::super::Foundation::SYSTEMTIME,
    pub stEnd: super::super::Foundation::SYSTEMTIME,
    pub rc: super::super::Foundation::RECT,
    pub pszName: ::windows_sys::core::PWSTR,
    pub cchName: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MCGRIDINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MCGRIDINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MCGRIDINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIF_DATE: MCGRIDINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIF_RECT: MCGRIDINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIF_NAME: MCGRIDINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MCGRIDINFO_PART = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_CALENDARCONTROL: MCGRIDINFO_PART = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_NEXT: MCGRIDINFO_PART = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_PREV: MCGRIDINFO_PART = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_FOOTER: MCGRIDINFO_PART = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_CALENDAR: MCGRIDINFO_PART = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_CALENDARHEADER: MCGRIDINFO_PART = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_CALENDARBODY: MCGRIDINFO_PART = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_CALENDARROW: MCGRIDINFO_PART = 7u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCGIP_CALENDARCELL: MCGRIDINFO_PART = 8u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct MCHITTESTINFO {
    pub cbSize: u32,
    pub pt: super::super::Foundation::POINT,
    pub uHit: u32,
    pub st: super::super::Foundation::SYSTEMTIME,
    pub rc: super::super::Foundation::RECT,
    pub iOffset: i32,
    pub iRow: i32,
    pub iCol: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for MCHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for MCHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_CALENDAR: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_CALENDARBK: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_CALENDARCONTROL: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_NEXT: u32 = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_NOWHERE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_PREV: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_TITLE: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_TITLEBK: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCHT_TODAYLINK: u32 = 196608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCMV_CENTURY: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCMV_DECADE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCMV_MAX: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCMV_MONTH: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCMV_YEAR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_FIRST: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCALENDARBORDER: u32 = 4127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCALENDARCOUNT: u32 = 4119u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCALENDARGRIDINFO: u32 = 4120u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCALID: u32 = 4123u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCOLOR: u32 = 4107u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCURRENTVIEW: u32 = 4118u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETCURSEL: u32 = 4097u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETFIRSTDAYOFWEEK: u32 = 4112u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETMAXSELCOUNT: u32 = 4099u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETMAXTODAYWIDTH: u32 = 4117u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETMINREQRECT: u32 = 4105u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETMONTHDELTA: u32 = 4115u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETMONTHRANGE: u32 = 4103u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETRANGE: u32 = 4113u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETSELRANGE: u32 = 4101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETTODAY: u32 = 4109u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_HITTEST: u32 = 4110u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETCALENDARBORDER: u32 = 4126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETCALID: u32 = 4124u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETCOLOR: u32 = 4106u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETCURRENTVIEW: u32 = 4128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETCURSEL: u32 = 4098u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETDAYSTATE: u32 = 4104u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETFIRSTDAYOFWEEK: u32 = 4111u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETMAXSELCOUNT: u32 = 4100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETMONTHDELTA: u32 = 4116u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETRANGE: u32 = 4114u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETSELRANGE: u32 = 4102u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETTODAY: u32 = 4108u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCM_SIZERECTTOMIN: u32 = 4125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCSC_BACKGROUND: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCSC_MONTHBK: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCSC_TEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCSC_TITLEBK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCSC_TITLETEXT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCSC_TRAILINGTEXT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_DAYSTATE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_MULTISELECT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_NOSELCHANGEONNAV: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_NOTODAY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_NOTODAYCIRCLE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_NOTRAILINGDATES: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_SHORTDAYSOFWEEK: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCS_WEEKNUMBERS: u32 = 4u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct MEASUREITEMSTRUCT {
    pub CtlType: u32,
    pub CtlID: u32,
    pub itemID: u32,
    pub itemWidth: u32,
    pub itemHeight: u32,
    pub itemData: usize,
}
impl ::core::marker::Copy for MEASUREITEMSTRUCT {}
impl ::core::clone::Clone for MEASUREITEMSTRUCT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MENUBANDPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDP_NEWAPPBUTTON: MENUBANDPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDP_SEPERATOR: MENUBANDPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MENUBANDSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDS_NORMAL: MENUBANDSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDS_HOT: MENUBANDSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDS_PRESSED: MENUBANDSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDS_DISABLED: MENUBANDSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDS_CHECKED: MENUBANDSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MDS_HOTCHECKED: MENUBANDSTATES = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MONTHCALPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_BACKGROUND: MONTHCALPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_BORDERS: MONTHCALPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_GRIDBACKGROUND: MONTHCALPARTS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_COLHEADERSPLITTER: MONTHCALPARTS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_GRIDCELLBACKGROUND: MONTHCALPARTS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_GRIDCELL: MONTHCALPARTS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_GRIDCELLUPPER: MONTHCALPARTS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_TRAILINGGRIDCELL: MONTHCALPARTS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_TRAILINGGRIDCELLUPPER: MONTHCALPARTS = 9i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_NAVNEXT: MONTHCALPARTS = 10i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MC_NAVPREV: MONTHCALPARTS = 11i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MONTHCAL_CLASS: &str = "SysMonthCal32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MONTHCAL_CLASSA: &str = "SysMonthCal32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MONTHCAL_CLASSW: &str = "SysMonthCal32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MOREPROGRAMSARROWBACKSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSB_NORMAL: MOREPROGRAMSARROWBACKSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSB_HOT: MOREPROGRAMSARROWBACKSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSB_PRESSED: MOREPROGRAMSARROWBACKSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MOREPROGRAMSARROWSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPS_NORMAL: MOREPROGRAMSARROWSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPS_HOT: MOREPROGRAMSARROWSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPS_PRESSED: MOREPROGRAMSARROWSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type MOREPROGRAMSTABSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPMPT_NORMAL: MOREPROGRAMSTABSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPMPT_HOT: MOREPROGRAMSTABSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPMPT_SELECTED: MOREPROGRAMSTABSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPMPT_DISABLED: MOREPROGRAMSTABSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPMPT_FOCUSED: MOREPROGRAMSTABSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MSGF_COMMCTRL_BEGINDRAG: u32 = 16896u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MSGF_COMMCTRL_DRAGSELECT: u32 = 16898u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MSGF_COMMCTRL_SIZEHEADER: u32 = 16897u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MSGF_COMMCTRL_TOOLBARCUST: u32 = 16899u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MULTIFILEOPENORD: u32 = 1537u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NAVNEXTSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNN_NORMAL: NAVNEXTSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNN_HOT: NAVNEXTSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNN_PRESSED: NAVNEXTSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNN_DISABLED: NAVNEXTSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NAVPREVSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNP_NORMAL: NAVPREVSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNP_HOT: NAVPREVSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNP_PRESSED: NAVPREVSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCNP_DISABLED: NAVPREVSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NEWFILEOPENORD: u32 = 1547u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NEWFILEOPENV2ORD: u32 = 1552u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NEWFILEOPENV3ORD: u32 = 1553u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NEWFORMATDLGWITHLINK: u32 = 1591u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NFS_ALL: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NFS_BUTTON: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NFS_EDIT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NFS_LISTCOMBO: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NFS_STATIC: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NFS_USEFONTASSOC: u32 = 32u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMBCDROPDOWN {
    pub hdr: NMHDR,
    pub rcButton: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMBCDROPDOWN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMBCDROPDOWN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMBCHOTITEM {
    pub hdr: NMHDR,
    pub dwFlags: NMTBHOTITEM_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMBCHOTITEM {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMBCHOTITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCBEDRAGBEGINA {
    pub hdr: NMHDR,
    pub iItemid: i32,
    pub szText: [super::super::Foundation::CHAR; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCBEDRAGBEGINA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCBEDRAGBEGINA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCBEDRAGBEGINW {
    pub hdr: NMHDR,
    pub iItemid: i32,
    pub szText: [u16; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCBEDRAGBEGINW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCBEDRAGBEGINW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCBEENDEDITA {
    pub hdr: NMHDR,
    pub fChanged: super::super::Foundation::BOOL,
    pub iNewSelection: i32,
    pub szText: [super::super::Foundation::CHAR; 260],
    pub iWhy: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCBEENDEDITA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCBEENDEDITA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCBEENDEDITW {
    pub hdr: NMHDR,
    pub fChanged: super::super::Foundation::BOOL,
    pub iNewSelection: i32,
    pub szText: [u16; 260],
    pub iWhy: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCBEENDEDITW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCBEENDEDITW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCHAR {
    pub hdr: NMHDR,
    pub ch: u32,
    pub dwItemPrev: u32,
    pub dwItemNext: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCHAR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCHAR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCOMBOBOXEXA {
    pub hdr: NMHDR,
    pub ceItem: COMBOBOXEXITEMA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCOMBOBOXEXA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCOMBOBOXEXA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCOMBOBOXEXW {
    pub hdr: NMHDR,
    pub ceItem: COMBOBOXEXITEMW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCOMBOBOXEXW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCOMBOBOXEXW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMCUSTOMDRAW {
    pub hdr: NMHDR,
    pub dwDrawStage: NMCUSTOMDRAW_DRAW_STAGE,
    pub hdc: super::super::Graphics::Gdi::HDC,
    pub rc: super::super::Foundation::RECT,
    pub dwItemSpec: usize,
    pub uItemState: u32,
    pub lItemlParam: super::super::Foundation::LPARAM,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMCUSTOMDRAW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMCUSTOMDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMCUSTOMDRAW_DRAW_STAGE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_POSTPAINT: NMCUSTOMDRAW_DRAW_STAGE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_PREERASE: NMCUSTOMDRAW_DRAW_STAGE = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_PREPAINT: NMCUSTOMDRAW_DRAW_STAGE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_ITEMPOSTERASE: NMCUSTOMDRAW_DRAW_STAGE = 65540u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_ITEMPOSTPAINT: NMCUSTOMDRAW_DRAW_STAGE = 65538u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_ITEMPREERASE: NMCUSTOMDRAW_DRAW_STAGE = 65539u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_ITEMPREPAINT: NMCUSTOMDRAW_DRAW_STAGE = 65537u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const CDDS_SUBITEM: NMCUSTOMDRAW_DRAW_STAGE = 131072u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMCUSTOMSPLITRECTINFO {
    pub hdr: NMHDR,
    pub rcClient: super::super::Foundation::RECT,
    pub rcButton: super::super::Foundation::RECT,
    pub rcSplit: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMCUSTOMSPLITRECTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMCUSTOMSPLITRECTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMCUSTOMTEXT {
    pub hdr: NMHDR,
    pub hDC: super::super::Graphics::Gdi::HDC,
    pub lpString: ::windows_sys::core::PCWSTR,
    pub nCount: i32,
    pub lpRect: *mut super::super::Foundation::RECT,
    pub uFormat: u32,
    pub fLink: super::super::Foundation::BOOL,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMCUSTOMTEXT {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMCUSTOMTEXT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMECHANGE {
    pub nmhdr: NMHDR,
    pub dwFlags: u32,
    pub st: super::super::Foundation::SYSTEMTIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMECHANGE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMECHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMEFORMATA {
    pub nmhdr: NMHDR,
    pub pszFormat: ::windows_sys::core::PCSTR,
    pub st: super::super::Foundation::SYSTEMTIME,
    pub pszDisplay: ::windows_sys::core::PCSTR,
    pub szDisplay: [super::super::Foundation::CHAR; 64],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMEFORMATA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMEFORMATA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMEFORMATQUERYA {
    pub nmhdr: NMHDR,
    pub pszFormat: ::windows_sys::core::PCSTR,
    pub szMax: super::super::Foundation::SIZE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMEFORMATQUERYA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMEFORMATQUERYA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMEFORMATQUERYW {
    pub nmhdr: NMHDR,
    pub pszFormat: ::windows_sys::core::PCWSTR,
    pub szMax: super::super::Foundation::SIZE,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMEFORMATQUERYW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMEFORMATQUERYW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMEFORMATW {
    pub nmhdr: NMHDR,
    pub pszFormat: ::windows_sys::core::PCWSTR,
    pub st: super::super::Foundation::SYSTEMTIME,
    pub pszDisplay: ::windows_sys::core::PCWSTR,
    pub szDisplay: [u16; 64],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMEFORMATW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMEFORMATW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMESTRINGA {
    pub nmhdr: NMHDR,
    pub pszUserString: ::windows_sys::core::PCSTR,
    pub st: super::super::Foundation::SYSTEMTIME,
    pub dwFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMESTRINGA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMESTRINGA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMESTRINGW {
    pub nmhdr: NMHDR,
    pub pszUserString: ::windows_sys::core::PCWSTR,
    pub st: super::super::Foundation::SYSTEMTIME,
    pub dwFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMESTRINGW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMESTRINGW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMEWMKEYDOWNA {
    pub nmhdr: NMHDR,
    pub nVirtKey: i32,
    pub pszFormat: ::windows_sys::core::PCSTR,
    pub st: super::super::Foundation::SYSTEMTIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMEWMKEYDOWNA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMEWMKEYDOWNA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDATETIMEWMKEYDOWNW {
    pub nmhdr: NMHDR,
    pub nVirtKey: i32,
    pub pszFormat: ::windows_sys::core::PCWSTR,
    pub st: super::super::Foundation::SYSTEMTIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDATETIMEWMKEYDOWNW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDATETIMEWMKEYDOWNW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMDAYSTATE {
    pub nmhdr: NMHDR,
    pub stStart: super::super::Foundation::SYSTEMTIME,
    pub cDayState: i32,
    pub prgDayState: *mut u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMDAYSTATE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMDAYSTATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMHDDISPINFOA {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub mask: HDI_MASK,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMHDDISPINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMHDDISPINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMHDDISPINFOW {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub mask: HDI_MASK,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMHDDISPINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMHDDISPINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMHDFILTERBTNCLICK {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub rc: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMHDFILTERBTNCLICK {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMHDFILTERBTNCLICK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMHDR {
    pub hwndFrom: super::super::Foundation::HWND,
    pub idFrom: usize,
    pub code: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMHDR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMHDR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMHEADERA {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub iButton: HEADER_CONTROL_NOTIFICATION_BUTTON,
    pub pitem: *mut HDITEMA,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMHEADERA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMHEADERA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMHEADERW {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub iButton: HEADER_CONTROL_NOTIFICATION_BUTTON,
    pub pitem: *mut HDITEMW,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMHEADERW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMHEADERW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMIPADDRESS {
    pub hdr: NMHDR,
    pub iField: i32,
    pub iValue: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMIPADDRESS {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMIPADDRESS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMITEMACTIVATE {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub iSubItem: i32,
    pub uNewState: u32,
    pub uOldState: u32,
    pub uChanged: u32,
    pub ptAction: super::super::Foundation::POINT,
    pub lParam: super::super::Foundation::LPARAM,
    pub uKeyFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMITEMACTIVATE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMITEMACTIVATE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMKEY {
    pub hdr: NMHDR,
    pub nVKey: u32,
    pub uFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMKEY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLINK {
    pub hdr: NMHDR,
    pub item: LITEM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLINK {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLINK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLISTVIEW {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub iSubItem: i32,
    pub uNewState: u32,
    pub uOldState: u32,
    pub uChanged: u32,
    pub ptAction: super::super::Foundation::POINT,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLISTVIEW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLISTVIEW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVCACHEHINT {
    pub hdr: NMHDR,
    pub iFrom: i32,
    pub iTo: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVCACHEHINT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVCACHEHINT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMLVCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub clrText: u32,
    pub clrTextBk: u32,
    pub iSubItem: i32,
    pub dwItemType: NMLVCUSTOMDRAW_ITEM_TYPE,
    pub clrFace: u32,
    pub iIconEffect: i32,
    pub iIconPhase: i32,
    pub iPartId: i32,
    pub iStateId: i32,
    pub rcText: super::super::Foundation::RECT,
    pub uAlign: NMLVCUSTOMDRAW_ALIGN,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMLVCUSTOMDRAW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMLVCUSTOMDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMLVCUSTOMDRAW_ALIGN = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGA_HEADER_CENTER: NMLVCUSTOMDRAW_ALIGN = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGA_HEADER_LEFT: NMLVCUSTOMDRAW_ALIGN = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVGA_HEADER_RIGHT: NMLVCUSTOMDRAW_ALIGN = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMLVCUSTOMDRAW_ITEM_TYPE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCDI_ITEM: NMLVCUSTOMDRAW_ITEM_TYPE = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCDI_GROUP: NMLVCUSTOMDRAW_ITEM_TYPE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LVCDI_ITEMSLIST: NMLVCUSTOMDRAW_ITEM_TYPE = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVDISPINFOA {
    pub hdr: NMHDR,
    pub item: LVITEMA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVDISPINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVDISPINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVDISPINFOW {
    pub hdr: NMHDR,
    pub item: LVITEMW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVDISPINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVDISPINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVEMPTYMARKUP {
    pub hdr: NMHDR,
    pub dwFlags: NMLVEMPTYMARKUP_FLAGS,
    pub szMarkup: [u16; 2084],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVEMPTYMARKUP {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVEMPTYMARKUP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMLVEMPTYMARKUP_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const EMF_CENTERED: NMLVEMPTYMARKUP_FLAGS = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVFINDITEMA {
    pub hdr: NMHDR,
    pub iStart: i32,
    pub lvfi: LVFINDINFOA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVFINDITEMA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVFINDITEMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVFINDITEMW {
    pub hdr: NMHDR,
    pub iStart: i32,
    pub lvfi: LVFINDINFOW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVFINDITEMW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVFINDITEMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVGETINFOTIPA {
    pub hdr: NMHDR,
    pub dwFlags: u32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iItem: i32,
    pub iSubItem: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVGETINFOTIPA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVGETINFOTIPA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVGETINFOTIPW {
    pub hdr: NMHDR,
    pub dwFlags: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iItem: i32,
    pub iSubItem: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVGETINFOTIPW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVGETINFOTIPW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVKEYDOWN {
    pub hdr: NMHDR,
    pub wVKey: u16,
    pub flags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVKEYDOWN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVKEYDOWN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVLINK {
    pub hdr: NMHDR,
    pub link: LITEM,
    pub iItem: i32,
    pub iSubItem: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVLINK {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVLINK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVODSTATECHANGE {
    pub hdr: NMHDR,
    pub iFrom: i32,
    pub iTo: i32,
    pub uNewState: u32,
    pub uOldState: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVODSTATECHANGE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVODSTATECHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMLVSCROLL {
    pub hdr: NMHDR,
    pub dx: i32,
    pub dy: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMLVSCROLL {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMLVSCROLL {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMMOUSE {
    pub hdr: NMHDR,
    pub dwItemSpec: usize,
    pub dwItemData: usize,
    pub pt: super::super::Foundation::POINT,
    pub dwHitInfo: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMMOUSE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMMOUSE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMOBJECTNOTIFY {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub piid: *const ::windows_sys::core::GUID,
    pub pObject: *mut ::core::ffi::c_void,
    pub hResult: ::windows_sys::core::HRESULT,
    pub dwFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMOBJECTNOTIFY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMOBJECTNOTIFY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMPGCALCSIZE {
    pub hdr: NMHDR,
    pub dwFlag: NMPGCALCSIZE_FLAGS,
    pub iWidth: i32,
    pub iHeight: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMPGCALCSIZE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMPGCALCSIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMPGCALCSIZE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_CALCHEIGHT: NMPGCALCSIZE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_CALCWIDTH: NMPGCALCSIZE_FLAGS = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMPGHOTITEM {
    pub hdr: NMHDR,
    pub idOld: i32,
    pub idNew: i32,
    pub dwFlags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMPGHOTITEM {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMPGHOTITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMPGSCROLL {
    pub hdr: NMHDR,
    pub fwKeys: NMPGSCROLL_KEYS,
    pub rcParent: super::super::Foundation::RECT,
    pub iDir: NMPGSCROLL_DIR,
    pub iXpos: i32,
    pub iYpos: i32,
    pub iScroll: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMPGSCROLL {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMPGSCROLL {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMPGSCROLL_DIR = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_SCROLLDOWN: NMPGSCROLL_DIR = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_SCROLLLEFT: NMPGSCROLL_DIR = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_SCROLLRIGHT: NMPGSCROLL_DIR = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_SCROLLUP: NMPGSCROLL_DIR = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMPGSCROLL_KEYS = u16;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGK_NONE: NMPGSCROLL_KEYS = 0u16;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGK_SHIFT: NMPGSCROLL_KEYS = 1u16;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGK_CONTROL: NMPGSCROLL_KEYS = 2u16;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGK_MENU: NMPGSCROLL_KEYS = 4u16;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMRBAUTOSIZE {
    pub hdr: NMHDR,
    pub fChanged: super::super::Foundation::BOOL,
    pub rcTarget: super::super::Foundation::RECT,
    pub rcActual: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMRBAUTOSIZE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMRBAUTOSIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMREBAR {
    pub hdr: NMHDR,
    pub dwMask: NMREBAR_MASK_FLAGS,
    pub uBand: u32,
    pub fStyle: u32,
    pub wID: u32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMREBAR {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMREBAR {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMREBARAUTOBREAK {
    pub hdr: NMHDR,
    pub uBand: u32,
    pub wID: u32,
    pub lParam: super::super::Foundation::LPARAM,
    pub uMsg: u32,
    pub fStyleCurrent: u32,
    pub fAutoBreak: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMREBARAUTOBREAK {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMREBARAUTOBREAK {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMREBARCHEVRON {
    pub hdr: NMHDR,
    pub uBand: u32,
    pub wID: u32,
    pub lParam: super::super::Foundation::LPARAM,
    pub rc: super::super::Foundation::RECT,
    pub lParamNM: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMREBARCHEVRON {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMREBARCHEVRON {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMREBARCHILDSIZE {
    pub hdr: NMHDR,
    pub uBand: u32,
    pub wID: u32,
    pub rcChild: super::super::Foundation::RECT,
    pub rcBand: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMREBARCHILDSIZE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMREBARCHILDSIZE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMREBARSPLITTER {
    pub hdr: NMHDR,
    pub rcSizing: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMREBARSPLITTER {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMREBARSPLITTER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMREBAR_MASK_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBNM_ID: NMREBAR_MASK_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBNM_LPARAM: NMREBAR_MASK_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBNM_STYLE: NMREBAR_MASK_FLAGS = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMSEARCHWEB {
    pub hdr: NMHDR,
    pub entrypoint: EC_SEARCHWEB_ENTRYPOINT,
    pub hasQueryText: super::super::Foundation::BOOL,
    pub invokeSucceeded: super::super::Foundation::BOOL,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMSEARCHWEB {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMSEARCHWEB {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMSELCHANGE {
    pub nmhdr: NMHDR,
    pub stSelStart: super::super::Foundation::SYSTEMTIME,
    pub stSelEnd: super::super::Foundation::SYSTEMTIME,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMSELCHANGE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMSELCHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMTBCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub hbrMonoDither: super::super::Graphics::Gdi::HBRUSH,
    pub hbrLines: super::super::Graphics::Gdi::HBRUSH,
    pub hpenLines: super::super::Graphics::Gdi::HPEN,
    pub clrText: u32,
    pub clrMark: u32,
    pub clrTextHighlight: u32,
    pub clrBtnFace: u32,
    pub clrBtnHighlight: u32,
    pub clrHighlightHotTrack: u32,
    pub rcText: super::super::Foundation::RECT,
    pub nStringBkMode: i32,
    pub nHLStringBkMode: i32,
    pub iListGap: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMTBCUSTOMDRAW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMTBCUSTOMDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBDISPINFOA {
    pub hdr: NMHDR,
    pub dwMask: NMTBDISPINFOW_MASK,
    pub idCommand: i32,
    pub lParam: usize,
    pub iImage: i32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchText: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBDISPINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBDISPINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBDISPINFOW {
    pub hdr: NMHDR,
    pub dwMask: NMTBDISPINFOW_MASK,
    pub idCommand: i32,
    pub lParam: usize,
    pub iImage: i32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchText: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBDISPINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBDISPINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMTBDISPINFOW_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBNF_IMAGE: NMTBDISPINFOW_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBNF_TEXT: NMTBDISPINFOW_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBNF_DI_SETITEM: NMTBDISPINFOW_MASK = 268435456u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBGETINFOTIPA {
    pub hdr: NMHDR,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iItem: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBGETINFOTIPA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBGETINFOTIPA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBGETINFOTIPW {
    pub hdr: NMHDR,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iItem: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBGETINFOTIPW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBGETINFOTIPW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBHOTITEM {
    pub hdr: NMHDR,
    pub idOld: i32,
    pub idNew: i32,
    pub dwFlags: NMTBHOTITEM_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBHOTITEM {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBHOTITEM {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type NMTBHOTITEM_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_ACCELERATOR: NMTBHOTITEM_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_ARROWKEYS: NMTBHOTITEM_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_DUPACCEL: NMTBHOTITEM_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_ENTERING: NMTBHOTITEM_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_LEAVING: NMTBHOTITEM_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_LMOUSE: NMTBHOTITEM_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_MOUSE: NMTBHOTITEM_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_OTHER: NMTBHOTITEM_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_RESELECT: NMTBHOTITEM_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const HICF_TOGGLEDROPDOWN: NMTBHOTITEM_FLAGS = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBRESTORE {
    pub hdr: NMHDR,
    pub pData: *mut u32,
    pub pCurrent: *mut u32,
    pub cbData: u32,
    pub iItem: i32,
    pub cButtons: i32,
    pub cbBytesPerRecord: i32,
    pub tbButton: TBBUTTON,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBRESTORE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBRESTORE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTBSAVE {
    pub hdr: NMHDR,
    pub pData: *mut u32,
    pub pCurrent: *mut u32,
    pub cbData: u32,
    pub iItem: i32,
    pub cButtons: i32,
    pub tbButton: TBBUTTON,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTBSAVE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTBSAVE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTCKEYDOWN {
    pub hdr: NMHDR,
    pub wVKey: u16,
    pub flags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTCKEYDOWN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTCKEYDOWN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTOOLBARA {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub tbButton: TBBUTTON,
    pub cchText: i32,
    pub pszText: ::windows_sys::core::PSTR,
    pub rcButton: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTOOLBARA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTOOLBARA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTOOLBARW {
    pub hdr: NMHDR,
    pub iItem: i32,
    pub tbButton: TBBUTTON,
    pub cchText: i32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub rcButton: super::super::Foundation::RECT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTOOLBARW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTOOLBARW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTOOLTIPSCREATED {
    pub hdr: NMHDR,
    pub hwndToolTips: super::super::Foundation::HWND,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTOOLTIPSCREATED {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTOOLTIPSCREATED {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTRBTHUMBPOSCHANGING {
    pub hdr: NMHDR,
    pub dwPos: u32,
    pub nReason: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTRBTHUMBPOSCHANGING {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTRBTHUMBPOSCHANGING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTREEVIEWA {
    pub hdr: NMHDR,
    pub action: u32,
    pub itemOld: TVITEMA,
    pub itemNew: TVITEMA,
    pub ptDrag: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTREEVIEWA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTREEVIEWA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTREEVIEWW {
    pub hdr: NMHDR,
    pub action: u32,
    pub itemOld: TVITEMW,
    pub itemNew: TVITEMW,
    pub ptDrag: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTREEVIEWW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTREEVIEWW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMTTCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub uDrawFlags: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMTTCUSTOMDRAW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMTTCUSTOMDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTTDISPINFOA {
    pub hdr: NMHDR,
    pub lpszText: ::windows_sys::core::PSTR,
    pub szText: [super::super::Foundation::CHAR; 80],
    pub hinst: super::super::Foundation::HINSTANCE,
    pub uFlags: u32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTTDISPINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTTDISPINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTTDISPINFOW {
    pub hdr: NMHDR,
    pub lpszText: ::windows_sys::core::PWSTR,
    pub szText: [u16; 80],
    pub hinst: super::super::Foundation::HINSTANCE,
    pub uFlags: u32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTTDISPINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTTDISPINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMTVASYNCDRAW {
    pub hdr: NMHDR,
    pub pimldp: *mut IMAGELISTDRAWPARAMS,
    pub hr: ::windows_sys::core::HRESULT,
    pub hItem: HTREEITEM,
    pub lParam: super::super::Foundation::LPARAM,
    pub dwRetFlags: u32,
    pub iRetImageIndex: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMTVASYNCDRAW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMTVASYNCDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct NMTVCUSTOMDRAW {
    pub nmcd: NMCUSTOMDRAW,
    pub clrText: u32,
    pub clrTextBk: u32,
    pub iLevel: i32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for NMTVCUSTOMDRAW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for NMTVCUSTOMDRAW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVDISPINFOA {
    pub hdr: NMHDR,
    pub item: TVITEMA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVDISPINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVDISPINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVDISPINFOEXA {
    pub hdr: NMHDR,
    pub item: TVITEMEXA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVDISPINFOEXA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVDISPINFOEXA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVDISPINFOEXW {
    pub hdr: NMHDR,
    pub item: TVITEMEXW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVDISPINFOEXW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVDISPINFOEXW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVDISPINFOW {
    pub hdr: NMHDR,
    pub item: TVITEMW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVDISPINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVDISPINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVGETINFOTIPA {
    pub hdr: NMHDR,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub hItem: HTREEITEM,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVGETINFOTIPA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVGETINFOTIPA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVGETINFOTIPW {
    pub hdr: NMHDR,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub hItem: HTREEITEM,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVGETINFOTIPW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVGETINFOTIPW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVITEMCHANGE {
    pub hdr: NMHDR,
    pub uChanged: u32,
    pub hItem: HTREEITEM,
    pub uStateNew: u32,
    pub uStateOld: u32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVITEMCHANGE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVITEMCHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVKEYDOWN {
    pub hdr: NMHDR,
    pub wVKey: u16,
    pub flags: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVKEYDOWN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVKEYDOWN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMTVSTATEIMAGECHANGING {
    pub hdr: NMHDR,
    pub hti: HTREEITEM,
    pub iOldStateImageIndex: i32,
    pub iNewStateImageIndex: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMTVSTATEIMAGECHANGING {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMTVSTATEIMAGECHANGING {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMUPDOWN {
    pub hdr: NMHDR,
    pub iPos: i32,
    pub iDelta: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMUPDOWN {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMUPDOWN {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct NMVIEWCHANGE {
    pub nmhdr: NMHDR,
    pub dwOldView: u32,
    pub dwNewView: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for NMVIEWCHANGE {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for NMVIEWCHANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const NM_GETCUSTOMSPLITRECT: u32 = 4294966049u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ODT_HEADER: u32 = 100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type OFFSETTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_TOPLEFT: OFFSETTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_TOPRIGHT: OFFSETTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_TOPMIDDLE: OFFSETTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_BOTTOMLEFT: OFFSETTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_BOTTOMRIGHT: OFFSETTYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_BOTTOMMIDDLE: OFFSETTYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_MIDDLELEFT: OFFSETTYPE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_MIDDLERIGHT: OFFSETTYPE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_LEFTOFCAPTION: OFFSETTYPE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_RIGHTOFCAPTION: OFFSETTYPE = 9i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_LEFTOFLASTBUTTON: OFFSETTYPE = 10i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_RIGHTOFLASTBUTTON: OFFSETTYPE = 11i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_ABOVELASTBUTTON: OFFSETTYPE = 12i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OT_BELOWLASTBUTTON: OFFSETTYPE = 13i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type OPENBOXSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPOB_NORMAL: OPENBOXSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPOB_HOT: OPENBOXSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPOB_SELECTED: OPENBOXSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPOB_DISABLED: OPENBOXSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPOB_FOCUSED: OPENBOXSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type OPEN_THEME_DATA_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OTD_FORCE_RECT_SIZING: OPEN_THEME_DATA_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const OTD_NONCLIENT: OPEN_THEME_DATA_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type PAGEPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGRP_UP: PAGEPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGRP_DOWN: PAGEPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGRP_UPHORZ: PAGEPARTS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGRP_DOWNHORZ: PAGEPARTS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PAGESETUPDLGORD: u32 = 1546u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PAGESETUPDLGORDMOTIF: u32 = 1550u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_DELTAPOS: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_GETBARCOLOR: u32 = 1039u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_GETBKCOLOR: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_GETPOS: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_GETRANGE: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_GETSTATE: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_GETSTEP: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETBARCOLOR: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETBKCOLOR: u32 = 8193u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETMARQUEE: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETPOS: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETRANGE: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETRANGE32: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETSTATE: u32 = 1040u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_SETSTEP: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBM_STEPIT: u32 = 1029u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct PBRANGE {
    pub iLow: i32,
    pub iHigh: i32,
}
impl ::core::marker::Copy for PBRANGE {}
impl ::core::clone::Clone for PBRANGE {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBST_ERROR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBST_NORMAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBST_PAUSED: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBS_MARQUEE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBS_SMOOTH: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBS_SMOOTHREVERSE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PBS_VERTICAL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNDACOMPARE = ::core::option::Option<unsafe extern "system" fn(p1: *const ::core::ffi::c_void, p2: *const ::core::ffi::c_void, lparam: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNDACOMPARECONST = ::core::option::Option<unsafe extern "system" fn(p1: *const ::core::ffi::c_void, p2: *const ::core::ffi::c_void, lparam: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type PFNDAENUMCALLBACK = ::core::option::Option<unsafe extern "system" fn(p: *const ::core::ffi::c_void, pdata: *const ::core::ffi::c_void) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type PFNDAENUMCALLBACKCONST = ::core::option::Option<unsafe extern "system" fn(p: *const ::core::ffi::c_void, pdata: *const ::core::ffi::c_void) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNDPAMERGE = ::core::option::Option<unsafe extern "system" fn(umsg: DPAMM_MESSAGE, pvdest: *const ::core::ffi::c_void, pvsrc: *const ::core::ffi::c_void, lparam: super::super::Foundation::LPARAM) -> *mut ::core::ffi::c_void>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNDPAMERGECONST = ::core::option::Option<unsafe extern "system" fn(umsg: DPAMM_MESSAGE, pvdest: *const ::core::ffi::c_void, pvsrc: *const ::core::ffi::c_void, lparam: super::super::Foundation::LPARAM) -> *mut ::core::ffi::c_void>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Com\"`*"]
#[cfg(feature = "Win32_System_Com")]
pub type PFNDPASTREAM = ::core::option::Option<unsafe extern "system" fn(pinfo: *const DPASTREAMINFO, pstream: super::super::System::Com::IStream, pvinstdata: *const ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNLVCOMPARE = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::LPARAM, param1: super::super::Foundation::LPARAM, param2: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type PFNLVGROUPCOMPARE = ::core::option::Option<unsafe extern "system" fn(param0: i32, param1: i32, param2: *mut ::core::ffi::c_void) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNPROPSHEETCALLBACK = ::core::option::Option<unsafe extern "system" fn(param0: super::super::Foundation::HWND, param1: u32, param2: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFNTVCOMPARE = ::core::option::Option<unsafe extern "system" fn(lparam1: super::super::Foundation::LPARAM, lparam2: super::super::Foundation::LPARAM, lparamsort: super::super::Foundation::LPARAM) -> i32>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type PFTASKDIALOGCALLBACK = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, msg: u32, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM, lprefdata: isize) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGB_BOTTOMORRIGHT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGB_TOPORLEFT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_DEPRESSED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_GRAYED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_HOT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_INVISIBLE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGF_NORMAL: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_FIRST: u32 = 5120u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_FORWARDMOUSE: u32 = 5123u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_GETBKCOLOR: u32 = 5125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_GETBORDER: u32 = 5127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_GETBUTTONSIZE: u32 = 5131u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_GETBUTTONSTATE: u32 = 5132u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_GETDROPTARGET: u32 = 8196u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_GETPOS: u32 = 5129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_RECALCSIZE: u32 = 5122u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_SETBKCOLOR: u32 = 5124u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_SETBORDER: u32 = 5126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_SETBUTTONSIZE: u32 = 5130u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_SETCHILD: u32 = 5121u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_SETPOS: u32 = 5128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGM_SETSCROLLINFO: u32 = 5133u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGS_AUTOSCROLL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGS_DRAGNDROP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGS_HORZ: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PGS_VERT: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct POINTER_DEVICE_CURSOR_INFO {
    pub cursorId: u32,
    pub cursor: POINTER_DEVICE_CURSOR_TYPE,
}
impl ::core::marker::Copy for POINTER_DEVICE_CURSOR_INFO {}
impl ::core::clone::Clone for POINTER_DEVICE_CURSOR_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type POINTER_DEVICE_CURSOR_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_CURSOR_TYPE_UNKNOWN: POINTER_DEVICE_CURSOR_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_CURSOR_TYPE_TIP: POINTER_DEVICE_CURSOR_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_CURSOR_TYPE_ERASER: POINTER_DEVICE_CURSOR_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_CURSOR_TYPE_MAX: POINTER_DEVICE_CURSOR_TYPE = -1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct POINTER_DEVICE_INFO {
    pub displayOrientation: u32,
    pub device: super::super::Foundation::HANDLE,
    pub pointerDeviceType: POINTER_DEVICE_TYPE,
    pub monitor: super::super::Graphics::Gdi::HMONITOR,
    pub startingCursorId: u32,
    pub maxActiveContacts: u16,
    pub productString: [u16; 520],
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for POINTER_DEVICE_INFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for POINTER_DEVICE_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct POINTER_DEVICE_PROPERTY {
    pub logicalMin: i32,
    pub logicalMax: i32,
    pub physicalMin: i32,
    pub physicalMax: i32,
    pub unit: u32,
    pub unitExponent: u32,
    pub usagePageId: u16,
    pub usageId: u16,
}
impl ::core::marker::Copy for POINTER_DEVICE_PROPERTY {}
impl ::core::clone::Clone for POINTER_DEVICE_PROPERTY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type POINTER_DEVICE_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_TYPE_INTEGRATED_PEN: POINTER_DEVICE_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_TYPE_EXTERNAL_PEN: POINTER_DEVICE_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_TYPE_TOUCH: POINTER_DEVICE_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_TYPE_TOUCH_PAD: POINTER_DEVICE_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_DEVICE_TYPE_MAX: POINTER_DEVICE_TYPE = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type POINTER_FEEDBACK_MODE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_FEEDBACK_DEFAULT: POINTER_FEEDBACK_MODE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_FEEDBACK_INDIRECT: POINTER_FEEDBACK_MODE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const POINTER_FEEDBACK_NONE: POINTER_FEEDBACK_MODE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Input_Pointer\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct POINTER_TYPE_INFO {
    pub r#type: super::WindowsAndMessaging::POINTER_INPUT_TYPE,
    pub Anonymous: POINTER_TYPE_INFO_0,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for POINTER_TYPE_INFO {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for POINTER_TYPE_INFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_Input_Pointer\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
pub union POINTER_TYPE_INFO_0 {
    pub touchInfo: super::Input::Pointer::POINTER_TOUCH_INFO,
    pub penInfo: super::Input::Pointer::POINTER_PEN_INFO,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for POINTER_TYPE_INFO_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_Input_Pointer", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for POINTER_TYPE_INFO_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PRINTDLGEXORD: u32 = 1549u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PRINTDLGORD: u32 = 1538u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PRNSETUPDLGORD: u32 = 1539u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROGRESS_CLASS: &str = "msctls_progress32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROGRESS_CLASSA: &str = "msctls_progress32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROGRESS_CLASSW: &str = "msctls_progress32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type PROPERTYORIGIN = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PO_STATE: PROPERTYORIGIN = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PO_PART: PROPERTYORIGIN = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PO_CLASS: PROPERTYORIGIN = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PO_GLOBAL: PROPERTYORIGIN = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PO_NOTFOUND: PROPERTYORIGIN = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETHEADERA_V1 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETHEADERA_V1_0,
    pub pszCaption: ::windows_sys::core::PCSTR,
    pub nPages: u32,
    pub Anonymous2: PROPSHEETHEADERA_V1_1,
    pub Anonymous3: PROPSHEETHEADERA_V1_2,
    pub pfnCallback: PFNPROPSHEETCALLBACK,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V1_0 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V1_1 {
    pub nStartPage: u32,
    pub pStartPage: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V1_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V1_2 {
    pub ppsp: *mut PROPSHEETPAGEA,
    pub phpage: *mut HPROPSHEETPAGE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V1_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V1_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETHEADERA_V2 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETHEADERA_V2_0,
    pub pszCaption: ::windows_sys::core::PCSTR,
    pub nPages: u32,
    pub Anonymous2: PROPSHEETHEADERA_V2_1,
    pub Anonymous3: PROPSHEETHEADERA_V2_2,
    pub pfnCallback: PFNPROPSHEETCALLBACK,
    pub Anonymous4: PROPSHEETHEADERA_V2_3,
    pub hplWatermark: super::super::Graphics::Gdi::HPALETTE,
    pub Anonymous5: PROPSHEETHEADERA_V2_4,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V2_0 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V2_1 {
    pub nStartPage: u32,
    pub pStartPage: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V2_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V2_2 {
    pub ppsp: *mut PROPSHEETPAGEA,
    pub phpage: *mut HPROPSHEETPAGE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V2_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V2_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V2_3 {
    pub hbmWatermark: super::super::Graphics::Gdi::HBITMAP,
    pub pszbmWatermark: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V2_3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V2_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERA_V2_4 {
    pub hbmHeader: super::super::Graphics::Gdi::HBITMAP,
    pub pszbmHeader: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERA_V2_4 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERA_V2_4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETHEADERW_V1 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETHEADERW_V1_0,
    pub pszCaption: ::windows_sys::core::PCWSTR,
    pub nPages: u32,
    pub Anonymous2: PROPSHEETHEADERW_V1_1,
    pub Anonymous3: PROPSHEETHEADERW_V1_2,
    pub pfnCallback: PFNPROPSHEETCALLBACK,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V1_0 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V1_1 {
    pub nStartPage: u32,
    pub pStartPage: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V1_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V1_2 {
    pub ppsp: *mut PROPSHEETPAGEW,
    pub phpage: *mut HPROPSHEETPAGE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V1_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V1_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETHEADERW_V2 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETHEADERW_V2_0,
    pub pszCaption: ::windows_sys::core::PCWSTR,
    pub nPages: u32,
    pub Anonymous2: PROPSHEETHEADERW_V2_1,
    pub Anonymous3: PROPSHEETHEADERW_V2_2,
    pub pfnCallback: PFNPROPSHEETCALLBACK,
    pub Anonymous4: PROPSHEETHEADERW_V2_3,
    pub hplWatermark: super::super::Graphics::Gdi::HPALETTE,
    pub Anonymous5: PROPSHEETHEADERW_V2_4,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V2_0 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V2_1 {
    pub nStartPage: u32,
    pub pStartPage: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V2_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V2_2 {
    pub ppsp: *mut PROPSHEETPAGEW,
    pub phpage: *mut HPROPSHEETPAGE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V2_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V2_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V2_3 {
    pub hbmWatermark: super::super::Graphics::Gdi::HBITMAP,
    pub pszbmWatermark: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V2_3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V2_3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETHEADERW_V2_4 {
    pub hbmHeader: super::super::Graphics::Gdi::HBITMAP,
    pub pszbmHeader: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETHEADERW_V2_4 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETHEADERW_V2_4 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEA {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEA_0,
    pub Anonymous2: PROPSHEETPAGEA_1,
    pub pszTitle: ::windows_sys::core::PCSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKA,
    pub pcRefParent: *mut u32,
    pub pszHeaderTitle: ::windows_sys::core::PCSTR,
    pub pszHeaderSubTitle: ::windows_sys::core::PCSTR,
    pub hActCtx: super::super::Foundation::HANDLE,
    pub Anonymous3: PROPSHEETPAGEA_2,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_0 {
    pub pszTemplate: ::windows_sys::core::PCSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_2 {
    pub hbmHeader: super::super::Graphics::Gdi::HBITMAP,
    pub pszbmHeader: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEA_V1 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEA_V1_0,
    pub Anonymous2: PROPSHEETPAGEA_V1_1,
    pub pszTitle: ::windows_sys::core::PCSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKA,
    pub pcRefParent: *mut u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_V1_0 {
    pub pszTemplate: ::windows_sys::core::PCSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_V1_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V1_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEA_V2 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEA_V2_0,
    pub Anonymous2: PROPSHEETPAGEA_V2_1,
    pub pszTitle: ::windows_sys::core::PCSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKA,
    pub pcRefParent: *mut u32,
    pub pszHeaderTitle: ::windows_sys::core::PCSTR,
    pub pszHeaderSubTitle: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_V2_0 {
    pub pszTemplate: ::windows_sys::core::PCSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_V2_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V2_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEA_V3 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEA_V3_0,
    pub Anonymous2: PROPSHEETPAGEA_V3_1,
    pub pszTitle: ::windows_sys::core::PCSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKA,
    pub pcRefParent: *mut u32,
    pub pszHeaderTitle: ::windows_sys::core::PCSTR,
    pub pszHeaderSubTitle: ::windows_sys::core::PCSTR,
    pub hActCtx: super::super::Foundation::HANDLE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_V3_0 {
    pub pszTemplate: ::windows_sys::core::PCSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V3_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V3_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEA_V3_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEA_V3_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEA_V3_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEW {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEW_0,
    pub Anonymous2: PROPSHEETPAGEW_1,
    pub pszTitle: ::windows_sys::core::PCWSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKW,
    pub pcRefParent: *mut u32,
    pub pszHeaderTitle: ::windows_sys::core::PCWSTR,
    pub pszHeaderSubTitle: ::windows_sys::core::PCWSTR,
    pub hActCtx: super::super::Foundation::HANDLE,
    pub Anonymous3: PROPSHEETPAGEW_2,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_0 {
    pub pszTemplate: ::windows_sys::core::PCWSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_2 {
    pub hbmHeader: super::super::Graphics::Gdi::HBITMAP,
    pub pszbmHeader: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEW_V1 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEW_V1_0,
    pub Anonymous2: PROPSHEETPAGEW_V1_1,
    pub pszTitle: ::windows_sys::core::PCWSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKW,
    pub pcRefParent: *mut u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_V1_0 {
    pub pszTemplate: ::windows_sys::core::PCWSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V1_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V1_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_V1_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V1_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V1_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEW_V2 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEW_V2_0,
    pub Anonymous2: PROPSHEETPAGEW_V2_1,
    pub pszTitle: ::windows_sys::core::PCWSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKW,
    pub pcRefParent: *mut u32,
    pub pszHeaderTitle: ::windows_sys::core::PCWSTR,
    pub pszHeaderSubTitle: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V2 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V2 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_V2_0 {
    pub pszTemplate: ::windows_sys::core::PCWSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V2_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V2_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_V2_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V2_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V2_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct PROPSHEETPAGEW_V3 {
    pub dwSize: u32,
    pub dwFlags: u32,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub Anonymous1: PROPSHEETPAGEW_V3_0,
    pub Anonymous2: PROPSHEETPAGEW_V3_1,
    pub pszTitle: ::windows_sys::core::PCWSTR,
    pub pfnDlgProc: super::WindowsAndMessaging::DLGPROC,
    pub lParam: super::super::Foundation::LPARAM,
    pub pfnCallback: LPFNPSPCALLBACKW,
    pub pcRefParent: *mut u32,
    pub pszHeaderTitle: ::windows_sys::core::PCWSTR,
    pub pszHeaderSubTitle: ::windows_sys::core::PCWSTR,
    pub hActCtx: super::super::Foundation::HANDLE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V3 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V3 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_V3_0 {
    pub pszTemplate: ::windows_sys::core::PCWSTR,
    pub pResource: *mut super::WindowsAndMessaging::DLGTEMPLATE,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V3_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V3_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
pub union PROPSHEETPAGEW_V3_1 {
    pub hIcon: super::WindowsAndMessaging::HICON,
    pub pszIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for PROPSHEETPAGEW_V3_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for PROPSHEETPAGEW_V3_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROP_LG_CXDLG: u32 = 252u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROP_LG_CYDLG: u32 = 218u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROP_MED_CXDLG: u32 = 227u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROP_MED_CYDLG: u32 = 215u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROP_SM_CXDLG: u32 = 212u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PROP_SM_CYDLG: u32 = 188u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_APPLYNOW: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_BACK: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_CANCEL: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_FINISH: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_HELP: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_MAX: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_NEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSBTN_OK: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSCB_BUTTONPRESSED: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSCB_INITIALIZED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSCB_PRECREATE: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct PSHNOTIFY {
    pub hdr: NMHDR,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for PSHNOTIFY {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for PSHNOTIFY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_AEROWIZARD: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_HASHELP: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_HEADER: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_HEADERBITMAP: u32 = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_MODELESS: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_NOAPPLYNOW: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_NOCONTEXTHELP: u32 = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_NOMARGIN: u32 = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_PROPSHEETPAGE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_PROPTITLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_RESIZABLE: u32 = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_RTLREADING: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_STRETCHWATERMARK: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USECALLBACK: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEHBMHEADER: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEHBMWATERMARK: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEHICON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEHPLWATERMARK: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEICONID: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEPAGELANG: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_USEPSTARTPAGE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_WATERMARK: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_WIZARD: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_WIZARD97: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_WIZARDCONTEXTHELP: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_WIZARDHASFINISH: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSH_WIZARD_LITE: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_ADDPAGE: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_APPLY: u32 = 1134u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_CANCELTOCLOSE: u32 = 1131u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_CHANGED: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_ENABLEWIZBUTTONS: u32 = 1163u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_GETCURRENTPAGEHWND: u32 = 1142u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_GETRESULT: u32 = 1159u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_GETTABCONTROL: u32 = 1140u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_HWNDTOINDEX: u32 = 1153u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_IDTOINDEX: u32 = 1157u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_INDEXTOHWND: u32 = 1154u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_INDEXTOID: u32 = 1158u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_INDEXTOPAGE: u32 = 1156u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_INSERTPAGE: u32 = 1143u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_ISDIALOGMESSAGE: u32 = 1141u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_PAGETOINDEX: u32 = 1155u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_PRESSBUTTON: u32 = 1137u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_QUERYSIBLINGS: u32 = 1132u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_REBOOTSYSTEM: u32 = 1130u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_RECALCPAGESIZES: u32 = 1160u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_REMOVEPAGE: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_RESTARTWINDOWS: u32 = 1129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETBUTTONTEXT: u32 = 1164u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETBUTTONTEXTW: u32 = 1164u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETCURSEL: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETCURSELID: u32 = 1138u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETFINISHTEXT: u32 = 1145u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETFINISHTEXTA: u32 = 1139u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETFINISHTEXTW: u32 = 1145u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETHEADERSUBTITLE: u32 = 1152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETHEADERSUBTITLEA: u32 = 1151u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETHEADERSUBTITLEW: u32 = 1152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETHEADERTITLE: u32 = 1150u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETHEADERTITLEA: u32 = 1149u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETHEADERTITLEW: u32 = 1150u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETNEXTTEXT: u32 = 1161u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETNEXTTEXTW: u32 = 1161u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETTITLE: u32 = 1144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETTITLEA: u32 = 1135u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETTITLEW: u32 = 1144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SETWIZBUTTONS: u32 = 1136u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_SHOWWIZBUTTONS: u32 = 1162u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSM_UNCHANGED: u32 = 1133u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSNRET_INVALID: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSNRET_INVALID_NOCHANGEPAGE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSNRET_MESSAGEHANDLED: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSNRET_NOERROR: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type PSPCB_MESSAGE = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSPCB_ADDREF: PSPCB_MESSAGE = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSPCB_CREATE: PSPCB_MESSAGE = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSPCB_RELEASE: PSPCB_MESSAGE = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSPCB_SI_INITDIALOG: PSPCB_MESSAGE = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_DLGINDIRECT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_HASHELP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_HIDEHEADER: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_PREMATURE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_RTLREADING: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USECALLBACK: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USEFUSIONCONTEXT: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USEHEADERSUBTITLE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USEHEADERTITLE: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USEHICON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USEICONID: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USEREFPARENT: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSP_USETITLE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZBF_ELEVATIONREQUIRED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_BACK: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_CANCEL: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_DISABLEDFINISH: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_FINISH: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_NEXT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_RESTORE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const PSWIZB_SHOW: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBAB_ADDBAND: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBAB_AUTOSIZE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_BACKGROUND: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_CHEVRONLOCATION: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_CHEVRONSTATE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_CHILD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_CHILDSIZE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_COLORS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_HEADERSIZE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_ID: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_IDEALSIZE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_IMAGE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_LPARAM: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_SIZE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_STYLE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBIM_TEXT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_BREAK: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_CHILDEDGE: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_FIXEDBMP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_FIXEDSIZE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_GRIPPERALWAYS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_HIDDEN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_HIDETITLE: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_NOGRIPPER: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_NOVERT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_TOPALIGN: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_USECHEVRON: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBBS_VARIABLEHEIGHT: u32 = 64u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct RBHITTESTINFO {
    pub pt: super::super::Foundation::POINT,
    pub flags: u32,
    pub iBand: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for RBHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for RBHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBHT_CAPTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBHT_CHEVRON: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBHT_CLIENT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBHT_GRABBER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBHT_NOWHERE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBHT_SPLITTER: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBIM_IMAGELIST: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBSTR_CHANGERECT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_AUTOSIZE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_BANDBORDERS: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_DBLCLKTOGGLE: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_FIXEDORDER: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_REGISTERDROP: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_TOOLTIPS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_VARHEIGHT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RBS_VERTICALGRIPPER: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_BEGINDRAG: u32 = 1048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_DELETEBAND: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_DRAGMOVE: u32 = 1050u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_ENDDRAG: u32 = 1049u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBANDBORDERS: u32 = 1058u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBANDCOUNT: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBANDINFO: u32 = 1052u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBANDINFOA: u32 = 1053u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBANDINFOW: u32 = 1052u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBANDMARGINS: u32 = 1064u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBARHEIGHT: u32 = 1051u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBARINFO: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETBKCOLOR: u32 = 1044u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETCOLORSCHEME: u32 = 8195u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETDROPTARGET: u32 = 8196u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETEXTENDEDSTYLE: u32 = 1066u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETPALETTE: u32 = 1062u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETRECT: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETROWCOUNT: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETROWHEIGHT: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETTEXTCOLOR: u32 = 1046u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETTOOLTIPS: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_HITTEST: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_IDTOINDEX: u32 = 1040u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_INSERTBAND: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_INSERTBANDA: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_INSERTBANDW: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_MAXIMIZEBAND: u32 = 1055u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_MINIMIZEBAND: u32 = 1054u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_MOVEBAND: u32 = 1063u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_PUSHCHEVRON: u32 = 1067u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETBANDINFO: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETBANDINFOA: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETBANDINFOW: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETBANDWIDTH: u32 = 1068u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETBARINFO: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETBKCOLOR: u32 = 1043u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETCOLORSCHEME: u32 = 8194u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETEXTENDEDSTYLE: u32 = 1065u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETPALETTE: u32 = 1061u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETPARENT: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETTEXTCOLOR: u32 = 1045u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETTOOLTIPS: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SETWINDOWTHEME: u32 = 8203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SHOWBAND: u32 = 1059u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RB_SIZETORECT: u32 = 1047u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct REBARBANDINFOA {
    pub cbSize: u32,
    pub fMask: u32,
    pub fStyle: u32,
    pub clrFore: u32,
    pub clrBack: u32,
    pub lpText: ::windows_sys::core::PSTR,
    pub cch: u32,
    pub iImage: i32,
    pub hwndChild: super::super::Foundation::HWND,
    pub cxMinChild: u32,
    pub cyMinChild: u32,
    pub cx: u32,
    pub hbmBack: super::super::Graphics::Gdi::HBITMAP,
    pub wID: u32,
    pub cyChild: u32,
    pub cyMaxChild: u32,
    pub cyIntegral: u32,
    pub cxIdeal: u32,
    pub lParam: super::super::Foundation::LPARAM,
    pub cxHeader: u32,
    pub rcChevronLocation: super::super::Foundation::RECT,
    pub uChevronState: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for REBARBANDINFOA {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for REBARBANDINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_Graphics_Gdi\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
pub struct REBARBANDINFOW {
    pub cbSize: u32,
    pub fMask: u32,
    pub fStyle: u32,
    pub clrFore: u32,
    pub clrBack: u32,
    pub lpText: ::windows_sys::core::PWSTR,
    pub cch: u32,
    pub iImage: i32,
    pub hwndChild: super::super::Foundation::HWND,
    pub cxMinChild: u32,
    pub cyMinChild: u32,
    pub cx: u32,
    pub hbmBack: super::super::Graphics::Gdi::HBITMAP,
    pub wID: u32,
    pub cyChild: u32,
    pub cyMaxChild: u32,
    pub cyIntegral: u32,
    pub cxIdeal: u32,
    pub lParam: super::super::Foundation::LPARAM,
    pub cxHeader: u32,
    pub rcChevronLocation: super::super::Foundation::RECT,
    pub uChevronState: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::marker::Copy for REBARBANDINFOW {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_Graphics_Gdi"))]
impl ::core::clone::Clone for REBARBANDINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const REBARCLASSNAME: &str = "ReBarWindow32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const REBARCLASSNAMEA: &str = "ReBarWindow32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const REBARCLASSNAMEW: &str = "ReBarWindow32";
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct REBARINFO {
    pub cbSize: u32,
    pub fMask: u32,
    pub himl: HIMAGELIST,
}
impl ::core::marker::Copy for REBARINFO {}
impl ::core::clone::Clone for REBARINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const REPLACEDLGORD: u32 = 1541u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const RUNDLGORD: u32 = 1545u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBARS_SIZEGRIP: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBARS_TOOLTIPS: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBT_NOBORDERS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBT_NOTABPARSING: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBT_OWNERDRAW: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBT_POPOUT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBT_RTLREADING: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SBT_TOOLTIPS: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETBORDERS: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETICON: u32 = 1044u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETPARTS: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETRECT: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTEXT: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTEXTA: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTEXTLENGTH: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTEXTLENGTHA: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTEXTLENGTHW: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTEXTW: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTIPTEXTA: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETTIPTEXTW: u32 = 1043u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_ISSIMPLE: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETBKCOLOR: u32 = 8193u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETICON: u32 = 1039u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETMINHEIGHT: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETPARTS: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETTEXT: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETTEXTA: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETTEXTW: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETTIPTEXTA: u32 = 1040u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETTIPTEXTW: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SIMPLE: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SB_SIMPLEID: u32 = 255u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type SIZINGTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ST_TRUESIZE: SIZINGTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ST_STRETCH: SIZINGTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ST_TILE: SIZINGTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type SOFTWAREEXPLORERSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSE_NORMAL: SOFTWAREEXPLORERSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSE_HOT: SOFTWAREEXPLORERSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSE_SELECTED: SOFTWAREEXPLORERSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSE_DISABLED: SOFTWAREEXPLORERSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPSE_FOCUSED: SOFTWAREEXPLORERSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type STARTPANELPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_USERPANE: STARTPANELPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_MOREPROGRAMS: STARTPANELPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_MOREPROGRAMSARROW: STARTPANELPARTS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_PROGLIST: STARTPANELPARTS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_PROGLISTSEPARATOR: STARTPANELPARTS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_PLACESLIST: STARTPANELPARTS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_PLACESLISTSEPARATOR: STARTPANELPARTS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_LOGOFF: STARTPANELPARTS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_LOGOFFBUTTONS: STARTPANELPARTS = 9i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_USERPICTURE: STARTPANELPARTS = 10i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_PREVIEW: STARTPANELPARTS = 11i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_MOREPROGRAMSTAB: STARTPANELPARTS = 12i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_NSCHOST: STARTPANELPARTS = 13i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_SOFTWAREEXPLORER: STARTPANELPARTS = 14i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_OPENBOX: STARTPANELPARTS = 15i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_SEARCHVIEW: STARTPANELPARTS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_MOREPROGRAMSARROWBACK: STARTPANELPARTS = 17i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_TOPMATCH: STARTPANELPARTS = 18i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SPP_LOGOFFSPLITBUTTONDROPDOWN: STARTPANELPARTS = 19i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type STATICPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STAT_TEXT: STATICPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATUSCLASSNAME: &str = "msctls_statusbar32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATUSCLASSNAMEA: &str = "msctls_statusbar32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STATUSCLASSNAMEW: &str = "msctls_statusbar32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_COPY: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_CUT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_DELETE: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_FILENEW: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_FILEOPEN: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_FILESAVE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_FIND: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_HELP: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_PASTE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_PRINT: u32 = 14u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_PRINTPRE: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_PROPERTIES: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_REDOW: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_REPLACE: u32 = 13u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const STD_UNDO: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SZ_THDOCPROP_AUTHOR: &str = "author";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SZ_THDOCPROP_CANONICALNAME: &str = "ThemeName";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SZ_THDOCPROP_DISPLAYNAME: &str = "DisplayName";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const SZ_THDOCPROP_TOOLTIP: &str = "ToolTip";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKBANDPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDP_GROUPCOUNT: TASKBANDPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDP_FLASHBUTTON: TASKBANDPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDP_FLASHBUTTONGROUPMENU: TASKBANDPARTS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKBARPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_BACKGROUNDBOTTOM: TASKBARPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_BACKGROUNDRIGHT: TASKBARPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_BACKGROUNDTOP: TASKBARPARTS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_BACKGROUNDLEFT: TASKBARPARTS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_SIZINGBARBOTTOM: TASKBARPARTS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_SIZINGBARRIGHT: TASKBARPARTS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_SIZINGBARTOP: TASKBARPARTS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBP_SIZINGBARLEFT: TASKBARPARTS = 8i32;
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub struct TASKDIALOGCONFIG {
    pub cbSize: u32,
    pub hwndParent: super::super::Foundation::HWND,
    pub hInstance: super::super::Foundation::HINSTANCE,
    pub dwFlags: TASKDIALOG_FLAGS,
    pub dwCommonButtons: TASKDIALOG_COMMON_BUTTON_FLAGS,
    pub pszWindowTitle: ::windows_sys::core::PCWSTR,
    pub Anonymous1: TASKDIALOGCONFIG_0,
    pub pszMainInstruction: ::windows_sys::core::PCWSTR,
    pub pszContent: ::windows_sys::core::PCWSTR,
    pub cButtons: u32,
    pub pButtons: *const TASKDIALOG_BUTTON,
    pub nDefaultButton: i32,
    pub cRadioButtons: u32,
    pub pRadioButtons: *const TASKDIALOG_BUTTON,
    pub nDefaultRadioButton: i32,
    pub pszVerificationText: ::windows_sys::core::PCWSTR,
    pub pszExpandedInformation: ::windows_sys::core::PCWSTR,
    pub pszExpandedControlText: ::windows_sys::core::PCWSTR,
    pub pszCollapsedControlText: ::windows_sys::core::PCWSTR,
    pub Anonymous2: TASKDIALOGCONFIG_1,
    pub pszFooter: ::windows_sys::core::PCWSTR,
    pub pfCallback: PFTASKDIALOGCALLBACK,
    pub lpCallbackData: isize,
    pub cxWidth: u32,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for TASKDIALOGCONFIG {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for TASKDIALOGCONFIG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub union TASKDIALOGCONFIG_0 {
    pub hMainIcon: super::WindowsAndMessaging::HICON,
    pub pszMainIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for TASKDIALOGCONFIG_0 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for TASKDIALOGCONFIG_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
pub union TASKDIALOGCONFIG_1 {
    pub hFooterIcon: super::WindowsAndMessaging::HICON,
    pub pszFooterIcon: ::windows_sys::core::PCWSTR,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::marker::Copy for TASKDIALOGCONFIG_1 {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
impl ::core::clone::Clone for TASKDIALOGCONFIG_1 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TASKDIALOG_BUTTON {
    pub nButtonID: i32,
    pub pszButtonText: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for TASKDIALOG_BUTTON {}
impl ::core::clone::Clone for TASKDIALOG_BUTTON {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKDIALOG_COMMON_BUTTON_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDCBF_OK_BUTTON: TASKDIALOG_COMMON_BUTTON_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDCBF_YES_BUTTON: TASKDIALOG_COMMON_BUTTON_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDCBF_NO_BUTTON: TASKDIALOG_COMMON_BUTTON_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDCBF_CANCEL_BUTTON: TASKDIALOG_COMMON_BUTTON_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDCBF_RETRY_BUTTON: TASKDIALOG_COMMON_BUTTON_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDCBF_CLOSE_BUTTON: TASKDIALOG_COMMON_BUTTON_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKDIALOG_ELEMENTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDE_CONTENT: TASKDIALOG_ELEMENTS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDE_EXPANDED_INFORMATION: TASKDIALOG_ELEMENTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDE_FOOTER: TASKDIALOG_ELEMENTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDE_MAIN_INSTRUCTION: TASKDIALOG_ELEMENTS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKDIALOG_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_ENABLE_HYPERLINKS: TASKDIALOG_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_USE_HICON_MAIN: TASKDIALOG_FLAGS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_USE_HICON_FOOTER: TASKDIALOG_FLAGS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_ALLOW_DIALOG_CANCELLATION: TASKDIALOG_FLAGS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_USE_COMMAND_LINKS: TASKDIALOG_FLAGS = 16i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_USE_COMMAND_LINKS_NO_ICON: TASKDIALOG_FLAGS = 32i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_EXPAND_FOOTER_AREA: TASKDIALOG_FLAGS = 64i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_EXPANDED_BY_DEFAULT: TASKDIALOG_FLAGS = 128i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_VERIFICATION_FLAG_CHECKED: TASKDIALOG_FLAGS = 256i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_SHOW_PROGRESS_BAR: TASKDIALOG_FLAGS = 512i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_SHOW_MARQUEE_PROGRESS_BAR: TASKDIALOG_FLAGS = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_CALLBACK_TIMER: TASKDIALOG_FLAGS = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_POSITION_RELATIVE_TO_WINDOW: TASKDIALOG_FLAGS = 4096i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_RTL_LAYOUT: TASKDIALOG_FLAGS = 8192i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_NO_DEFAULT_RADIO_BUTTON: TASKDIALOG_FLAGS = 16384i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_CAN_BE_MINIMIZED: TASKDIALOG_FLAGS = 32768i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_NO_SET_FOREGROUND: TASKDIALOG_FLAGS = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDF_SIZE_TO_CONTENT: TASKDIALOG_FLAGS = 16777216i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKDIALOG_ICON_ELEMENTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDIE_ICON_MAIN: TASKDIALOG_ICON_ELEMENTS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDIE_ICON_FOOTER: TASKDIALOG_ICON_ELEMENTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKDIALOG_MESSAGES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_NAVIGATE_PAGE: TASKDIALOG_MESSAGES = 1125i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_CLICK_BUTTON: TASKDIALOG_MESSAGES = 1126i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_MARQUEE_PROGRESS_BAR: TASKDIALOG_MESSAGES = 1127i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_PROGRESS_BAR_STATE: TASKDIALOG_MESSAGES = 1128i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_PROGRESS_BAR_RANGE: TASKDIALOG_MESSAGES = 1129i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_PROGRESS_BAR_POS: TASKDIALOG_MESSAGES = 1130i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_PROGRESS_BAR_MARQUEE: TASKDIALOG_MESSAGES = 1131i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_ELEMENT_TEXT: TASKDIALOG_MESSAGES = 1132i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_CLICK_RADIO_BUTTON: TASKDIALOG_MESSAGES = 1134i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_ENABLE_BUTTON: TASKDIALOG_MESSAGES = 1135i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_ENABLE_RADIO_BUTTON: TASKDIALOG_MESSAGES = 1136i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_CLICK_VERIFICATION: TASKDIALOG_MESSAGES = 1137i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_UPDATE_ELEMENT_TEXT: TASKDIALOG_MESSAGES = 1138i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_SET_BUTTON_ELEVATION_REQUIRED_STATE: TASKDIALOG_MESSAGES = 1139i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDM_UPDATE_ICON: TASKDIALOG_MESSAGES = 1140i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TASKDIALOG_NOTIFICATIONS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_CREATED: TASKDIALOG_NOTIFICATIONS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_NAVIGATED: TASKDIALOG_NOTIFICATIONS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_BUTTON_CLICKED: TASKDIALOG_NOTIFICATIONS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_HYPERLINK_CLICKED: TASKDIALOG_NOTIFICATIONS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_TIMER: TASKDIALOG_NOTIFICATIONS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_DESTROYED: TASKDIALOG_NOTIFICATIONS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_RADIO_BUTTON_CLICKED: TASKDIALOG_NOTIFICATIONS = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_DIALOG_CONSTRUCTED: TASKDIALOG_NOTIFICATIONS = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_VERIFICATION_CLICKED: TASKDIALOG_NOTIFICATIONS = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_HELP: TASKDIALOG_NOTIFICATIONS = 9i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TDN_EXPANDO_BUTTON_CLICKED: TASKDIALOG_NOTIFICATIONS = 10i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TA_CUBIC_BEZIER {
    pub header: TA_TIMINGFUNCTION,
    pub rX0: f32,
    pub rY0: f32,
    pub rX1: f32,
    pub rY1: f32,
}
impl ::core::marker::Copy for TA_CUBIC_BEZIER {}
impl ::core::clone::Clone for TA_CUBIC_BEZIER {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TA_PROPERTY = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAP_FLAGS: TA_PROPERTY = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAP_TRANSFORMCOUNT: TA_PROPERTY = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAP_STAGGERDELAY: TA_PROPERTY = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAP_STAGGERDELAYCAP: TA_PROPERTY = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAP_STAGGERDELAYFACTOR: TA_PROPERTY = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAP_ZORDER: TA_PROPERTY = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TA_PROPERTY_FLAG = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAPF_NONE: TA_PROPERTY_FLAG = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAPF_HASSTAGGER: TA_PROPERTY_FLAG = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAPF_ISRTLAWARE: TA_PROPERTY_FLAG = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAPF_ALLOWCOLLECTION: TA_PROPERTY_FLAG = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAPF_HASBACKGROUND: TA_PROPERTY_FLAG = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TAPF_HASPERSPECTIVE: TA_PROPERTY_FLAG = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TA_TIMINGFUNCTION {
    pub eTimingFunctionType: TA_TIMINGFUNCTION_TYPE,
}
impl ::core::marker::Copy for TA_TIMINGFUNCTION {}
impl ::core::clone::Clone for TA_TIMINGFUNCTION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TA_TIMINGFUNCTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTFT_UNDEFINED: TA_TIMINGFUNCTION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTFT_CUBIC_BEZIER: TA_TIMINGFUNCTION_TYPE = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TA_TRANSFORM {
    pub eTransformType: TA_TRANSFORM_TYPE,
    pub dwTimingFunctionId: u32,
    pub dwStartTime: u32,
    pub dwDurationTime: u32,
    pub eFlags: TA_TRANSFORM_FLAG,
}
impl ::core::marker::Copy for TA_TRANSFORM {}
impl ::core::clone::Clone for TA_TRANSFORM {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TA_TRANSFORM_2D {
    pub header: TA_TRANSFORM,
    pub rX: f32,
    pub rY: f32,
    pub rInitialX: f32,
    pub rInitialY: f32,
    pub rOriginX: f32,
    pub rOriginY: f32,
}
impl ::core::marker::Copy for TA_TRANSFORM_2D {}
impl ::core::clone::Clone for TA_TRANSFORM_2D {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TA_TRANSFORM_CLIP {
    pub header: TA_TRANSFORM,
    pub rLeft: f32,
    pub rTop: f32,
    pub rRight: f32,
    pub rBottom: f32,
    pub rInitialLeft: f32,
    pub rInitialTop: f32,
    pub rInitialRight: f32,
    pub rInitialBottom: f32,
}
impl ::core::marker::Copy for TA_TRANSFORM_CLIP {}
impl ::core::clone::Clone for TA_TRANSFORM_CLIP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TA_TRANSFORM_FLAG = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATF_NONE: TA_TRANSFORM_FLAG = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATF_TARGETVALUES_USER: TA_TRANSFORM_FLAG = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATF_HASINITIALVALUES: TA_TRANSFORM_FLAG = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATF_HASORIGINVALUES: TA_TRANSFORM_FLAG = 4i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TA_TRANSFORM_OPACITY {
    pub header: TA_TRANSFORM,
    pub rOpacity: f32,
    pub rInitialOpacity: f32,
}
impl ::core::marker::Copy for TA_TRANSFORM_OPACITY {}
impl ::core::clone::Clone for TA_TRANSFORM_OPACITY {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TA_TRANSFORM_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATT_TRANSLATE_2D: TA_TRANSFORM_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATT_SCALE_2D: TA_TRANSFORM_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATT_OPACITY: TA_TRANSFORM_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TATT_CLIP: TA_TRANSFORM_TYPE = 3i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TBADDBITMAP {
    pub hInst: super::super::Foundation::HINSTANCE,
    pub nID: usize,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TBADDBITMAP {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TBADDBITMAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBBF_LARGE: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
pub struct TBBUTTON {
    pub iBitmap: i32,
    pub idCommand: i32,
    pub fsState: u8,
    pub fsStyle: u8,
    pub bReserved: [u8; 6],
    pub dwData: usize,
    pub iString: isize,
}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::marker::Copy for TBBUTTON {}
#[cfg(any(target_arch = "aarch64", target_arch = "x86_64"))]
impl ::core::clone::Clone for TBBUTTON {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
#[cfg(target_arch = "x86")]
pub struct TBBUTTON {
    pub iBitmap: i32,
    pub idCommand: i32,
    pub fsState: u8,
    pub fsStyle: u8,
    pub bReserved: [u8; 2],
    pub dwData: usize,
    pub iString: isize,
}
#[cfg(target_arch = "x86")]
impl ::core::marker::Copy for TBBUTTON {}
#[cfg(target_arch = "x86")]
impl ::core::clone::Clone for TBBUTTON {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TBBUTTONINFOA {
    pub cbSize: u32,
    pub dwMask: TBBUTTONINFOW_MASK,
    pub idCommand: i32,
    pub iImage: i32,
    pub fsState: u8,
    pub fsStyle: u8,
    pub cx: u16,
    pub lParam: usize,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchText: i32,
}
impl ::core::marker::Copy for TBBUTTONINFOA {}
impl ::core::clone::Clone for TBBUTTONINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TBBUTTONINFOW {
    pub cbSize: u32,
    pub dwMask: TBBUTTONINFOW_MASK,
    pub idCommand: i32,
    pub iImage: i32,
    pub fsState: u8,
    pub fsStyle: u8,
    pub cx: u16,
    pub lParam: usize,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchText: i32,
}
impl ::core::marker::Copy for TBBUTTONINFOW {}
impl ::core::clone::Clone for TBBUTTONINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TBBUTTONINFOW_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_BYINDEX: TBBUTTONINFOW_MASK = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_COMMAND: TBBUTTONINFOW_MASK = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_IMAGE: TBBUTTONINFOW_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_LPARAM: TBBUTTONINFOW_MASK = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_SIZE: TBBUTTONINFOW_MASK = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_STATE: TBBUTTONINFOW_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_STYLE: TBBUTTONINFOW_MASK = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIF_TEXT: TBBUTTONINFOW_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_BLENDICON: u32 = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_HILITEHOTTRACK: u32 = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_NOBACKGROUND: u32 = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_NOEDGES: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_NOETCHEDEFFECT: u32 = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_NOMARK: u32 = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_NOOFFSET: u32 = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCDRF_USECDCOLORS: u32 = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCD_CHANNEL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCD_THUMB: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBCD_TICS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBDDRET_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBDDRET_NODEFAULT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBDDRET_TREATPRESSED: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TBINSERTMARK {
    pub iButton: i32,
    pub dwFlags: TBINSERTMARK_FLAGS,
}
impl ::core::marker::Copy for TBINSERTMARK {}
impl ::core::clone::Clone for TBINSERTMARK {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TBINSERTMARK_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIMHT_NONE: TBINSERTMARK_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIMHT_AFTER: TBINSERTMARK_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBIMHT_BACKGROUND: TBINSERTMARK_FLAGS = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TBMETRICS {
    pub cbSize: u32,
    pub dwMask: u32,
    pub cxPad: i32,
    pub cyPad: i32,
    pub cxBarPad: i32,
    pub cyBarPad: i32,
    pub cxButtonSpacing: i32,
    pub cyButtonSpacing: i32,
}
impl ::core::marker::Copy for TBMETRICS {}
impl ::core::clone::Clone for TBMETRICS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBMF_BARPAD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBMF_BUTTONSPACING: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBMF_PAD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_CLEARSEL: u32 = 1043u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_CLEARTICS: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETBUDDY: u32 = 1057u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETCHANNELRECT: u32 = 1050u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETLINESIZE: u32 = 1048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETNUMTICS: u32 = 1040u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETPAGESIZE: u32 = 1046u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETPTICS: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETRANGEMAX: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETRANGEMIN: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETSELEND: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETSELSTART: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETTHUMBLENGTH: u32 = 1052u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETTHUMBRECT: u32 = 1049u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETTIC: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETTICPOS: u32 = 1039u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETTOOLTIPS: u32 = 1054u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETBUDDY: u32 = 1056u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETLINESIZE: u32 = 1047u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETPAGESIZE: u32 = 1045u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETPOS: u32 = 1029u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETPOSNOTIFY: u32 = 1058u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETRANGE: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETRANGEMAX: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETRANGEMIN: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETSEL: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETSELEND: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETSELSTART: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETTHUMBLENGTH: u32 = 1051u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETTIC: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETTICFREQ: u32 = 1044u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETTIPSIDE: u32 = 1055u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETTOOLTIPS: u32 = 1053u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBNRF_ENDCUSTOMIZE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBNRF_HIDEHELP: u32 = 1u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TBREPLACEBITMAP {
    pub hInstOld: super::super::Foundation::HINSTANCE,
    pub nIDOld: usize,
    pub hInstNew: super::super::Foundation::HINSTANCE,
    pub nIDNew: usize,
    pub nButtons: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TBREPLACEBITMAP {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TBREPLACEBITMAP {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Registry\"`*"]
#[cfg(feature = "Win32_System_Registry")]
pub struct TBSAVEPARAMSA {
    pub hkr: super::super::System::Registry::HKEY,
    pub pszSubKey: ::windows_sys::core::PCSTR,
    pub pszValueName: ::windows_sys::core::PCSTR,
}
#[cfg(feature = "Win32_System_Registry")]
impl ::core::marker::Copy for TBSAVEPARAMSA {}
#[cfg(feature = "Win32_System_Registry")]
impl ::core::clone::Clone for TBSAVEPARAMSA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_System_Registry\"`*"]
#[cfg(feature = "Win32_System_Registry")]
pub struct TBSAVEPARAMSW {
    pub hkr: super::super::System::Registry::HKEY,
    pub pszSubKey: ::windows_sys::core::PCWSTR,
    pub pszValueName: ::windows_sys::core::PCWSTR,
}
#[cfg(feature = "Win32_System_Registry")]
impl ::core::marker::Copy for TBSAVEPARAMSW {}
#[cfg(feature = "Win32_System_Registry")]
impl ::core::clone::Clone for TBSAVEPARAMSW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_CHECKED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_ELLIPSES: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_ENABLED: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_HIDDEN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_INDETERMINATE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_MARKED: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_PRESSED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTATE_WRAP: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_ALTDRAG: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_AUTOSIZE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_BUTTON: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_CHECK: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_CUSTOMERASE: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_DROPDOWN: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_EX_DOUBLEBUFFER: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_EX_DRAWDDARROWS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_EX_HIDECLIPPEDBUTTONS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_EX_MIXEDBUTTONS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_EX_MULTICOLUMN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_EX_VERTICAL: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_FLAT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_GROUP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_LIST: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_NOPREFIX: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_REGISTERDROP: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_SEP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_TOOLTIPS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_TRANSPARENT: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBSTYLE_WRAPABLE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_AUTOTICKS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_BOTH: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_BOTTOM: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_DOWNISLEFT: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_ENABLESELRANGE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_FIXEDLENGTH: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_HORZ: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_LEFT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_NOTHUMB: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_NOTICKS: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_NOTIFYBEFOREMOVE: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_REVERSED: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_RIGHT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_TOOLTIPS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_TOP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_TRANSPARENTBKGND: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBS_VERT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBTS_BOTTOM: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBTS_LEFT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBTS_RIGHT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TBTS_TOP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDBITMAP: u32 = 1043u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDBUTTONS: u32 = 1092u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDBUTTONSA: u32 = 1044u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDBUTTONSW: u32 = 1092u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDSTRING: u32 = 1101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDSTRINGA: u32 = 1052u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ADDSTRINGW: u32 = 1101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_AUTOSIZE: u32 = 1057u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_BOTTOM: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_BUTTONCOUNT: u32 = 1048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_BUTTONSTRUCTSIZE: u32 = 1054u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_CHANGEBITMAP: u32 = 1067u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_CHECKBUTTON: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_COMMANDTOINDEX: u32 = 1049u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_CUSTOMIZE: u32 = 1051u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_DELETEBUTTON: u32 = 1046u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ENABLEBUTTON: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ENDTRACK: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETANCHORHIGHLIGHT: u32 = 1098u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBITMAP: u32 = 1068u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBITMAPFLAGS: u32 = 1065u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTON: u32 = 1047u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONINFO: u32 = 1087u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONINFOA: u32 = 1089u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONINFOW: u32 = 1087u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONSIZE: u32 = 1082u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONTEXT: u32 = 1099u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONTEXTA: u32 = 1069u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETBUTTONTEXTW: u32 = 1099u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETCOLORSCHEME: u32 = 8195u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETDISABLEDIMAGELIST: u32 = 1079u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETEXTENDEDSTYLE: u32 = 1109u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETHOTIMAGELIST: u32 = 1077u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETHOTITEM: u32 = 1095u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETIDEALSIZE: u32 = 1123u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETIMAGELIST: u32 = 1073u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETIMAGELISTCOUNT: u32 = 1122u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETINSERTMARK: u32 = 1103u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETINSERTMARKCOLOR: u32 = 1113u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETITEMDROPDOWNRECT: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETITEMRECT: u32 = 1053u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETMAXSIZE: u32 = 1107u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETMETRICS: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETOBJECT: u32 = 1086u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETPADDING: u32 = 1110u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETPRESSEDIMAGELIST: u32 = 1129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETRECT: u32 = 1075u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETROWS: u32 = 1064u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETSTATE: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETSTRING: u32 = 1115u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETSTRINGA: u32 = 1116u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETSTRINGW: u32 = 1115u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETSTYLE: u32 = 1081u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETTEXTROWS: u32 = 1085u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETTOOLTIPS: u32 = 1059u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_HASACCELERATOR: u32 = 1119u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_HIDEBUTTON: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_HITTEST: u32 = 1093u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_INDETERMINATE: u32 = 1029u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_INSERTBUTTON: u32 = 1091u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_INSERTBUTTONA: u32 = 1045u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_INSERTBUTTONW: u32 = 1091u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_INSERTMARKHITTEST: u32 = 1105u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ISBUTTONCHECKED: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ISBUTTONENABLED: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ISBUTTONHIDDEN: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ISBUTTONHIGHLIGHTED: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ISBUTTONINDETERMINATE: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_ISBUTTONPRESSED: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_LINEDOWN: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_LINEUP: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_LOADIMAGES: u32 = 1074u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_MAPACCELERATOR: u32 = 1114u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_MAPACCELERATORA: u32 = 1102u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_MAPACCELERATORW: u32 = 1114u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_MARKBUTTON: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_MOVEBUTTON: u32 = 1106u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_PAGEDOWN: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_PAGEUP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_PRESSBUTTON: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_REPLACEBITMAP: u32 = 1070u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SAVERESTORE: u32 = 1100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SAVERESTOREA: u32 = 1050u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SAVERESTOREW: u32 = 1100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETANCHORHIGHLIGHT: u32 = 1097u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBITMAPSIZE: u32 = 1056u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBOUNDINGSIZE: u32 = 1117u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBUTTONINFO: u32 = 1088u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBUTTONINFOA: u32 = 1090u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBUTTONINFOW: u32 = 1088u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBUTTONSIZE: u32 = 1055u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETBUTTONWIDTH: u32 = 1083u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETCMDID: u32 = 1066u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETCOLORSCHEME: u32 = 8194u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETDISABLEDIMAGELIST: u32 = 1078u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETDRAWTEXTFLAGS: u32 = 1094u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETEXTENDEDSTYLE: u32 = 1108u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETHOTIMAGELIST: u32 = 1076u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETHOTITEM: u32 = 1096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETHOTITEM2: u32 = 1118u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETIMAGELIST: u32 = 1072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETINDENT: u32 = 1071u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETINSERTMARK: u32 = 1104u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETINSERTMARKCOLOR: u32 = 1112u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETLISTGAP: u32 = 1120u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETMAXTEXTROWS: u32 = 1084u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETMETRICS: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETPADDING: u32 = 1111u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETPARENT: u32 = 1061u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETPRESSEDIMAGELIST: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETROWS: u32 = 1063u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETSTATE: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETSTYLE: u32 = 1080u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETTOOLTIPS: u32 = 1060u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_SETWINDOWTHEME: u32 = 8203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_THUMBPOSITION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_THUMBTRACK: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TB_TOP: u32 = 6u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TCHITTESTINFO {
    pub pt: super::super::Foundation::POINT,
    pub flags: TCHITTESTINFO_FLAGS,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TCHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TCHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TCHITTESTINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCHT_NOWHERE: TCHITTESTINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCHT_ONITEM: TCHITTESTINFO_FLAGS = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCHT_ONITEMICON: TCHITTESTINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCHT_ONITEMLABEL: TCHITTESTINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIS_BUTTONPRESSED: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIS_HIGHLIGHTED: u32 = 2u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TCITEMA {
    pub mask: TCITEMHEADERA_MASK,
    pub dwState: u32,
    pub dwStateMask: u32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TCITEMA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TCITEMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TCITEMHEADERA {
    pub mask: TCITEMHEADERA_MASK,
    pub lpReserved1: u32,
    pub lpReserved2: u32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
}
impl ::core::marker::Copy for TCITEMHEADERA {}
impl ::core::clone::Clone for TCITEMHEADERA {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TCITEMHEADERA_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIF_IMAGE: TCITEMHEADERA_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIF_RTLREADING: TCITEMHEADERA_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIF_TEXT: TCITEMHEADERA_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIF_PARAM: TCITEMHEADERA_MASK = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCIF_STATE: TCITEMHEADERA_MASK = 16u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TCITEMHEADERW {
    pub mask: TCITEMHEADERA_MASK,
    pub lpReserved1: u32,
    pub lpReserved2: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
}
impl ::core::marker::Copy for TCITEMHEADERW {}
impl ::core::clone::Clone for TCITEMHEADERW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TCITEMW {
    pub mask: TCITEMHEADERA_MASK,
    pub dwState: u32,
    pub dwStateMask: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TCITEMW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TCITEMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_ADJUSTRECT: u32 = 4904u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_DELETEALLITEMS: u32 = 4873u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_DELETEITEM: u32 = 4872u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_DESELECTALL: u32 = 4914u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_FIRST: u32 = 4864u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETCURFOCUS: u32 = 4911u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETCURSEL: u32 = 4875u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETEXTENDEDSTYLE: u32 = 4917u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETIMAGELIST: u32 = 4866u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETITEM: u32 = 4924u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETITEMA: u32 = 4869u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETITEMCOUNT: u32 = 4868u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETITEMRECT: u32 = 4874u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETITEMW: u32 = 4924u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETROWCOUNT: u32 = 4908u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETTOOLTIPS: u32 = 4909u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_HIGHLIGHTITEM: u32 = 4915u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_HITTEST: u32 = 4877u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_INSERTITEM: u32 = 4926u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_INSERTITEMA: u32 = 4871u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_INSERTITEMW: u32 = 4926u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_REMOVEIMAGE: u32 = 4906u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETCURFOCUS: u32 = 4912u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETCURSEL: u32 = 4876u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETEXTENDEDSTYLE: u32 = 4916u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETIMAGELIST: u32 = 4867u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETITEM: u32 = 4925u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETITEMA: u32 = 4870u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETITEMEXTRA: u32 = 4878u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETITEMSIZE: u32 = 4905u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETITEMW: u32 = 4925u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETMINTABWIDTH: u32 = 4913u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETPADDING: u32 = 4907u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETTOOLTIPS: u32 = 4910u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_BOTTOM: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_BUTTONS: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_EX_FLATSEPARATORS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_EX_REGISTERDROP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_FIXEDWIDTH: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_FLATBUTTONS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_FOCUSNEVER: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_FOCUSONBUTTONDOWN: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_FORCEICONLEFT: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_FORCELABELLEFT: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_HOTTRACK: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_MULTILINE: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_MULTISELECT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_OWNERDRAWFIXED: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_RAGGEDRIGHT: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_RIGHT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_RIGHTJUSTIFY: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_SCROLLOPPOSITE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_SINGLELINE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_TABS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_TOOLTIPS: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TCS_VERTICAL: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TEXTSHADOWTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TST_NONE: TEXTSHADOWTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TST_SINGLE: TEXTSHADOWTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TST_CONTINUOUS: TEXTSHADOWTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type THEMESIZE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TS_MIN: THEMESIZE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TS_TRUE: THEMESIZE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TS_DRAW: THEMESIZE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type THEME_PROPERTY_SYMBOL_ID = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_RESERVEDLOW: THEME_PROPERTY_SYMBOL_ID = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_RESERVEDHIGH: THEME_PROPERTY_SYMBOL_ID = 7999u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DIBDATA: THEME_PROPERTY_SYMBOL_ID = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHDIBDATA: THEME_PROPERTY_SYMBOL_ID = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ENUM: THEME_PROPERTY_SYMBOL_ID = 200u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_STRING: THEME_PROPERTY_SYMBOL_ID = 201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INT: THEME_PROPERTY_SYMBOL_ID = 202u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BOOL: THEME_PROPERTY_SYMBOL_ID = 203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COLOR: THEME_PROPERTY_SYMBOL_ID = 204u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MARGINS: THEME_PROPERTY_SYMBOL_ID = 205u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FILENAME: THEME_PROPERTY_SYMBOL_ID = 206u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SIZE: THEME_PROPERTY_SYMBOL_ID = 207u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_POSITION: THEME_PROPERTY_SYMBOL_ID = 208u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_RECT: THEME_PROPERTY_SYMBOL_ID = 209u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FONT: THEME_PROPERTY_SYMBOL_ID = 210u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INTLIST: THEME_PROPERTY_SYMBOL_ID = 211u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HBITMAP: THEME_PROPERTY_SYMBOL_ID = 212u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DISKSTREAM: THEME_PROPERTY_SYMBOL_ID = 213u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_STREAM: THEME_PROPERTY_SYMBOL_ID = 214u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BITMAPREF: THEME_PROPERTY_SYMBOL_ID = 215u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FLOAT: THEME_PROPERTY_SYMBOL_ID = 216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FLOATLIST: THEME_PROPERTY_SYMBOL_ID = 217u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COLORSCHEMES: THEME_PROPERTY_SYMBOL_ID = 401u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SIZES: THEME_PROPERTY_SYMBOL_ID = 402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CHARSET: THEME_PROPERTY_SYMBOL_ID = 403u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_NAME: THEME_PROPERTY_SYMBOL_ID = 600u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DISPLAYNAME: THEME_PROPERTY_SYMBOL_ID = 601u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOOLTIP: THEME_PROPERTY_SYMBOL_ID = 602u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COMPANY: THEME_PROPERTY_SYMBOL_ID = 603u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_AUTHOR: THEME_PROPERTY_SYMBOL_ID = 604u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COPYRIGHT: THEME_PROPERTY_SYMBOL_ID = 605u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_URL: THEME_PROPERTY_SYMBOL_ID = 606u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_VERSION: THEME_PROPERTY_SYMBOL_ID = 607u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DESCRIPTION: THEME_PROPERTY_SYMBOL_ID = 608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRST_RCSTRING_NAME: THEME_PROPERTY_SYMBOL_ID = 601u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LAST_RCSTRING_NAME: THEME_PROPERTY_SYMBOL_ID = 608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CAPTIONFONT: THEME_PROPERTY_SYMBOL_ID = 801u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SMALLCAPTIONFONT: THEME_PROPERTY_SYMBOL_ID = 802u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENUFONT: THEME_PROPERTY_SYMBOL_ID = 803u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_STATUSFONT: THEME_PROPERTY_SYMBOL_ID = 804u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MSGBOXFONT: THEME_PROPERTY_SYMBOL_ID = 805u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ICONTITLEFONT: THEME_PROPERTY_SYMBOL_ID = 806u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HEADING1FONT: THEME_PROPERTY_SYMBOL_ID = 807u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HEADING2FONT: THEME_PROPERTY_SYMBOL_ID = 808u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BODYFONT: THEME_PROPERTY_SYMBOL_ID = 809u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRSTFONT: THEME_PROPERTY_SYMBOL_ID = 801u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTFONT: THEME_PROPERTY_SYMBOL_ID = 809u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FLATMENUS: THEME_PROPERTY_SYMBOL_ID = 1001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRSTBOOL: THEME_PROPERTY_SYMBOL_ID = 1001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTBOOL: THEME_PROPERTY_SYMBOL_ID = 1001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SIZINGBORDERWIDTH: THEME_PROPERTY_SYMBOL_ID = 1201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SCROLLBARWIDTH: THEME_PROPERTY_SYMBOL_ID = 1202u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SCROLLBARHEIGHT: THEME_PROPERTY_SYMBOL_ID = 1203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CAPTIONBARWIDTH: THEME_PROPERTY_SYMBOL_ID = 1204u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CAPTIONBARHEIGHT: THEME_PROPERTY_SYMBOL_ID = 1205u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SMCAPTIONBARWIDTH: THEME_PROPERTY_SYMBOL_ID = 1206u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SMCAPTIONBARHEIGHT: THEME_PROPERTY_SYMBOL_ID = 1207u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENUBARWIDTH: THEME_PROPERTY_SYMBOL_ID = 1208u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENUBARHEIGHT: THEME_PROPERTY_SYMBOL_ID = 1209u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_PADDEDBORDERWIDTH: THEME_PROPERTY_SYMBOL_ID = 1210u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRSTSIZE: THEME_PROPERTY_SYMBOL_ID = 1201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTSIZE: THEME_PROPERTY_SYMBOL_ID = 1210u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINCOLORDEPTH: THEME_PROPERTY_SYMBOL_ID = 1301u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRSTINT: THEME_PROPERTY_SYMBOL_ID = 1301u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTINT: THEME_PROPERTY_SYMBOL_ID = 1301u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CSSNAME: THEME_PROPERTY_SYMBOL_ID = 1401u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_XMLNAME: THEME_PROPERTY_SYMBOL_ID = 1402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTUPDATED: THEME_PROPERTY_SYMBOL_ID = 1403u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ALIAS: THEME_PROPERTY_SYMBOL_ID = 1404u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRSTSTRING: THEME_PROPERTY_SYMBOL_ID = 1401u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTSTRING: THEME_PROPERTY_SYMBOL_ID = 1404u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SCROLLBAR: THEME_PROPERTY_SYMBOL_ID = 1601u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BACKGROUND: THEME_PROPERTY_SYMBOL_ID = 1602u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ACTIVECAPTION: THEME_PROPERTY_SYMBOL_ID = 1603u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INACTIVECAPTION: THEME_PROPERTY_SYMBOL_ID = 1604u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENU: THEME_PROPERTY_SYMBOL_ID = 1605u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_WINDOW: THEME_PROPERTY_SYMBOL_ID = 1606u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_WINDOWFRAME: THEME_PROPERTY_SYMBOL_ID = 1607u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENUTEXT: THEME_PROPERTY_SYMBOL_ID = 1608u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_WINDOWTEXT: THEME_PROPERTY_SYMBOL_ID = 1609u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CAPTIONTEXT: THEME_PROPERTY_SYMBOL_ID = 1610u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ACTIVEBORDER: THEME_PROPERTY_SYMBOL_ID = 1611u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INACTIVEBORDER: THEME_PROPERTY_SYMBOL_ID = 1612u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_APPWORKSPACE: THEME_PROPERTY_SYMBOL_ID = 1613u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HIGHLIGHT: THEME_PROPERTY_SYMBOL_ID = 1614u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HIGHLIGHTTEXT: THEME_PROPERTY_SYMBOL_ID = 1615u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BTNFACE: THEME_PROPERTY_SYMBOL_ID = 1616u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BTNSHADOW: THEME_PROPERTY_SYMBOL_ID = 1617u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRAYTEXT: THEME_PROPERTY_SYMBOL_ID = 1618u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BTNTEXT: THEME_PROPERTY_SYMBOL_ID = 1619u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INACTIVECAPTIONTEXT: THEME_PROPERTY_SYMBOL_ID = 1620u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BTNHIGHLIGHT: THEME_PROPERTY_SYMBOL_ID = 1621u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DKSHADOW3D: THEME_PROPERTY_SYMBOL_ID = 1622u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LIGHT3D: THEME_PROPERTY_SYMBOL_ID = 1623u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INFOTEXT: THEME_PROPERTY_SYMBOL_ID = 1624u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INFOBK: THEME_PROPERTY_SYMBOL_ID = 1625u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BUTTONALTERNATEFACE: THEME_PROPERTY_SYMBOL_ID = 1626u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HOTTRACKING: THEME_PROPERTY_SYMBOL_ID = 1627u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTACTIVECAPTION: THEME_PROPERTY_SYMBOL_ID = 1628u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTINACTIVECAPTION: THEME_PROPERTY_SYMBOL_ID = 1629u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENUHILIGHT: THEME_PROPERTY_SYMBOL_ID = 1630u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MENUBAR: THEME_PROPERTY_SYMBOL_ID = 1631u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FIRSTCOLOR: THEME_PROPERTY_SYMBOL_ID = 1601u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LASTCOLOR: THEME_PROPERTY_SYMBOL_ID = 1631u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMHUE1: THEME_PROPERTY_SYMBOL_ID = 1801u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMHUE2: THEME_PROPERTY_SYMBOL_ID = 1802u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMHUE3: THEME_PROPERTY_SYMBOL_ID = 1803u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMHUE4: THEME_PROPERTY_SYMBOL_ID = 1804u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMHUE5: THEME_PROPERTY_SYMBOL_ID = 1805u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOHUE1: THEME_PROPERTY_SYMBOL_ID = 1806u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOHUE2: THEME_PROPERTY_SYMBOL_ID = 1807u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOHUE3: THEME_PROPERTY_SYMBOL_ID = 1808u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOHUE4: THEME_PROPERTY_SYMBOL_ID = 1809u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOHUE5: THEME_PROPERTY_SYMBOL_ID = 1810u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMCOLOR1: THEME_PROPERTY_SYMBOL_ID = 2001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMCOLOR2: THEME_PROPERTY_SYMBOL_ID = 2002u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMCOLOR3: THEME_PROPERTY_SYMBOL_ID = 2003u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMCOLOR4: THEME_PROPERTY_SYMBOL_ID = 2004u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FROMCOLOR5: THEME_PROPERTY_SYMBOL_ID = 2005u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOCOLOR1: THEME_PROPERTY_SYMBOL_ID = 2006u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOCOLOR2: THEME_PROPERTY_SYMBOL_ID = 2007u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOCOLOR3: THEME_PROPERTY_SYMBOL_ID = 2008u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOCOLOR4: THEME_PROPERTY_SYMBOL_ID = 2009u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TOCOLOR5: THEME_PROPERTY_SYMBOL_ID = 2010u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TRANSPARENT: THEME_PROPERTY_SYMBOL_ID = 2201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_AUTOSIZE: THEME_PROPERTY_SYMBOL_ID = 2202u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BORDERONLY: THEME_PROPERTY_SYMBOL_ID = 2203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COMPOSITED: THEME_PROPERTY_SYMBOL_ID = 2204u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BGFILL: THEME_PROPERTY_SYMBOL_ID = 2205u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHTRANSPARENT: THEME_PROPERTY_SYMBOL_ID = 2206u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHONLY: THEME_PROPERTY_SYMBOL_ID = 2207u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ALWAYSSHOWSIZINGBAR: THEME_PROPERTY_SYMBOL_ID = 2208u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MIRRORIMAGE: THEME_PROPERTY_SYMBOL_ID = 2209u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_UNIFORMSIZING: THEME_PROPERTY_SYMBOL_ID = 2210u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_INTEGRALSIZING: THEME_PROPERTY_SYMBOL_ID = 2211u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SOURCEGROW: THEME_PROPERTY_SYMBOL_ID = 2212u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SOURCESHRINK: THEME_PROPERTY_SYMBOL_ID = 2213u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DRAWBORDERS: THEME_PROPERTY_SYMBOL_ID = 2214u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_NOETCHEDEFFECT: THEME_PROPERTY_SYMBOL_ID = 2215u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTAPPLYOVERLAY: THEME_PROPERTY_SYMBOL_ID = 2216u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTGLOW: THEME_PROPERTY_SYMBOL_ID = 2217u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTITALIC: THEME_PROPERTY_SYMBOL_ID = 2218u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COMPOSITEDOPAQUE: THEME_PROPERTY_SYMBOL_ID = 2219u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_LOCALIZEDMIRRORIMAGE: THEME_PROPERTY_SYMBOL_ID = 2220u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGECOUNT: THEME_PROPERTY_SYMBOL_ID = 2401u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ALPHALEVEL: THEME_PROPERTY_SYMBOL_ID = 2402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BORDERSIZE: THEME_PROPERTY_SYMBOL_ID = 2403u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ROUNDCORNERWIDTH: THEME_PROPERTY_SYMBOL_ID = 2404u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ROUNDCORNERHEIGHT: THEME_PROPERTY_SYMBOL_ID = 2405u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTRATIO1: THEME_PROPERTY_SYMBOL_ID = 2406u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTRATIO2: THEME_PROPERTY_SYMBOL_ID = 2407u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTRATIO3: THEME_PROPERTY_SYMBOL_ID = 2408u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTRATIO4: THEME_PROPERTY_SYMBOL_ID = 2409u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTRATIO5: THEME_PROPERTY_SYMBOL_ID = 2410u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_PROGRESSCHUNKSIZE: THEME_PROPERTY_SYMBOL_ID = 2411u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_PROGRESSSPACESIZE: THEME_PROPERTY_SYMBOL_ID = 2412u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SATURATION: THEME_PROPERTY_SYMBOL_ID = 2413u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTBORDERSIZE: THEME_PROPERTY_SYMBOL_ID = 2414u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ALPHATHRESHOLD: THEME_PROPERTY_SYMBOL_ID = 2415u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_WIDTH: THEME_PROPERTY_SYMBOL_ID = 2416u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HEIGHT: THEME_PROPERTY_SYMBOL_ID = 2417u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHINDEX: THEME_PROPERTY_SYMBOL_ID = 2418u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TRUESIZESTRETCHMARK: THEME_PROPERTY_SYMBOL_ID = 2419u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI1: THEME_PROPERTY_SYMBOL_ID = 2420u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI2: THEME_PROPERTY_SYMBOL_ID = 2421u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI3: THEME_PROPERTY_SYMBOL_ID = 2422u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI4: THEME_PROPERTY_SYMBOL_ID = 2423u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI5: THEME_PROPERTY_SYMBOL_ID = 2424u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTGLOWSIZE: THEME_PROPERTY_SYMBOL_ID = 2425u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FRAMESPERSECOND: THEME_PROPERTY_SYMBOL_ID = 2426u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_PIXELSPERFRAME: THEME_PROPERTY_SYMBOL_ID = 2427u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ANIMATIONDELAY: THEME_PROPERTY_SYMBOL_ID = 2428u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLOWINTENSITY: THEME_PROPERTY_SYMBOL_ID = 2429u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_OPACITY: THEME_PROPERTY_SYMBOL_ID = 2430u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COLORIZATIONCOLOR: THEME_PROPERTY_SYMBOL_ID = 2431u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_COLORIZATIONOPACITY: THEME_PROPERTY_SYMBOL_ID = 2432u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI6: THEME_PROPERTY_SYMBOL_ID = 2433u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINDPI7: THEME_PROPERTY_SYMBOL_ID = 2434u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHFONT: THEME_PROPERTY_SYMBOL_ID = 2601u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE: THEME_PROPERTY_SYMBOL_ID = 3001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE1: THEME_PROPERTY_SYMBOL_ID = 3002u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE2: THEME_PROPERTY_SYMBOL_ID = 3003u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE3: THEME_PROPERTY_SYMBOL_ID = 3004u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE4: THEME_PROPERTY_SYMBOL_ID = 3005u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE5: THEME_PROPERTY_SYMBOL_ID = 3006u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHIMAGEFILE: THEME_PROPERTY_SYMBOL_ID = 3008u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE6: THEME_PROPERTY_SYMBOL_ID = 3009u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGEFILE7: THEME_PROPERTY_SYMBOL_ID = 3010u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXT: THEME_PROPERTY_SYMBOL_ID = 3201u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CLASSICVALUE: THEME_PROPERTY_SYMBOL_ID = 3202u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_OFFSET: THEME_PROPERTY_SYMBOL_ID = 3401u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTSHADOWOFFSET: THEME_PROPERTY_SYMBOL_ID = 3402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE: THEME_PROPERTY_SYMBOL_ID = 3403u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE1: THEME_PROPERTY_SYMBOL_ID = 3404u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE2: THEME_PROPERTY_SYMBOL_ID = 3405u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE3: THEME_PROPERTY_SYMBOL_ID = 3406u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE4: THEME_PROPERTY_SYMBOL_ID = 3407u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE5: THEME_PROPERTY_SYMBOL_ID = 3408u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_NORMALSIZE: THEME_PROPERTY_SYMBOL_ID = 3409u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE6: THEME_PROPERTY_SYMBOL_ID = 3410u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_MINSIZE7: THEME_PROPERTY_SYMBOL_ID = 3411u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SIZINGMARGINS: THEME_PROPERTY_SYMBOL_ID = 3601u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CONTENTMARGINS: THEME_PROPERTY_SYMBOL_ID = 3602u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CAPTIONMARGINS: THEME_PROPERTY_SYMBOL_ID = 3603u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BORDERCOLOR: THEME_PROPERTY_SYMBOL_ID = 3801u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FILLCOLOR: THEME_PROPERTY_SYMBOL_ID = 3802u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3803u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_EDGELIGHTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3804u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_EDGEHIGHLIGHTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3805u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_EDGESHADOWCOLOR: THEME_PROPERTY_SYMBOL_ID = 3806u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_EDGEDKSHADOWCOLOR: THEME_PROPERTY_SYMBOL_ID = 3807u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_EDGEFILLCOLOR: THEME_PROPERTY_SYMBOL_ID = 3808u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TRANSPARENTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3809u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTCOLOR1: THEME_PROPERTY_SYMBOL_ID = 3810u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTCOLOR2: THEME_PROPERTY_SYMBOL_ID = 3811u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTCOLOR3: THEME_PROPERTY_SYMBOL_ID = 3812u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTCOLOR4: THEME_PROPERTY_SYMBOL_ID = 3813u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GRADIENTCOLOR5: THEME_PROPERTY_SYMBOL_ID = 3814u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SHADOWCOLOR: THEME_PROPERTY_SYMBOL_ID = 3815u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLOWCOLOR: THEME_PROPERTY_SYMBOL_ID = 3816u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTBORDERCOLOR: THEME_PROPERTY_SYMBOL_ID = 3817u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTSHADOWCOLOR: THEME_PROPERTY_SYMBOL_ID = 3818u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHTEXTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3819u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHTRANSPARENTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3820u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FILLCOLORHINT: THEME_PROPERTY_SYMBOL_ID = 3821u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BORDERCOLORHINT: THEME_PROPERTY_SYMBOL_ID = 3822u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ACCENTCOLORHINT: THEME_PROPERTY_SYMBOL_ID = 3823u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTCOLORHINT: THEME_PROPERTY_SYMBOL_ID = 3824u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HEADING1TEXTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3825u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HEADING2TEXTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3826u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BODYTEXTCOLOR: THEME_PROPERTY_SYMBOL_ID = 3827u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BGTYPE: THEME_PROPERTY_SYMBOL_ID = 4001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BORDERTYPE: THEME_PROPERTY_SYMBOL_ID = 4002u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_FILLTYPE: THEME_PROPERTY_SYMBOL_ID = 4003u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SIZINGTYPE: THEME_PROPERTY_SYMBOL_ID = 4004u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_HALIGN: THEME_PROPERTY_SYMBOL_ID = 4005u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CONTENTALIGNMENT: THEME_PROPERTY_SYMBOL_ID = 4006u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_VALIGN: THEME_PROPERTY_SYMBOL_ID = 4007u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_OFFSETTYPE: THEME_PROPERTY_SYMBOL_ID = 4008u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ICONEFFECT: THEME_PROPERTY_SYMBOL_ID = 4009u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TEXTSHADOWTYPE: THEME_PROPERTY_SYMBOL_ID = 4010u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGELAYOUT: THEME_PROPERTY_SYMBOL_ID = 4011u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHTYPE: THEME_PROPERTY_SYMBOL_ID = 4012u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_IMAGESELECTTYPE: THEME_PROPERTY_SYMBOL_ID = 4013u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_GLYPHFONTSIZINGTYPE: THEME_PROPERTY_SYMBOL_ID = 4014u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TRUESIZESCALINGTYPE: THEME_PROPERTY_SYMBOL_ID = 4015u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_USERPICTURE: THEME_PROPERTY_SYMBOL_ID = 5001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_DEFAULTPANESIZE: THEME_PROPERTY_SYMBOL_ID = 5002u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_BLENDCOLOR: THEME_PROPERTY_SYMBOL_ID = 5003u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_CUSTOMSPLITRECT: THEME_PROPERTY_SYMBOL_ID = 5004u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ANIMATIONBUTTONRECT: THEME_PROPERTY_SYMBOL_ID = 5005u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ANIMATIONDURATION: THEME_PROPERTY_SYMBOL_ID = 5006u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_TRANSITIONDURATIONS: THEME_PROPERTY_SYMBOL_ID = 6000u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_SCALEDBACKGROUND: THEME_PROPERTY_SYMBOL_ID = 7001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ATLASIMAGE: THEME_PROPERTY_SYMBOL_ID = 8000u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ATLASINPUTIMAGE: THEME_PROPERTY_SYMBOL_ID = 8001u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TMT_ATLASRECT: THEME_PROPERTY_SYMBOL_ID = 8002u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TOOLBARCLASSNAME: &str = "ToolbarWindow32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TOOLBARCLASSNAMEA: &str = "ToolbarWindow32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TOOLBARCLASSNAMEW: &str = "ToolbarWindow32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TOOLTIPS_CLASS: &str = "tooltips_class32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TOOLTIPS_CLASSA: &str = "tooltips_class32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TOOLTIPS_CLASSW: &str = "tooltips_class32";
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TOUCH_HIT_TESTING_INPUT {
    pub pointerId: u32,
    pub point: super::super::Foundation::POINT,
    pub boundingBox: super::super::Foundation::RECT,
    pub nonOccludedBoundingBox: super::super::Foundation::RECT,
    pub orientation: u32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TOUCH_HIT_TESTING_INPUT {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TOUCH_HIT_TESTING_INPUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TOUCH_HIT_TESTING_PROXIMITY_EVALUATION {
    pub score: u16,
    pub adjustedPoint: super::super::Foundation::POINT,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TOUCH_HIT_TESTING_PROXIMITY_EVALUATION {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TOUCH_HIT_TESTING_PROXIMITY_EVALUATION {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TRACKBAR_CLASS: &str = "msctls_trackbar32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TRACKBAR_CLASSA: &str = "msctls_trackbar32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TRACKBAR_CLASSW: &str = "msctls_trackbar32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TRAILINGGRIDCELLSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_HOT: TRAILINGGRIDCELLSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_HASSTATE: TRAILINGGRIDCELLSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_HASSTATEHOT: TRAILINGGRIDCELLSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_TODAY: TRAILINGGRIDCELLSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_TODAYSELECTED: TRAILINGGRIDCELLSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_SELECTED: TRAILINGGRIDCELLSTATES = 6i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGC_SELECTEDHOT: TRAILINGGRIDCELLSTATES = 7i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TRAILINGGRIDCELLUPPERSTATES = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGCU_HOT: TRAILINGGRIDCELLUPPERSTATES = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGCU_HASSTATE: TRAILINGGRIDCELLUPPERSTATES = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGCU_HASSTATEHOT: TRAILINGGRIDCELLUPPERSTATES = 3i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGCU_SELECTED: TRAILINGGRIDCELLUPPERSTATES = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const MCTGCU_SELECTEDHOT: TRAILINGGRIDCELLUPPERSTATES = 5i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TRAYNOTIFYPARTS = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TNP_BACKGROUND: TRAYNOTIFYPARTS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TNP_ANIMBACKGROUND: TRAYNOTIFYPARTS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TRUESIZESCALINGTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TSST_NONE: TRUESIZESCALINGTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TSST_SIZE: TRUESIZESCALINGTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TSST_DPI: TRUESIZESCALINGTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTDT_AUTOMATIC: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTDT_AUTOPOP: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTDT_INITIAL: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTDT_RESHOW: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_DI_SETITEM: u32 = 32768u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct TTGETTITLE {
    pub dwSize: u32,
    pub uTitleBitmap: u32,
    pub cch: u32,
    pub pszTitle: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for TTGETTITLE {}
impl ::core::clone::Clone for TTGETTITLE {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TTHITTESTINFOA {
    pub hwnd: super::super::Foundation::HWND,
    pub pt: super::super::Foundation::POINT,
    pub ti: TTTOOLINFOA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TTHITTESTINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TTHITTESTINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TTHITTESTINFOW {
    pub hwnd: super::super::Foundation::HWND,
    pub pt: super::super::Foundation::POINT,
    pub ti: TTTOOLINFOW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TTHITTESTINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TTHITTESTINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ACTIVATE: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ADDTOOL: u32 = 1074u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ADDTOOLA: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ADDTOOLW: u32 = 1074u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ADJUSTRECT: u32 = 1055u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_DELTOOL: u32 = 1075u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_DELTOOLA: u32 = 1029u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_DELTOOLW: u32 = 1075u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ENUMTOOLS: u32 = 1082u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ENUMTOOLSA: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_ENUMTOOLSW: u32 = 1082u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETBUBBLESIZE: u32 = 1054u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETCURRENTTOOL: u32 = 1083u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETCURRENTTOOLA: u32 = 1039u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETCURRENTTOOLW: u32 = 1083u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETDELAYTIME: u32 = 1045u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETMARGIN: u32 = 1051u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETMAXTIPWIDTH: u32 = 1049u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTEXT: u32 = 1080u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTEXTA: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTEXTW: u32 = 1080u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTIPBKCOLOR: u32 = 1046u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTIPTEXTCOLOR: u32 = 1047u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTITLE: u32 = 1059u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTOOLCOUNT: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTOOLINFO: u32 = 1077u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTOOLINFOA: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_GETTOOLINFOW: u32 = 1077u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_HITTEST: u32 = 1079u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_HITTESTA: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_HITTESTW: u32 = 1079u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_NEWTOOLRECT: u32 = 1076u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_NEWTOOLRECTA: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_NEWTOOLRECTW: u32 = 1076u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_POP: u32 = 1052u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_POPUP: u32 = 1058u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_RELAYEVENT: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETDELAYTIME: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETMARGIN: u32 = 1050u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETMAXTIPWIDTH: u32 = 1048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTIPBKCOLOR: u32 = 1043u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTIPTEXTCOLOR: u32 = 1044u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTITLE: u32 = 1057u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTITLEA: u32 = 1056u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTITLEW: u32 = 1057u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTOOLINFO: u32 = 1078u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTOOLINFOA: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETTOOLINFOW: u32 = 1078u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_SETWINDOWTHEME: u32 = 8203u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_TRACKACTIVATE: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_TRACKPOSITION: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_UPDATE: u32 = 1053u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_UPDATETIPTEXT: u32 = 1081u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_UPDATETIPTEXTA: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_UPDATETIPTEXTW: u32 = 1081u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTM_WINDOWFROMPOINT: u32 = 1040u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_ALWAYSTIP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_BALLOON: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_CLOSE: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_NOANIMATE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_NOFADE: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_NOPREFIX: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTS_USEVISUALSTYLE: u32 = 256u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TTTOOLINFOA {
    pub cbSize: u32,
    pub uFlags: TTTOOLINFO_FLAGS,
    pub hwnd: super::super::Foundation::HWND,
    pub uId: usize,
    pub rect: super::super::Foundation::RECT,
    pub hinst: super::super::Foundation::HINSTANCE,
    pub lpszText: ::windows_sys::core::PSTR,
    pub lParam: super::super::Foundation::LPARAM,
    pub lpReserved: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TTTOOLINFOA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TTTOOLINFOA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TTTOOLINFOW {
    pub cbSize: u32,
    pub uFlags: TTTOOLINFO_FLAGS,
    pub hwnd: super::super::Foundation::HWND,
    pub uId: usize,
    pub rect: super::super::Foundation::RECT,
    pub hinst: super::super::Foundation::HINSTANCE,
    pub lpszText: ::windows_sys::core::PWSTR,
    pub lParam: super::super::Foundation::LPARAM,
    pub lpReserved: *mut ::core::ffi::c_void,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TTTOOLINFOW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TTTOOLINFOW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TTTOOLINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_ABSOLUTE: TTTOOLINFO_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_CENTERTIP: TTTOOLINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_IDISHWND: TTTOOLINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_PARSELINKS: TTTOOLINFO_FLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_RTLREADING: TTTOOLINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_SUBCLASS: TTTOOLINFO_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_TRACK: TTTOOLINFO_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TTF_TRANSPARENT: TTTOOLINFO_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVCDRF_NOIMAGES: u32 = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVC_BYKEYBOARD: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVC_BYMOUSE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVC_UNKNOWN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVE_COLLAPSE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVE_COLLAPSERESET: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVE_EXPAND: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVE_EXPANDPARTIAL: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVE_TOGGLE: u32 = 3u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVGETITEMPARTRECTINFO {
    pub hti: HTREEITEM,
    pub prc: *mut super::super::Foundation::RECT,
    pub partID: TVITEMPART,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVGETITEMPARTRECTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVGETITEMPARTRECTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_CARET: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_CHILD: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_DROPHILITE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_FIRSTVISIBLE: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_LASTVISIBLE: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_NEXT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_NEXTSELECTED: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_NEXTVISIBLE: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_PARENT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_PREVIOUS: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_PREVIOUSVISIBLE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGN_ROOT: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVHITTESTINFO {
    pub pt: super::super::Foundation::POINT,
    pub flags: TVHITTESTINFO_FLAGS,
    pub hItem: HTREEITEM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVHITTESTINFO {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVHITTESTINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TVHITTESTINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ABOVE: TVHITTESTINFO_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_BELOW: TVHITTESTINFO_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_NOWHERE: TVHITTESTINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEM: TVHITTESTINFO_FLAGS = 70u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEMBUTTON: TVHITTESTINFO_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEMICON: TVHITTESTINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEMINDENT: TVHITTESTINFO_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEMLABEL: TVHITTESTINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEMRIGHT: TVHITTESTINFO_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_ONITEMSTATEICON: TVHITTESTINFO_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_TOLEFT: TVHITTESTINFO_FLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVHT_TORIGHT: TVHITTESTINFO_FLAGS = 1024u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVINSERTSTRUCTA {
    pub hParent: HTREEITEM,
    pub hInsertAfter: HTREEITEM,
    pub Anonymous: TVINSERTSTRUCTA_0,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVINSERTSTRUCTA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVINSERTSTRUCTA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union TVINSERTSTRUCTA_0 {
    pub itemex: TVITEMEXA,
    pub item: TVITEMA,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVINSERTSTRUCTA_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVINSERTSTRUCTA_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVINSERTSTRUCTW {
    pub hParent: HTREEITEM,
    pub hInsertAfter: HTREEITEM,
    pub Anonymous: TVINSERTSTRUCTW_0,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVINSERTSTRUCTW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVINSERTSTRUCTW {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub union TVINSERTSTRUCTW_0 {
    pub itemex: TVITEMEXW,
    pub item: TVITEMW,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVINSERTSTRUCTW_0 {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVINSERTSTRUCTW_0 {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_BOLD: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_CUT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_DROPHILITED: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_EXPANDED: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_EXPANDEDONCE: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_EXPANDPARTIAL: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_EX_ALL: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_EX_DISABLED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_EX_FLAT: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_OVERLAYMASK: u32 = 3840u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_SELECTED: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_STATEIMAGEMASK: u32 = 61440u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIS_USERMASK: u32 = 61440u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVITEMA {
    pub mask: TVITEM_MASK,
    pub hItem: HTREEITEM,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub cChildren: TVITEMEXW_CHILDREN,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVITEMA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVITEMA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVITEMEXA {
    pub mask: TVITEM_MASK,
    pub hItem: HTREEITEM,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: ::windows_sys::core::PSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub cChildren: TVITEMEXW_CHILDREN,
    pub lParam: super::super::Foundation::LPARAM,
    pub iIntegral: i32,
    pub uStateEx: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub iExpandedImage: i32,
    pub iReserved: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVITEMEXA {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVITEMEXA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVITEMEXW {
    pub mask: TVITEM_MASK,
    pub hItem: HTREEITEM,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub cChildren: TVITEMEXW_CHILDREN,
    pub lParam: super::super::Foundation::LPARAM,
    pub iIntegral: i32,
    pub uStateEx: u32,
    pub hwnd: super::super::Foundation::HWND,
    pub iExpandedImage: i32,
    pub iReserved: i32,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVITEMEXW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVITEMEXW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TVITEMEXW_CHILDREN = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_ZERO: TVITEMEXW_CHILDREN = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_ONE_OR_MORE: TVITEMEXW_CHILDREN = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_CHILDRENCALLBACK: TVITEMEXW_CHILDREN = -1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const I_CHILDRENAUTO: TVITEMEXW_CHILDREN = -2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TVITEMPART = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVGIPR_BUTTON: TVITEMPART = 1i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVITEMW {
    pub mask: TVITEM_MASK,
    pub hItem: HTREEITEM,
    pub state: u32,
    pub stateMask: u32,
    pub pszText: ::windows_sys::core::PWSTR,
    pub cchTextMax: i32,
    pub iImage: i32,
    pub iSelectedImage: i32,
    pub cChildren: TVITEMEXW_CHILDREN,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVITEMW {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVITEMW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type TVITEM_MASK = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_CHILDREN: TVITEM_MASK = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_DI_SETITEM: TVITEM_MASK = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_HANDLE: TVITEM_MASK = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_IMAGE: TVITEM_MASK = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_PARAM: TVITEM_MASK = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_SELECTEDIMAGE: TVITEM_MASK = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_STATE: TVITEM_MASK = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_TEXT: TVITEM_MASK = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_EXPANDEDIMAGE: TVITEM_MASK = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_INTEGRAL: TVITEM_MASK = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVIF_STATEEX: TVITEM_MASK = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVI_FIRST: HTREEITEM = -65535i32 as _;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVI_LAST: HTREEITEM = -65534i32 as _;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVI_ROOT: HTREEITEM = -65536i32 as _;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVI_SORT: HTREEITEM = -65533i32 as _;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_CREATEDRAGIMAGE: u32 = 4370u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_DELETEITEM: u32 = 4353u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_EDITLABEL: u32 = 4417u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_EDITLABELA: u32 = 4366u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_EDITLABELW: u32 = 4417u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_ENDEDITLABELNOW: u32 = 4374u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_ENSUREVISIBLE: u32 = 4372u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_EXPAND: u32 = 4354u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETBKCOLOR: u32 = 4383u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETCOUNT: u32 = 4357u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETEDITCONTROL: u32 = 4367u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETEXTENDEDSTYLE: u32 = 4397u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETIMAGELIST: u32 = 4360u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETINDENT: u32 = 4358u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETINSERTMARKCOLOR: u32 = 4390u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETISEARCHSTRING: u32 = 4416u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETISEARCHSTRINGA: u32 = 4375u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETISEARCHSTRINGW: u32 = 4416u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEM: u32 = 4414u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEMA: u32 = 4364u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEMHEIGHT: u32 = 4380u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEMPARTRECT: u32 = 4424u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEMRECT: u32 = 4356u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEMSTATE: u32 = 4391u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETITEMW: u32 = 4414u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETLINECOLOR: u32 = 4393u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETNEXTITEM: u32 = 4362u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETSCROLLTIME: u32 = 4386u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETSELECTEDCOUNT: u32 = 4422u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETTEXTCOLOR: u32 = 4384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETTOOLTIPS: u32 = 4377u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_GETVISIBLECOUNT: u32 = 4368u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_HITTEST: u32 = 4369u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_INSERTITEM: u32 = 4402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_INSERTITEMA: u32 = 4352u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_INSERTITEMW: u32 = 4402u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_MAPACCIDTOHTREEITEM: u32 = 4394u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_MAPHTREEITEMTOACCID: u32 = 4395u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SELECTITEM: u32 = 4363u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETAUTOSCROLLINFO: u32 = 4411u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETBKCOLOR: u32 = 4381u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETBORDER: u32 = 4387u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETEXTENDEDSTYLE: u32 = 4396u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETHOT: u32 = 4410u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETIMAGELIST: u32 = 4361u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETINDENT: u32 = 4359u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETINSERTMARK: u32 = 4378u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETINSERTMARKCOLOR: u32 = 4389u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETITEM: u32 = 4415u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETITEMA: u32 = 4365u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETITEMHEIGHT: u32 = 4379u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETITEMW: u32 = 4415u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETLINECOLOR: u32 = 4392u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETSCROLLTIME: u32 = 4385u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETTEXTCOLOR: u32 = 4382u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETTOOLTIPS: u32 = 4376u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SHOWINFOTIP: u32 = 4423u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SORTCHILDREN: u32 = 4371u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVM_SORTCHILDRENCB: u32 = 4373u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVNRET_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVNRET_SKIPNEW: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVNRET_SKIPOLD: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVSBF_XBORDER: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVSBF_YBORDER: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVSIL_NORMAL: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVSIL_STATE: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVSI_NOSINGLEEXPAND: u32 = 32768u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct TVSORTCB {
    pub hParent: HTREEITEM,
    pub lpfnCompare: PFNTVCOMPARE,
    pub lParam: super::super::Foundation::LPARAM,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for TVSORTCB {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for TVSORTCB {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_CHECKBOXES: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_DISABLEDRAGDROP: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EDITLABELS: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_AUTOHSCROLL: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_DIMMEDCHECKBOXES: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_DOUBLEBUFFER: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_DRAWIMAGEASYNC: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_EXCLUSIONCHECKBOXES: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_FADEINOUTEXPANDOS: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_MULTISELECT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_NOINDENTSTATE: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_NOSINGLECOLLAPSE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_PARTIALCHECKBOXES: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_EX_RICHTOOLTIP: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_FULLROWSELECT: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_HASBUTTONS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_HASLINES: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_INFOTIP: u32 = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_LINESATROOT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_NOHSCROLL: u32 = 32768u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_NONEVENHEIGHT: u32 = 16384u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_NOSCROLL: u32 = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_NOTOOLTIPS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_RTLREADING: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_SHOWSELALWAYS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_SINGLEEXPAND: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TVS_TRACKSELECT: u32 = 512u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const TV_FIRST: u32 = 4352u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct UDACCEL {
    pub nSec: u32,
    pub nInc: u32,
}
impl ::core::marker::Copy for UDACCEL {}
impl ::core::clone::Clone for UDACCEL {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETACCEL: u32 = 1132u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETBASE: u32 = 1134u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETBUDDY: u32 = 1130u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETPOS: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETPOS32: u32 = 1138u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETRANGE: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETRANGE32: u32 = 1136u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_GETUNICODEFORMAT: u32 = 8198u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETACCEL: u32 = 1131u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETBASE: u32 = 1133u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETBUDDY: u32 = 1129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETPOS: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETPOS32: u32 = 1137u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETRANGE: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETRANGE32: u32 = 1135u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDM_SETUNICODEFORMAT: u32 = 8197u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_ALIGNLEFT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_ALIGNRIGHT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_ARROWKEYS: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_AUTOBUDDY: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_HORZ: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_HOTTRACK: u32 = 256u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_NOTHOUSANDS: u32 = 128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_SETBUDDYINT: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UDS_WRAP: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UD_MAXVAL: u32 = 32767u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UPDOWN_CLASS: &str = "msctls_updown32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UPDOWN_CLASSA: &str = "msctls_updown32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const UPDOWN_CLASSW: &str = "msctls_updown32";
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct USAGE_PROPERTIES {
    pub level: u16,
    pub page: u16,
    pub usage: u16,
    pub logicalMinimum: i32,
    pub logicalMaximum: i32,
    pub unit: u16,
    pub exponent: u16,
    pub count: u8,
    pub physicalMinimum: i32,
    pub physicalMaximum: i32,
}
impl ::core::marker::Copy for USAGE_PROPERTIES {}
impl ::core::clone::Clone for USAGE_PROPERTIES {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type VALIGN = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VA_TOP: VALIGN = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VA_CENTER: VALIGN = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VA_BOTTOM: VALIGN = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_DETAILS: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_LARGEICONS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_LIST: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_NETCONNECT: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_NETDISCONNECT: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_NEWFOLDER: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_PARENTFOLDER: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_SMALLICONS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_SORTDATE: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_SORTNAME: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_SORTSIZE: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_SORTTYPE: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VIEW_VIEWMENU: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_CLOCK: &str = "CLOCK";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_EMPTYMARKUP: &str = "EMPTYMARKUP";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_LINK: &str = "LINK";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_MENUBAND: &str = "MENUBAND";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_MONTHCAL: &str = "MONTHCAL";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_PAGE: &str = "PAGE";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_STARTPANEL: &str = "STARTPANEL";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_STATIC: &str = "STATIC";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_TASKBAND: &str = "TASKBAND";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_TASKBAR: &str = "TASKBAR";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const VSCLASS_TRAYNOTIFY: &str = "TRAYNOTIFY";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_BUTTON: &str = "Button";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_BUTTONA: &str = "Button";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_BUTTONW: &str = "Button";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_COMBOBOX: &str = "ComboBox";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_COMBOBOXA: &str = "ComboBox";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_COMBOBOXEX: &str = "ComboBoxEx32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_COMBOBOXEXA: &str = "ComboBoxEx32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_COMBOBOXEXW: &str = "ComboBoxEx32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_COMBOBOXW: &str = "ComboBox";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_EDIT: &str = "Edit";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_EDITA: &str = "Edit";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_EDITW: &str = "Edit";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_HEADER: &str = "SysHeader32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_HEADERA: &str = "SysHeader32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_HEADERW: &str = "SysHeader32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_IPADDRESS: &str = "SysIPAddress32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_IPADDRESSA: &str = "SysIPAddress32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_IPADDRESSW: &str = "SysIPAddress32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LINK: &str = "SysLink";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LISTBOX: &str = "ListBox";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LISTBOXA: &str = "ListBox";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LISTBOXW: &str = "ListBox";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LISTVIEW: &str = "SysListView32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LISTVIEWA: &str = "SysListView32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_LISTVIEWW: &str = "SysListView32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_NATIVEFONTCTL: &str = "NativeFontCtl";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_NATIVEFONTCTLA: &str = "NativeFontCtl";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_NATIVEFONTCTLW: &str = "NativeFontCtl";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_PAGESCROLLER: &str = "SysPager";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_PAGESCROLLERA: &str = "SysPager";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_PAGESCROLLERW: &str = "SysPager";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_SCROLLBAR: &str = "ScrollBar";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_SCROLLBARA: &str = "ScrollBar";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_SCROLLBARW: &str = "ScrollBar";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_STATIC: &str = "Static";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_STATICA: &str = "Static";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_STATICW: &str = "Static";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_TABCONTROL: &str = "SysTabControl32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_TABCONTROLA: &str = "SysTabControl32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_TABCONTROLW: &str = "SysTabControl32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_TREEVIEW: &str = "SysTreeView32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_TREEVIEWA: &str = "SysTreeView32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WC_TREEVIEWW: &str = "SysTreeView32";
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type WINDOWTHEMEATTRIBUTETYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WTA_NONCLIENT: WINDOWTHEMEATTRIBUTETYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WIZ_BODYCX: u32 = 184u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WIZ_BODYX: u32 = 92u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WIZ_CXBMP: u32 = 80u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WIZ_CXDLG: u32 = 276u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WIZ_CYDLG: u32 = 140u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WM_CTLCOLOR: u32 = 25u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WM_MOUSEHOVER: u32 = 673u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WM_MOUSELEAVE: u32 = 675u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type WORD_BREAK_ACTION = u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_CLASSIFY: WORD_BREAK_ACTION = 3u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_ISDELIMITER: WORD_BREAK_ACTION = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_LEFT: WORD_BREAK_ACTION = 0u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_LEFTBREAK: WORD_BREAK_ACTION = 6u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_MOVEWORDLEFT: WORD_BREAK_ACTION = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_MOVEWORDRIGHT: WORD_BREAK_ACTION = 5u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_RIGHT: WORD_BREAK_ACTION = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WB_RIGHTBREAK: WORD_BREAK_ACTION = 7u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type WSB_PROP = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_CXHSCROLL: WSB_PROP = 2i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_CXHTHUMB: WSB_PROP = 16i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_CXVSCROLL: WSB_PROP = 8i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_CYHSCROLL: WSB_PROP = 4i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_CYVSCROLL: WSB_PROP = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_CYVTHUMB: WSB_PROP = 32i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_HBKGCOLOR: WSB_PROP = 128i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_HSTYLE: WSB_PROP = 512i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_PALETTE: WSB_PROP = 2048i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_VBKGCOLOR: WSB_PROP = 64i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_VSTYLE: WSB_PROP = 256i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_WINSTYLE: WSB_PROP = 1024i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WSB_PROP_MASK: i32 = 4095i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub struct WTA_OPTIONS {
    pub dwFlags: u32,
    pub dwMask: u32,
}
impl ::core::marker::Copy for WTA_OPTIONS {}
impl ::core::clone::Clone for WTA_OPTIONS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WTNCA_NODRAWCAPTION: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WTNCA_NODRAWICON: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WTNCA_NOMIRRORHELP: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const WTNCA_NOSYSMENU: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub type _LI_METRIC = i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIM_SMALL: _LI_METRIC = 0i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const LIM_LARGE: _LI_METRIC = 1i32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx1: u32 = 1040u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx10: u32 = 1049u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx11: u32 = 1050u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx12: u32 = 1051u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx13: u32 = 1052u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx14: u32 = 1053u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx15: u32 = 1054u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx16: u32 = 1055u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx2: u32 = 1041u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx3: u32 = 1042u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx4: u32 = 1043u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx5: u32 = 1044u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx6: u32 = 1045u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx7: u32 = 1046u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx8: u32 = 1047u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const chx9: u32 = 1048u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb1: u32 = 1136u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb10: u32 = 1145u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb11: u32 = 1146u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb12: u32 = 1147u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb13: u32 = 1148u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb14: u32 = 1149u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb15: u32 = 1150u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb16: u32 = 1151u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb2: u32 = 1137u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb3: u32 = 1138u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb4: u32 = 1139u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb5: u32 = 1140u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb6: u32 = 1141u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb7: u32 = 1142u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb8: u32 = 1143u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const cmb9: u32 = 1144u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ctl1: u32 = 1184u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ctlFirst: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ctlLast: u32 = 1279u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt1: u32 = 1152u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt10: u32 = 1161u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt11: u32 = 1162u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt12: u32 = 1163u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt13: u32 = 1164u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt14: u32 = 1165u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt15: u32 = 1166u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt16: u32 = 1167u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt2: u32 = 1153u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt3: u32 = 1154u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt4: u32 = 1155u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt5: u32 = 1156u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt6: u32 = 1157u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt7: u32 = 1158u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt8: u32 = 1159u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const edt9: u32 = 1160u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const frm1: u32 = 1076u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const frm2: u32 = 1077u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const frm3: u32 = 1078u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const frm4: u32 = 1079u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const grp1: u32 = 1072u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const grp2: u32 = 1073u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const grp3: u32 = 1074u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const grp4: u32 = 1075u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ico1: u32 = 1084u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ico2: u32 = 1085u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ico3: u32 = 1086u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const ico4: u32 = 1087u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst1: u32 = 1120u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst10: u32 = 1129u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst11: u32 = 1130u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst12: u32 = 1131u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst13: u32 = 1132u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst14: u32 = 1133u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst15: u32 = 1134u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst16: u32 = 1135u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst2: u32 = 1121u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst3: u32 = 1122u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst4: u32 = 1123u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst5: u32 = 1124u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst6: u32 = 1125u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst7: u32 = 1126u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst8: u32 = 1127u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const lst9: u32 = 1128u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh1: u32 = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh10: u32 = 1033u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh11: u32 = 1034u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh12: u32 = 1035u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh13: u32 = 1036u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh14: u32 = 1037u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh15: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh16: u32 = 1039u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh2: u32 = 1025u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh3: u32 = 1026u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh4: u32 = 1027u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh5: u32 = 1028u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh6: u32 = 1029u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh7: u32 = 1030u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh8: u32 = 1031u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const psh9: u32 = 1032u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const pshHelp: u32 = 1038u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad1: u32 = 1056u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad10: u32 = 1065u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad11: u32 = 1066u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad12: u32 = 1067u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad13: u32 = 1068u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad14: u32 = 1069u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad15: u32 = 1070u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad16: u32 = 1071u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad2: u32 = 1057u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad3: u32 = 1058u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad4: u32 = 1059u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad5: u32 = 1060u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad6: u32 = 1061u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad7: u32 = 1062u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad8: u32 = 1063u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rad9: u32 = 1064u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rct1: u32 = 1080u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rct2: u32 = 1081u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rct3: u32 = 1082u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const rct4: u32 = 1083u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr1: u32 = 1168u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr2: u32 = 1169u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr3: u32 = 1170u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr4: u32 = 1171u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr5: u32 = 1172u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr6: u32 = 1173u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr7: u32 = 1174u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const scr8: u32 = 1175u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc1: u32 = 1088u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc10: u32 = 1097u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc11: u32 = 1098u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc12: u32 = 1099u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc13: u32 = 1100u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc14: u32 = 1101u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc15: u32 = 1102u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc16: u32 = 1103u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc17: u32 = 1104u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc18: u32 = 1105u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc19: u32 = 1106u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc2: u32 = 1089u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc20: u32 = 1107u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc21: u32 = 1108u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc22: u32 = 1109u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc23: u32 = 1110u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc24: u32 = 1111u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc25: u32 = 1112u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc26: u32 = 1113u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc27: u32 = 1114u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc28: u32 = 1115u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc29: u32 = 1116u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc3: u32 = 1090u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc30: u32 = 1117u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc31: u32 = 1118u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc32: u32 = 1119u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc4: u32 = 1091u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc5: u32 = 1092u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc6: u32 = 1093u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc7: u32 = 1094u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc8: u32 = 1095u32;
#[doc = "*Required features: `\"Win32_UI_Controls\"`*"]
pub const stc9: u32 = 1096u32;
