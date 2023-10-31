#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn AccNotifyTouchInteraction(hwndapp: super::super::Foundation::HWND, hwndtarget: super::super::Foundation::HWND, pttarget: super::super::Foundation::POINT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn AccSetRunningUtilityState(hwndapp: super::super::Foundation::HWND, dwutilitystatemask: u32, dwutilitystate: ACC_UTILITY_STATE_FLAGS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn AccessibleChildren(pacccontainer: IAccessible, ichildstart: i32, cchildren: i32, rgvarchildren: *mut super::super::System::Com::VARIANT, pcobtained: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn AccessibleObjectFromEvent(hwnd: super::super::Foundation::HWND, dwid: u32, dwchildid: u32, ppacc: *mut IAccessible, pvarchild: *mut super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn AccessibleObjectFromPoint(ptscreen: super::super::Foundation::POINT, ppacc: *mut IAccessible, pvarchild: *mut super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn AccessibleObjectFromWindow(hwnd: super::super::Foundation::HWND, dwid: u32, riid: *const ::windows_sys::core::GUID, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateStdAccessibleObject(hwnd: super::super::Foundation::HWND, idobject: i32, riid: *const ::windows_sys::core::GUID, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateStdAccessibleProxyA(hwnd: super::super::Foundation::HWND, pclassname: ::windows_sys::core::PCSTR, idobject: i32, riid: *const ::windows_sys::core::GUID, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CreateStdAccessibleProxyW(hwnd: super::super::Foundation::HWND, pclassname: ::windows_sys::core::PCWSTR, idobject: i32, riid: *const ::windows_sys::core::GUID, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn DockPattern_SetDockPosition(hobj: HUIAPATTERNOBJECT, dockposition: DockPosition) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn ExpandCollapsePattern_Collapse(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn ExpandCollapsePattern_Expand(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn GetOleaccVersionInfo(pver: *mut u32, pbuild: *mut u32);
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn GetRoleTextA(lrole: u32, lpszrole: ::windows_sys::core::PSTR, cchrolemax: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn GetRoleTextW(lrole: u32, lpszrole: ::windows_sys::core::PWSTR, cchrolemax: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn GetStateTextA(lstatebit: u32, lpszstate: ::windows_sys::core::PSTR, cchstate: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn GetStateTextW(lstatebit: u32, lpszstate: ::windows_sys::core::PWSTR, cchstate: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn GridPattern_GetItem(hobj: HUIAPATTERNOBJECT, row: i32, column: i32, presult: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn InvokePattern_Invoke(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsWinEventHookInstalled(event: u32) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn ItemContainerPattern_FindItemByProperty(hobj: HUIAPATTERNOBJECT, hnodestartafter: HUIANODE, propertyid: i32, value: super::super::System::Com::VARIANT, pfound: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn LegacyIAccessiblePattern_DoDefaultAction(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn LegacyIAccessiblePattern_GetIAccessible(hobj: HUIAPATTERNOBJECT, paccessible: *mut IAccessible) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn LegacyIAccessiblePattern_Select(hobj: HUIAPATTERNOBJECT, flagsselect: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn LegacyIAccessiblePattern_SetValue(hobj: HUIAPATTERNOBJECT, szvalue: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn LresultFromObject(riid: *const ::windows_sys::core::GUID, wparam: super::super::Foundation::WPARAM, punk: ::windows_sys::core::IUnknown) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn MultipleViewPattern_GetViewName(hobj: HUIAPATTERNOBJECT, viewid: i32, ppstr: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn MultipleViewPattern_SetCurrentView(hobj: HUIAPATTERNOBJECT, viewid: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn NotifyWinEvent(event: u32, hwnd: super::super::Foundation::HWND, idobject: i32, idchild: i32);
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ObjectFromLresult(lresult: super::super::Foundation::LRESULT, riid: *const ::windows_sys::core::GUID, wparam: super::super::Foundation::WPARAM, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn RangeValuePattern_SetValue(hobj: HUIAPATTERNOBJECT, val: f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn RegisterPointerInputTarget(hwnd: super::super::Foundation::HWND, pointertype: super::WindowsAndMessaging::POINTER_INPUT_TYPE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn RegisterPointerInputTargetEx(hwnd: super::super::Foundation::HWND, pointertype: super::WindowsAndMessaging::POINTER_INPUT_TYPE, fobserve: super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn ScrollItemPattern_ScrollIntoView(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn ScrollPattern_Scroll(hobj: HUIAPATTERNOBJECT, horizontalamount: ScrollAmount, verticalamount: ScrollAmount) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn ScrollPattern_SetScrollPercent(hobj: HUIAPATTERNOBJECT, horizontalpercent: f64, verticalpercent: f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn SelectionItemPattern_AddToSelection(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn SelectionItemPattern_RemoveFromSelection(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn SelectionItemPattern_Select(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SetWinEventHook(eventmin: u32, eventmax: u32, hmodwineventproc: super::super::Foundation::HINSTANCE, pfnwineventproc: WINEVENTPROC, idprocess: u32, idthread: u32, dwflags: u32) -> HWINEVENTHOOK;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn SynchronizedInputPattern_Cancel(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn SynchronizedInputPattern_StartListening(hobj: HUIAPATTERNOBJECT, inputtype: SynchronizedInputType) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn TextPattern_GetSelection(hobj: HUIAPATTERNOBJECT, pretval: *mut *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn TextPattern_GetVisibleRanges(hobj: HUIAPATTERNOBJECT, pretval: *mut *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextPattern_RangeFromChild(hobj: HUIAPATTERNOBJECT, hnodechild: HUIANODE, pretval: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextPattern_RangeFromPoint(hobj: HUIAPATTERNOBJECT, point: UiaPoint, pretval: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextPattern_get_DocumentRange(hobj: HUIAPATTERNOBJECT, pretval: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextPattern_get_SupportedTextSelection(hobj: HUIAPATTERNOBJECT, pretval: *mut SupportedTextSelection) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_AddToSelection(hobj: HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_Clone(hobj: HUIATEXTRANGE, pretval: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TextRange_Compare(hobj: HUIATEXTRANGE, range: HUIATEXTRANGE, pretval: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_CompareEndpoints(hobj: HUIATEXTRANGE, endpoint: TextPatternRangeEndpoint, targetrange: HUIATEXTRANGE, targetendpoint: TextPatternRangeEndpoint, pretval: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_ExpandToEnclosingUnit(hobj: HUIATEXTRANGE, unit: TextUnit) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn TextRange_FindAttribute(hobj: HUIATEXTRANGE, attributeid: i32, val: super::super::System::Com::VARIANT, backward: super::super::Foundation::BOOL, pretval: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TextRange_FindText(hobj: HUIATEXTRANGE, text: super::super::Foundation::BSTR, backward: super::super::Foundation::BOOL, ignorecase: super::super::Foundation::BOOL, pretval: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn TextRange_GetAttributeValue(hobj: HUIATEXTRANGE, attributeid: i32, pretval: *mut super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn TextRange_GetBoundingRectangles(hobj: HUIATEXTRANGE, pretval: *mut *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn TextRange_GetChildren(hobj: HUIATEXTRANGE, pretval: *mut *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_GetEnclosingElement(hobj: HUIATEXTRANGE, pretval: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TextRange_GetText(hobj: HUIATEXTRANGE, maxlength: i32, pretval: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_Move(hobj: HUIATEXTRANGE, unit: TextUnit, count: i32, pretval: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_MoveEndpointByRange(hobj: HUIATEXTRANGE, endpoint: TextPatternRangeEndpoint, targetrange: HUIATEXTRANGE, targetendpoint: TextPatternRangeEndpoint) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_MoveEndpointByUnit(hobj: HUIATEXTRANGE, endpoint: TextPatternRangeEndpoint, unit: TextUnit, count: i32, pretval: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_RemoveFromSelection(hobj: HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn TextRange_ScrollIntoView(hobj: HUIATEXTRANGE, aligntotop: super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TextRange_Select(hobj: HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TogglePattern_Toggle(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TransformPattern_Move(hobj: HUIAPATTERNOBJECT, x: f64, y: f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TransformPattern_Resize(hobj: HUIAPATTERNOBJECT, width: f64, height: f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn TransformPattern_Rotate(hobj: HUIAPATTERNOBJECT, degrees: f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaAddEvent(hnode: HUIANODE, eventid: i32, pcallback: *mut UiaEventCallback, scope: TreeScope, pproperties: *mut i32, cproperties: i32, prequest: *mut UiaCacheRequest, phevent: *mut HUIAEVENT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaClientsAreListening() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaDisconnectAllProviders() -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaDisconnectProvider(pprovider: IRawElementProviderSimple) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaEventAddWindow(hevent: HUIAEVENT, hwnd: super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaEventRemoveWindow(hevent: HUIAEVENT, hwnd: super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaFind(hnode: HUIANODE, pparams: *mut UiaFindParams, prequest: *mut UiaCacheRequest, pprequesteddata: *mut *mut super::super::System::Com::SAFEARRAY, ppoffsets: *mut *mut super::super::System::Com::SAFEARRAY, pptreestructures: *mut *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaGetErrorDescription(pdescription: *mut super::super::Foundation::BSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaGetPatternProvider(hnode: HUIANODE, patternid: i32, phobj: *mut HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaGetPropertyValue(hnode: HUIANODE, propertyid: i32, pvalue: *mut super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaGetReservedMixedAttributeValue(punkmixedattributevalue: *mut ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaGetReservedNotSupportedValue(punknotsupportedvalue: *mut ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaGetRootNode(phnode: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn UiaGetRuntimeId(hnode: HUIANODE, pruntimeid: *mut *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaGetUpdatedCache(hnode: HUIANODE, prequest: *mut UiaCacheRequest, normalizestate: NormalizeState, pnormalizecondition: *mut UiaCondition, pprequesteddata: *mut *mut super::super::System::Com::SAFEARRAY, pptreestructure: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaHPatternObjectFromVariant(pvar: *mut super::super::System::Com::VARIANT, phobj: *mut HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaHTextRangeFromVariant(pvar: *mut super::super::System::Com::VARIANT, phtextrange: *mut HUIATEXTRANGE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaHUiaNodeFromVariant(pvar: *mut super::super::System::Com::VARIANT, phnode: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaHasServerSideProvider(hwnd: super::super::Foundation::HWND) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaHostProviderFromHwnd(hwnd: super::super::Foundation::HWND, ppprovider: *mut IRawElementProviderSimple) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaIAccessibleFromProvider(pprovider: IRawElementProviderSimple, dwflags: u32, ppaccessible: *mut IAccessible, pvarchild: *mut super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaLookupId(r#type: AutomationIdentifierType, pguid: *const ::windows_sys::core::GUID) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaNavigate(hnode: HUIANODE, direction: NavigateDirection, pcondition: *mut UiaCondition, prequest: *mut UiaCacheRequest, pprequesteddata: *mut *mut super::super::System::Com::SAFEARRAY, pptreestructure: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaNodeFromFocus(prequest: *mut UiaCacheRequest, pprequesteddata: *mut *mut super::super::System::Com::SAFEARRAY, pptreestructure: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaNodeFromHandle(hwnd: super::super::Foundation::HWND, phnode: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaNodeFromPoint(x: f64, y: f64, prequest: *mut UiaCacheRequest, pprequesteddata: *mut *mut super::super::System::Com::SAFEARRAY, pptreestructure: *mut super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaNodeFromProvider(pprovider: IRawElementProviderSimple, phnode: *mut HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaNodeRelease(hnode: HUIANODE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaPatternRelease(hobj: HUIAPATTERNOBJECT) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaProviderForNonClient(hwnd: super::super::Foundation::HWND, idobject: i32, idchild: i32, ppprovider: *mut IRawElementProviderSimple) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn UiaProviderFromIAccessible(paccessible: IAccessible, idchild: i32, dwflags: u32, ppprovider: *mut IRawElementProviderSimple) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaRaiseActiveTextPositionChangedEvent(provider: IRawElementProviderSimple, textrange: ITextRangeProvider) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaRaiseAsyncContentLoadedEvent(pprovider: IRawElementProviderSimple, asynccontentloadedstate: AsyncContentLoadedState, percentcomplete: f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaRaiseAutomationEvent(pprovider: IRawElementProviderSimple, id: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaRaiseAutomationPropertyChangedEvent(pprovider: IRawElementProviderSimple, id: i32, oldvalue: super::super::System::Com::VARIANT, newvalue: super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn UiaRaiseChangesEvent(pprovider: IRawElementProviderSimple, eventidcount: i32, puiachanges: *mut UiaChangeInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaRaiseNotificationEvent(provider: IRawElementProviderSimple, notificationkind: NotificationKind, notificationprocessing: NotificationProcessing, displaystring: super::super::Foundation::BSTR, activityid: super::super::Foundation::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaRaiseStructureChangedEvent(pprovider: IRawElementProviderSimple, structurechangetype: StructureChangeType, pruntimeid: *mut i32, cruntimeidlen: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn UiaRaiseTextEditTextChangedEvent(pprovider: IRawElementProviderSimple, texteditchangetype: TextEditChangeType, pchangeddata: *mut super::super::System::Com::SAFEARRAY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn UiaRegisterProviderCallback(pcallback: *mut UiaProviderCallback);
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaRemoveEvent(hevent: HUIAEVENT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaReturnRawElementProvider(hwnd: super::super::Foundation::HWND, wparam: super::super::Foundation::WPARAM, lparam: super::super::Foundation::LPARAM, el: IRawElementProviderSimple) -> super::super::Foundation::LRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn UiaSetFocus(hnode: HUIANODE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UiaTextRangeRelease(hobj: HUIATEXTRANGE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UnhookWinEvent(hwineventhook: HWINEVENTHOOK) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn UnregisterPointerInputTarget(hwnd: super::super::Foundation::HWND, pointertype: super::WindowsAndMessaging::POINTER_INPUT_TYPE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_UI_WindowsAndMessaging\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_UI_WindowsAndMessaging"))]
    pub fn UnregisterPointerInputTargetEx(hwnd: super::super::Foundation::HWND, pointertype: super::WindowsAndMessaging::POINTER_INPUT_TYPE) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn ValuePattern_SetValue(hobj: HUIAPATTERNOBJECT, pval: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn VirtualizedItemPattern_Realize(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
    pub fn WindowFromAccessibleObject(param0: IAccessible, phwnd: *mut super::super::Foundation::HWND) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn WindowPattern_Close(hobj: HUIAPATTERNOBJECT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
    pub fn WindowPattern_SetWindowVisualState(hobj: HUIAPATTERNOBJECT, state: WindowVisualState) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WindowPattern_WaitForInputIdle(hobj: HUIAPATTERNOBJECT, milliseconds: i32, presult: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct ACCESSTIMEOUT {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub iTimeOutMSec: u32,
}
impl ::core::marker::Copy for ACCESSTIMEOUT {}
impl ::core::clone::Clone for ACCESSTIMEOUT {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ACC_UTILITY_STATE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANRUS_ON_SCREEN_KEYBOARD_ACTIVE: ACC_UTILITY_STATE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANRUS_TOUCH_MODIFICATION_ACTIVE: ACC_UTILITY_STATE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANRUS_PRIORITY_AUDIO_ACTIVE: ACC_UTILITY_STATE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANRUS_PRIORITY_AUDIO_ACTIVE_NODUCK: ACC_UTILITY_STATE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANRUS_PRIORITY_AUDIO_DYNAMIC_DUCK: u32 = 16u32;
pub const AcceleratorKey_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1363699167, data2: 9559, data3: 19641, data4: [174, 237, 108, 237, 8, 76, 229, 44] };
pub const AccessKey_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 109214482, data2: 43001, data3: 18965, data4: [145, 124, 255, 165, 173, 62, 176, 167] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ActiveEnd = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ActiveEnd_None: ActiveEnd = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ActiveEnd_Start: ActiveEnd = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ActiveEnd_End: ActiveEnd = 2i32;
pub const ActiveTextPositionChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2780864156, data2: 51069, data3: 20261, data4: [180, 145, 229, 187, 112, 23, 203, 212] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type AnimationStyle = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_None: AnimationStyle = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_LasVegasLights: AnimationStyle = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_BlinkingBackground: AnimationStyle = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_SparkleText: AnimationStyle = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_MarchingBlackAnts: AnimationStyle = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_MarchingRedAnts: AnimationStyle = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_Shimmer: AnimationStyle = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnimationStyle_Other: AnimationStyle = -1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type AnnoScope = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANNO_THIS: AnnoScope = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ANNO_CONTAINER: AnnoScope = 1i32;
pub const AnnotationObjects_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 822677704, data2: 31854, data3: 20256, data4: [190, 205, 74, 175, 109, 25, 17, 86] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_AdvancedProofingIssue: i32 = 60020i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Author: i32 = 60019i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_CircularReferenceError: i32 = 60022i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Comment: i32 = 60003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_ConflictingChange: i32 = 60018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_DataValidationError: i32 = 60021i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_DeletionChange: i32 = 60012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_EditingLockedChange: i32 = 60016i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Endnote: i32 = 60009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_ExternalChange: i32 = 60017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Footer: i32 = 60007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Footnote: i32 = 60010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_FormatChange: i32 = 60014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_FormulaError: i32 = 60004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_GrammarError: i32 = 60002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Header: i32 = 60006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Highlighted: i32 = 60008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_InsertionChange: i32 = 60011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Mathematics: i32 = 60023i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_MoveChange: i32 = 60013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Sensitive: i32 = 60024i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_SpellingError: i32 = 60001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_TrackChanges: i32 = 60005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_Unknown: i32 = 60000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AnnotationType_UnsyncedChange: i32 = 60015i32;
pub const AnnotationTypes_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1689722742, data2: 21444, data3: 18070, data4: [162, 25, 32, 233, 64, 201, 161, 118] };
pub const Annotation_AdvancedProofingIssue_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3670521644, data2: 49394, data3: 19332, data4: [185, 13, 95, 175, 192, 240, 239, 28] };
pub const Annotation_AnnotationTypeId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 548292687, data2: 27119, data3: 19528, data4: [143, 91, 196, 147, 139, 32, 106, 199] };
pub const Annotation_AnnotationTypeName_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2608957586, data2: 23241, data3: 19193, data4: [170, 150, 245, 138, 119, 176, 88, 227] };
pub const Annotation_Author_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4049720231, data2: 63515, data3: 16680, data4: [177, 127, 113, 246, 144, 145, 69, 32] };
pub const Annotation_Author_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2052228194, data2: 40028, data3: 18947, data4: [169, 116, 139, 48, 122, 153, 55, 242] };
pub const Annotation_CircularReferenceError_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 633183476, data2: 5957, data3: 18009, data4: [186, 103, 114, 127, 3, 24, 198, 22] };
pub const Annotation_Comment_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4247771696, data2: 9907, data3: 19462, data4: [139, 199, 152, 241, 83, 46, 70, 253] };
pub const Annotation_ConflictingChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2561640450, data2: 20860, data3: 17823, data4: [175, 19, 1, 109, 63, 171, 135, 126] };
pub const Annotation_Custom_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2663917392, data2: 14641, data3: 18770, data4: [133, 188, 29, 191, 247, 138, 67, 227] };
pub const Annotation_DataValidationError_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3362037672, data2: 38773, data3: 17278, data4: [173, 70, 231, 9, 217, 60, 35, 67] };
pub const Annotation_DateTime_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2578827869, data2: 6863, data3: 16715, data4: [164, 208, 107, 53, 11, 4, 117, 120] };
pub const Annotation_DeletionChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3191692037, data2: 38173, data3: 17127, data4: [144, 29, 173, 200, 194, 207, 52, 208] };
pub const Annotation_EditingLockedChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3273604636, data2: 29731, data3: 19884, data4: [131, 72, 65, 240, 153, 255, 111, 100] };
pub const Annotation_Endnote_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1969582684, data2: 11673, data3: 18489, data4: [150, 13, 51, 211, 184, 102, 171, 165] };
pub const Annotation_ExternalChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1973443377, data2: 24337, data3: 17149, data4: [136, 125, 223, 160, 16, 219, 35, 146] };
pub const Annotation_Footer_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3437932614, data2: 6195, data3: 18346, data4: [128, 128, 112, 30, 208, 176, 200, 50] };
pub const Annotation_Footnote_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1038159393, data2: 16677, data3: 17115, data4: [134, 32, 190, 128, 131, 8, 6, 36] };
pub const Annotation_FormatChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3945034565, data2: 54513, data3: 16846, data4: [142, 82, 247, 155, 105, 99, 94, 72] };
pub const Annotation_FormulaError_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2506168706, data2: 3243, data3: 18133, data4: [162, 240, 227, 13, 25, 5, 248, 191] };
pub const Annotation_GrammarError_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1970930829, data2: 17688, data3: 16838, data4: [133, 76, 220, 0, 155, 124, 251, 83] };
pub const Annotation_Header_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2256224411, data2: 45590, data3: 17522, data4: [162, 25, 82, 94, 49, 6, 129, 248] };
pub const Annotation_Highlighted_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1971095630, data2: 32899, data3: 16513, data4: [139, 156, 232, 127, 80, 114, 240, 228] };
pub const Annotation_InsertionChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 230601638, data2: 57109, data3: 16740, data4: [163, 192, 226, 26, 140, 233, 49, 196] };
pub const Annotation_Mathematics_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3937100619, data2: 9936, data3: 16577, data4: [128, 115, 87, 202, 28, 99, 60, 155] };
pub const Annotation_MoveChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2644871147, data2: 9189, data3: 17552, data4: [179, 133, 26, 34, 221, 200, 177, 135] };
pub const Annotation_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4140247767, data2: 13676, data3: 18512, data4: [146, 145, 49, 111, 96, 138, 140, 132] };
pub const Annotation_Sensitive_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 938786895, data2: 3858, data3: 17508, data4: [146, 156, 130, 143, 209, 82, 146, 227] };
pub const Annotation_SpellingError_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2927974014, data2: 40654, data3: 16959, data4: [129, 183, 150, 196, 61, 83, 229, 14] };
pub const Annotation_Target_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3072012333, data2: 8452, data3: 17581, data4: [156, 92, 9, 43, 73, 7, 215, 15] };
pub const Annotation_TrackChanges_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 568780936, data2: 56340, data3: 16406, data4: [172, 39, 25, 5, 83, 200, 196, 112] };
pub const Annotation_UnsyncedChange_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 407966058, data2: 3655, data3: 19248, data4: [140, 181, 215, 218, 228, 251, 205, 27] };
pub const AppBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1628737677, data2: 52226, data3: 19767, data4: [135, 91, 181, 48, 199, 19, 149, 84] };
pub const AriaProperties_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1108567948, data2: 57381, data3: 18722, data4: [190, 181, 228, 59, 160, 142, 98, 33] };
pub const AriaRole_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3709893525, data2: 48714, data3: 19981, data4: [183, 39, 99, 172, 233, 75, 105, 22] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type AsyncContentLoadedState = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AsyncContentLoadedState_Beginning: AsyncContentLoadedState = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AsyncContentLoadedState_Progress: AsyncContentLoadedState = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AsyncContentLoadedState_Completed: AsyncContentLoadedState = 2i32;
pub const AsyncContentLoaded_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1608442140, data2: 54010, data3: 20409, data4: [144, 78, 92, 190, 232, 148, 213, 239] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type AutomationElementMode = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationElementMode_None: AutomationElementMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationElementMode_Full: AutomationElementMode = 1i32;
pub const AutomationFocusChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3062505239, data2: 62989, data3: 16807, data4: [163, 204, 176, 82, 146, 21, 95, 224] };
pub const AutomationId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3358328064, data2: 46606, data3: 17168, data4: [162, 103, 48, 60, 83, 31, 142, 229] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type AutomationIdentifierType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_Property: AutomationIdentifierType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_Pattern: AutomationIdentifierType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_Event: AutomationIdentifierType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_ControlType: AutomationIdentifierType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_TextAttribute: AutomationIdentifierType = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_LandmarkType: AutomationIdentifierType = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_Annotation: AutomationIdentifierType = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_Changes: AutomationIdentifierType = 7i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const AutomationIdentifierType_Style: AutomationIdentifierType = 8i32;
pub const AutomationPropertyChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 623377313, data2: 36218, data3: 17968, data4: [164, 204, 230, 99, 21, 148, 47, 82] };
pub const BoundingRectangle_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2076174514, data2: 15356, data3: 18653, data4: [183, 41, 199, 148, 184, 70, 233, 161] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type BulletStyle = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_None: BulletStyle = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_HollowRoundBullet: BulletStyle = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_FilledRoundBullet: BulletStyle = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_HollowSquareBullet: BulletStyle = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_FilledSquareBullet: BulletStyle = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_DashBullet: BulletStyle = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const BulletStyle_Other: BulletStyle = -1i32;
pub const Button_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1517871977, data2: 50849, data3: 20275, data4: [169, 215, 121, 242, 13, 12, 120, 142] };
pub const CAccPropServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3052942603, data2: 1352, data3: 18609, data4: [166, 238, 136, 189, 0, 180, 165, 231] };
pub const CLSID_AccPropServices: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3052942603, data2: 1352, data3: 18609, data4: [166, 238, 136, 189, 0, 180, 165, 231] };
pub const CUIAutomation: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4282964900, data2: 24815, data3: 16897, data4: [170, 135, 84, 16, 62, 239, 89, 78] };
pub const CUIAutomation8: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3794457395, data2: 45663, data3: 17932, data4: [131, 208, 5, 129, 16, 115, 149, 201] };
pub const CUIAutomationRegistrar: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1848244927, data2: 39287, data3: 17105, data4: [141, 14, 202, 126, 97, 173, 135, 230] };
pub const Calendar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2299784072, data2: 229, data3: 18108, data4: [142, 78, 20, 167, 134, 225, 101, 161] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type CapStyle = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_None: CapStyle = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_SmallCap: CapStyle = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_AllCap: CapStyle = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_AllPetiteCaps: CapStyle = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_PetiteCaps: CapStyle = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_Unicase: CapStyle = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_Titling: CapStyle = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CapStyle_Other: CapStyle = -1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type CaretBidiMode = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CaretBidiMode_LTR: CaretBidiMode = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CaretBidiMode_RTL: CaretBidiMode = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type CaretPosition = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CaretPosition_Unknown: CaretPosition = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CaretPosition_EndOfLine: CaretPosition = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CaretPosition_BeginningOfLine: CaretPosition = 2i32;
pub const CenterPoint_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 212864008, data2: 21516, data3: 20187, data4: [148, 69, 38, 53, 158, 166, 151, 133] };
pub const Changes_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2113038100, data2: 24911, data3: 19973, data4: [148, 136, 113, 108, 91, 161, 148, 54] };
pub const Changes_Summary_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 826107302, data2: 58895, data3: 19810, data4: [152, 97, 85, 175, 215, 40, 210, 7] };
pub const CheckBox_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4216387874, data2: 41947, data3: 18880, data4: [139, 195, 6, 218, 213, 87, 120, 226] };
pub const ClassName_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 360411669, data2: 35151, data3: 19301, data4: [132, 226, 170, 192, 218, 8, 177, 107] };
pub const ClickablePoint_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 26644539, data2: 45571, data3: 18456, data4: [169, 243, 240, 142, 103, 95, 35, 65] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type CoalesceEventsOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CoalesceEventsOptions_Disabled: CoalesceEventsOptions = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const CoalesceEventsOptions_Enabled: CoalesceEventsOptions = 1i32;
pub const ComboBox_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1422606956, data2: 12083, data3: 20479, data4: [170, 161, 174, 246, 13, 172, 93, 235] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ConditionType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConditionType_True: ConditionType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConditionType_False: ConditionType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConditionType_Property: ConditionType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConditionType_And: ConditionType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConditionType_Or: ConditionType = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConditionType_Not: ConditionType = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ConnectionRecoveryBehaviorOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConnectionRecoveryBehaviorOptions_Disabled: ConnectionRecoveryBehaviorOptions = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ConnectionRecoveryBehaviorOptions_Enabled: ConnectionRecoveryBehaviorOptions = 1i32;
pub const ControlType_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3396816874, data2: 10412, data3: 19394, data4: [148, 202, 172, 236, 109, 108, 16, 163] };
pub const ControllerFor_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1360153738, data2: 42450, data3: 20243, data4: [155, 230, 127, 168, 186, 157, 58, 144] };
pub const Culture_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3805761319, data2: 15737, data3: 19906, data4: [184, 139, 48, 68, 150, 58, 138, 251] };
pub const CustomNavigation_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2951385994, data2: 25118, data3: 16468, data4: [187, 44, 47, 70, 17, 77, 172, 63] };
pub const Custom_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4070482115, data2: 44471, data3: 17162, data4: [186, 144, 229, 44, 115, 19, 230, 237] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_CHILD: i32 = -5002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_CHILDCOUNT: i32 = -5001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_DEFAULTACTION: i32 = -5013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_DESCRIPTION: i32 = -5005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_DODEFAULTACTION: i32 = -5018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_FOCUS: i32 = -5011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_HELP: i32 = -5008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_HELPTOPIC: i32 = -5009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_HITTEST: i32 = -5017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_KEYBOARDSHORTCUT: i32 = -5010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_LOCATION: i32 = -5015i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_NAME: i32 = -5003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_NAVIGATE: i32 = -5016i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_PARENT: i32 = -5000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_ROLE: i32 = -5006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_SELECT: i32 = -5014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_SELECTION: i32 = -5012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_STATE: i32 = -5007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DISPID_ACC_VALUE: i32 = -5004i32;
pub const DataGrid_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2226619311, data2: 53507, data3: 19210, data4: [132, 21, 231, 57, 66, 65, 15, 75] };
pub const DataItem_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2685892674, data2: 55631, data3: 17061, data4: [129, 75, 96, 104, 173, 220, 141, 165] };
pub const DescribedBy_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2086167992, data2: 39314, data3: 16637, data4: [141, 176, 107, 241, 211, 23, 249, 152] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type DockPosition = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DockPosition_Top: DockPosition = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DockPosition_Left: DockPosition = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DockPosition_Bottom: DockPosition = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DockPosition_Right: DockPosition = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DockPosition_Fill: DockPosition = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const DockPosition_None: DockPosition = 5i32;
pub const Dock_DockPosition_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1835528238, data2: 49328, data3: 19216, data4: [181, 185, 24, 214, 236, 249, 135, 96] };
pub const Dock_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2629478470, data2: 33736, data3: 17037, data4: [130, 127, 126, 96, 99, 254, 6, 32] };
pub const Document_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1020705647, data2: 28424, data3: 17762, data4: [178, 41, 228, 226, 252, 122, 158, 180] };
pub const Drag_DragCancel_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3287148282, data2: 13393, data3: 19983, data4: [158, 113, 223, 156, 40, 10, 70, 87] };
pub const Drag_DragComplete_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 954818952, data2: 61215, data3: 17982, data4: [145, 202, 58, 119, 146, 194, 156, 175] };
pub const Drag_DragStart_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2285520907, data2: 15017, data3: 17053, data4: [149, 228, 217, 200, 208, 17, 240, 221] };
pub const Drag_DropEffect_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1685006201, data2: 18643, data3: 19235, data4: [137, 2, 75, 241, 0, 0, 93, 243] };
pub const Drag_DropEffects_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4124447062, data2: 31974, data3: 18878, data4: [168, 54, 146, 105, 220, 236, 146, 15] };
pub const Drag_GrabbedItems_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2009159212, data2: 31622, data3: 19233, data4: [158, 215, 60, 239, 218, 111, 76, 67] };
pub const Drag_IsGrabbed_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1173489395, data2: 30156, data3: 19658, data4: [169, 185, 252, 223, 185, 130, 216, 162] };
pub const Drag_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3233735199, data2: 52403, data3: 20461, data4: [153, 91, 17, 79, 110, 61, 39, 40] };
pub const DropTarget_DragEnter_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2866360731, data2: 812, data3: 19080, data4: [150, 29, 28, 245, 121, 88, 30, 52] };
pub const DropTarget_DragLeave_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 260238101, data2: 9378, data3: 18824, data4: [146, 23, 222, 22, 42, 238, 39, 43] };
pub const DropTarget_DropTargetEffect_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2344049013, data2: 41162, data3: 18817, data4: [184, 24, 135, 252, 102, 233, 80, 157] };
pub const DropTarget_DropTargetEffects_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3156071661, data2: 52105, data3: 17905, data4: [165, 146, 224, 59, 8, 174, 121, 15] };
pub const DropTarget_Dropped_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1647110872, data2: 7899, data3: 19005, data4: [171, 188, 190, 34, 17, 255, 104, 181] };
pub const DropTarget_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 197913686, data2: 48436, data3: 19323, data4: [159, 213, 38, 89, 144, 94, 163, 220] };
pub const Edit_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1694803400, data2: 11398, data3: 20359, data4: [174, 123, 26, 189, 220, 129, 12, 249] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type EventArgsType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_Simple: EventArgsType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_PropertyChanged: EventArgsType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_StructureChanged: EventArgsType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_AsyncContentLoaded: EventArgsType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_WindowClosed: EventArgsType = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_TextEditTextChanged: EventArgsType = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_Changes: EventArgsType = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_Notification: EventArgsType = 7i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_ActiveTextPositionChanged: EventArgsType = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const EventArgsType_StructuredMarkup: EventArgsType = 9i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ExpandCollapseState = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ExpandCollapseState_Collapsed: ExpandCollapseState = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ExpandCollapseState_Expanded: ExpandCollapseState = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ExpandCollapseState_PartiallyExpanded: ExpandCollapseState = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ExpandCollapseState_LeafNode: ExpandCollapseState = 3i32;
pub const ExpandCollapse_ExpandCollapseState_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 660229192, data2: 34215, data3: 20329, data4: [171, 160, 175, 21, 118, 16, 0, 43] };
pub const ExpandCollapse_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2919624610, data2: 63953, data3: 17034, data4: [131, 76, 83, 165, 197, 47, 155, 139] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct ExtendedProperty {
    pub PropertyName: super::super::Foundation::BSTR,
    pub PropertyValue: super::super::Foundation::BSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for ExtendedProperty {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for ExtendedProperty {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct FILTERKEYS {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub iWaitMSec: u32,
    pub iDelayMSec: u32,
    pub iRepeatMSec: u32,
    pub iBounceMSec: u32,
}
impl ::core::marker::Copy for FILTERKEYS {}
impl ::core::clone::Clone for FILTERKEYS {
    fn clone(&self) -> Self {
        *self
    }
}
pub const FillColor_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1846461648, data2: 58024, data3: 19030, data4: [157, 231, 149, 51, 137, 147, 59, 57] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type FillType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FillType_None: FillType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FillType_Color: FillType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FillType_Gradient: FillType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FillType_Picture: FillType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FillType_Pattern: FillType = 4i32;
pub const FillType_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3338433764, data2: 36025, data3: 17052, data4: [169, 225, 155, 196, 172, 55, 43, 98] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type FlowDirections = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FlowDirections_Default: FlowDirections = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FlowDirections_RightToLeft: FlowDirections = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FlowDirections_BottomToTop: FlowDirections = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const FlowDirections_Vertical: FlowDirections = 4i32;
pub const FlowsFrom_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 96896079, data2: 6622, data3: 18680, data4: [149, 250, 136, 13, 91, 15, 214, 21] };
pub const FlowsTo_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3841146144, data2: 21914, data3: 18427, data4: [168, 48, 249, 203, 79, 241, 167, 10] };
pub const FrameworkId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3690830080, data2: 32282, data3: 20312, data4: [182, 27, 112, 99, 18, 15, 119, 59] };
pub const FullDescription_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 222580991, data2: 27375, data3: 20275, data4: [149, 221, 123, 239, 167, 42, 67, 145] };
pub const GridItem_ColumnSpan_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1480500213, data2: 34512, data3: 19208, data4: [166, 236, 44, 84, 99, 255, 193, 9] };
pub const GridItem_Column_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3346317660, data2: 25280, data3: 17689, data4: [139, 220, 71, 190, 87, 60, 138, 213] };
pub const GridItem_Parent_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2643534418, data2: 47487, data3: 20172, data4: [133, 16, 234, 14, 51, 66, 124, 114] };
pub const GridItem_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4074096759, data2: 42082, data3: 18775, data4: [162, 165, 44, 150, 179, 3, 188, 99] };
pub const GridItem_RowSpan_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1166158108, data2: 18027, data3: 20115, data4: [142, 131, 61, 23, 21, 236, 12, 94] };
pub const GridItem_Row_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1646499626, data2: 51525, data3: 17763, data4: [147, 41, 253, 201, 116, 175, 37, 83] };
pub const Grid_ColumnCount_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4271305589, data2: 17578, data3: 17718, data4: [172, 122, 42, 117, 215, 26, 62, 252] };
pub const Grid_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 638201035, data2: 37800, data3: 20036, data4: [164, 193, 61, 243, 151, 242, 176, 43] };
pub const Grid_RowCount_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 714409407, data2: 49899, data3: 20406, data4: [179, 86, 130, 69, 174, 83, 112, 62] };
pub const Group_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2907744796, data2: 59592, data3: 18292, data4: [174, 27, 221, 134, 223, 11, 59, 220] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct HIGHCONTRASTA {
    pub cbSize: u32,
    pub dwFlags: HIGHCONTRASTW_FLAGS,
    pub lpszDefaultScheme: ::windows_sys::core::PSTR,
}
impl ::core::marker::Copy for HIGHCONTRASTA {}
impl ::core::clone::Clone for HIGHCONTRASTA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct HIGHCONTRASTW {
    pub cbSize: u32,
    pub dwFlags: HIGHCONTRASTW_FLAGS,
    pub lpszDefaultScheme: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for HIGHCONTRASTW {}
impl ::core::clone::Clone for HIGHCONTRASTW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type HIGHCONTRASTW_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_HIGHCONTRASTON: HIGHCONTRASTW_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_AVAILABLE: HIGHCONTRASTW_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_HOTKEYACTIVE: HIGHCONTRASTW_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_CONFIRMHOTKEY: HIGHCONTRASTW_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_HOTKEYSOUND: HIGHCONTRASTW_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_INDICATOR: HIGHCONTRASTW_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_HOTKEYAVAILABLE: HIGHCONTRASTW_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HCF_OPTION_NOTHEMECHANGE: HIGHCONTRASTW_FLAGS = 4096u32;
pub type HUIAEVENT = isize;
pub type HUIANODE = isize;
pub type HUIAPATTERNOBJECT = isize;
pub type HUIATEXTRANGE = isize;
pub type HWINEVENTHOOK = isize;
pub const HasKeyboardFocus_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3481992505, data2: 16198, data3: 18432, data4: [150, 86, 178, 191, 18, 82, 153, 5] };
pub const HeaderItem_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3871085259, data2: 31886, data3: 18895, data4: [177, 104, 74, 147, 163, 43, 235, 176] };
pub const Header_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1536216014, data2: 30971, data3: 17940, data4: [130, 182, 85, 77, 116, 113, 142, 103] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel1: i32 = 80051i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel2: i32 = 80052i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel3: i32 = 80053i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel4: i32 = 80054i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel5: i32 = 80055i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel6: i32 = 80056i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel7: i32 = 80057i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel8: i32 = 80058i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel9: i32 = 80059i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HeadingLevel_None: i32 = 80050i32;
pub const HeadingLevel_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 688407154, data2: 43695, data3: 18992, data4: [135, 150, 60, 18, 246, 43, 107, 187] };
pub const HelpText_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 139810437, data2: 2423, data3: 17863, data4: [167, 166, 171, 175, 86, 132, 18, 26] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type HorizontalTextAlignment = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HorizontalTextAlignment_Left: HorizontalTextAlignment = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HorizontalTextAlignment_Centered: HorizontalTextAlignment = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HorizontalTextAlignment_Right: HorizontalTextAlignment = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const HorizontalTextAlignment_Justified: HorizontalTextAlignment = 3i32;
pub const HostedFragmentRootsInvalidated_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3871191102, data2: 2337, data3: 20165, data4: [141, 207, 234, 232, 119, 176, 66, 107] };
pub const Hyperlink_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2320892460, data2: 45069, data3: 19733, data4: [143, 240, 91, 107, 38, 110, 94, 2] };
pub type IAccIdentity = *mut ::core::ffi::c_void;
pub type IAccPropServer = *mut ::core::ffi::c_void;
pub type IAccPropServices = *mut ::core::ffi::c_void;
pub type IAccessible = *mut ::core::ffi::c_void;
pub type IAccessibleEx = *mut ::core::ffi::c_void;
pub type IAccessibleHandler = *mut ::core::ffi::c_void;
pub type IAccessibleHostingElementProviders = *mut ::core::ffi::c_void;
pub type IAccessibleWindowlessSite = *mut ::core::ffi::c_void;
pub type IAnnotationProvider = *mut ::core::ffi::c_void;
pub type ICustomNavigationProvider = *mut ::core::ffi::c_void;
pub type IDockProvider = *mut ::core::ffi::c_void;
pub type IDragProvider = *mut ::core::ffi::c_void;
pub type IDropTargetProvider = *mut ::core::ffi::c_void;
pub type IExpandCollapseProvider = *mut ::core::ffi::c_void;
pub type IGridItemProvider = *mut ::core::ffi::c_void;
pub type IGridProvider = *mut ::core::ffi::c_void;
pub const IIS_ControlAccessible: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 952533670, data2: 38705, data3: 17394, data4: [159, 174, 233, 1, 230, 65, 177, 1] };
pub const IIS_IsOleaccProxy: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2418448378, data2: 32996, data3: 17760, data4: [128, 42, 161, 63, 34, 166, 71, 9] };
pub type IInvokeProvider = *mut ::core::ffi::c_void;
pub type IItemContainerProvider = *mut ::core::ffi::c_void;
pub type ILegacyIAccessibleProvider = *mut ::core::ffi::c_void;
pub type IMultipleViewProvider = *mut ::core::ffi::c_void;
pub type IObjectModelProvider = *mut ::core::ffi::c_void;
pub type IProxyProviderWinEventHandler = *mut ::core::ffi::c_void;
pub type IProxyProviderWinEventSink = *mut ::core::ffi::c_void;
pub type IRangeValueProvider = *mut ::core::ffi::c_void;
pub type IRawElementProviderAdviseEvents = *mut ::core::ffi::c_void;
pub type IRawElementProviderFragment = *mut ::core::ffi::c_void;
pub type IRawElementProviderFragmentRoot = *mut ::core::ffi::c_void;
pub type IRawElementProviderHostingAccessibles = *mut ::core::ffi::c_void;
pub type IRawElementProviderHwndOverride = *mut ::core::ffi::c_void;
pub type IRawElementProviderSimple = *mut ::core::ffi::c_void;
pub type IRawElementProviderSimple2 = *mut ::core::ffi::c_void;
pub type IRawElementProviderSimple3 = *mut ::core::ffi::c_void;
pub type IRawElementProviderWindowlessSite = *mut ::core::ffi::c_void;
pub type IRichEditUiaInformation = *mut ::core::ffi::c_void;
pub type IRicheditWindowlessAccessibility = *mut ::core::ffi::c_void;
pub type IScrollItemProvider = *mut ::core::ffi::c_void;
pub type IScrollProvider = *mut ::core::ffi::c_void;
pub type ISelectionItemProvider = *mut ::core::ffi::c_void;
pub type ISelectionProvider = *mut ::core::ffi::c_void;
pub type ISelectionProvider2 = *mut ::core::ffi::c_void;
pub type ISpreadsheetItemProvider = *mut ::core::ffi::c_void;
pub type ISpreadsheetProvider = *mut ::core::ffi::c_void;
pub type IStylesProvider = *mut ::core::ffi::c_void;
pub type ISynchronizedInputProvider = *mut ::core::ffi::c_void;
pub type ITableItemProvider = *mut ::core::ffi::c_void;
pub type ITableProvider = *mut ::core::ffi::c_void;
pub type ITextChildProvider = *mut ::core::ffi::c_void;
pub type ITextEditProvider = *mut ::core::ffi::c_void;
pub type ITextProvider = *mut ::core::ffi::c_void;
pub type ITextProvider2 = *mut ::core::ffi::c_void;
pub type ITextRangeProvider = *mut ::core::ffi::c_void;
pub type ITextRangeProvider2 = *mut ::core::ffi::c_void;
pub type IToggleProvider = *mut ::core::ffi::c_void;
pub type ITransformProvider = *mut ::core::ffi::c_void;
pub type ITransformProvider2 = *mut ::core::ffi::c_void;
pub type IUIAutomation = *mut ::core::ffi::c_void;
pub type IUIAutomation2 = *mut ::core::ffi::c_void;
pub type IUIAutomation3 = *mut ::core::ffi::c_void;
pub type IUIAutomation4 = *mut ::core::ffi::c_void;
pub type IUIAutomation5 = *mut ::core::ffi::c_void;
pub type IUIAutomation6 = *mut ::core::ffi::c_void;
pub type IUIAutomationActiveTextPositionChangedEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationAndCondition = *mut ::core::ffi::c_void;
pub type IUIAutomationAnnotationPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationBoolCondition = *mut ::core::ffi::c_void;
pub type IUIAutomationCacheRequest = *mut ::core::ffi::c_void;
pub type IUIAutomationChangesEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationCondition = *mut ::core::ffi::c_void;
pub type IUIAutomationCustomNavigationPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationDockPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationDragPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationDropTargetPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationElement = *mut ::core::ffi::c_void;
pub type IUIAutomationElement2 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement3 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement4 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement5 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement6 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement7 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement8 = *mut ::core::ffi::c_void;
pub type IUIAutomationElement9 = *mut ::core::ffi::c_void;
pub type IUIAutomationElementArray = *mut ::core::ffi::c_void;
pub type IUIAutomationEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationEventHandlerGroup = *mut ::core::ffi::c_void;
pub type IUIAutomationExpandCollapsePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationFocusChangedEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationGridItemPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationGridPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationInvokePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationItemContainerPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationLegacyIAccessiblePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationMultipleViewPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationNotCondition = *mut ::core::ffi::c_void;
pub type IUIAutomationNotificationEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationObjectModelPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationOrCondition = *mut ::core::ffi::c_void;
pub type IUIAutomationPatternHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationPatternInstance = *mut ::core::ffi::c_void;
pub type IUIAutomationPropertyChangedEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationPropertyCondition = *mut ::core::ffi::c_void;
pub type IUIAutomationProxyFactory = *mut ::core::ffi::c_void;
pub type IUIAutomationProxyFactoryEntry = *mut ::core::ffi::c_void;
pub type IUIAutomationProxyFactoryMapping = *mut ::core::ffi::c_void;
pub type IUIAutomationRangeValuePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationRegistrar = *mut ::core::ffi::c_void;
pub type IUIAutomationScrollItemPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationScrollPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationSelectionItemPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationSelectionPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationSelectionPattern2 = *mut ::core::ffi::c_void;
pub type IUIAutomationSpreadsheetItemPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationSpreadsheetPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationStructureChangedEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationStylesPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationSynchronizedInputPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTableItemPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTablePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTextChildPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTextEditPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTextEditTextChangedEventHandler = *mut ::core::ffi::c_void;
pub type IUIAutomationTextPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTextPattern2 = *mut ::core::ffi::c_void;
pub type IUIAutomationTextRange = *mut ::core::ffi::c_void;
pub type IUIAutomationTextRange2 = *mut ::core::ffi::c_void;
pub type IUIAutomationTextRange3 = *mut ::core::ffi::c_void;
pub type IUIAutomationTextRangeArray = *mut ::core::ffi::c_void;
pub type IUIAutomationTogglePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTransformPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationTransformPattern2 = *mut ::core::ffi::c_void;
pub type IUIAutomationTreeWalker = *mut ::core::ffi::c_void;
pub type IUIAutomationValuePattern = *mut ::core::ffi::c_void;
pub type IUIAutomationVirtualizedItemPattern = *mut ::core::ffi::c_void;
pub type IUIAutomationWindowPattern = *mut ::core::ffi::c_void;
pub type IValueProvider = *mut ::core::ffi::c_void;
pub type IVirtualizedItemProvider = *mut ::core::ffi::c_void;
pub type IWindowProvider = *mut ::core::ffi::c_void;
pub const Image_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 758593252, data2: 27414, data3: 19543, data4: [169, 98, 249, 50, 96, 167, 82, 67] };
pub const InputDiscarded_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2134295399, data2: 31512, data3: 16764, data4: [151, 227, 157, 88, 221, 201, 68, 171] };
pub const InputReachedOtherElement_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3978304906, data2: 20076, data3: 16734, data4: [168, 116, 36, 96, 201, 182, 107, 168] };
pub const InputReachedTarget_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2481804442, data2: 1353, data3: 16624, data4: [190, 219, 40, 228, 79, 125, 226, 163] };
pub const Invoke_Invoked_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3755383280, data2: 51477, data3: 18909, data4: [180, 34, 221, 231, 133, 195, 210, 75] };
pub const Invoke_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3648439036, data2: 26346, data3: 19054, data4: [178, 143, 194, 76, 117, 70, 173, 55] };
pub const IsAnnotationPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 190526008, data2: 27996, data3: 16822, data4: [188, 196, 94, 128, 127, 101, 81, 196] };
pub const IsContentElement_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1272603816, data2: 62936, data3: 18443, data4: [129, 85, 239, 46, 137, 173, 182, 114] };
pub const IsControlElement_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2515751045, data2: 43980, data3: 19197, data4: [165, 244, 219, 180, 108, 35, 15, 219] };
pub const IsCustomNavigationPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2408480980, data2: 9041, data3: 18656, data4: [135, 74, 84, 170, 115, 19, 136, 154] };
pub const IsDataValidForForm_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1146799748, data2: 50172, data3: 19929, data4: [172, 248, 132, 90, 87, 146, 150, 186] };
pub const IsDialog_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2634939291, data2: 33846, data3: 17665, data4: [187, 187, 229, 52, 164, 251, 59, 63] };
pub const IsDockPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 637576388, data2: 12280, data3: 19606, data4: [174, 49, 143, 230, 25, 161, 60, 108] };
pub const IsDragPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3919030199, data2: 7481, data3: 19623, data4: [190, 15, 39, 127, 207, 86, 5, 204] };
pub const IsDropTargetPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 109491758, data2: 36377, data3: 19119, data4: [135, 61, 56, 79, 109, 59, 146, 190] };
pub const IsEnabled_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 554254975, data2: 55904, data3: 20461, data4: [191, 27, 38, 75, 220, 230, 235, 58] };
pub const IsExpandCollapsePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2459777030, data2: 21127, data3: 18213, data4: [170, 22, 34, 42, 252, 99, 213, 149] };
pub const IsGridItemPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1514399012, data2: 63906, data3: 19218, data4: [132, 200, 180, 138, 62, 254, 221, 52] };
pub const IsGridPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1445118572, data2: 61679, data3: 20283, data4: [151, 203, 113, 76, 8, 104, 88, 139] };
pub const IsInvokePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1316116280, data2: 33636, data3: 18041, data4: [170, 108, 243, 244, 25, 49, 247, 80] };
pub const IsItemContainerPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1649106087, data2: 65088, data3: 18775, data4: [160, 25, 32, 196, 207, 17, 146, 15] };
pub const IsKeyboardFocusable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4156052778, data2: 2137, data3: 19255, data4: [185, 203, 81, 231, 32, 146, 242, 159] };
pub const IsLegacyIAccessiblePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3639333063, data2: 37530, data3: 20199, data4: [141, 58, 211, 217, 68, 19, 2, 123] };
pub const IsMultipleViewPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4278858219, data2: 36389, data3: 18077, data4: [141, 110, 231, 113, 162, 124, 27, 144] };
pub const IsObjectModelPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1797380251, data2: 10305, data3: 16687, data4: [142, 242, 21, 202, 149, 35, 24, 186] };
pub const IsOffscreen_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 63164768, data2: 56185, data3: 17115, data4: [162, 239, 28, 35, 30, 237, 229, 7] };
pub const IsPassword_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3897044657, data2: 26748, data3: 18811, data4: [190, 188, 3, 190, 83, 236, 20, 84] };
pub const IsPeripheral_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3665134198, data2: 32469, data3: 18900, data4: [142, 104, 236, 201, 162, 211, 0, 221] };
pub const IsRangeValuePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4255392842, data2: 60237, data3: 17407, data4: [181, 173, 237, 54, 211, 115, 236, 76] };
pub const IsRequiredForForm_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1331643343, data2: 23035, data3: 19422, data4: [162, 112, 96, 46, 94, 17, 65, 233] };
pub const IsScrollItemPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 481106437, data2: 2343, data3: 19318, data4: [151, 225, 15, 205, 178, 9, 185, 138] };
pub const IsScrollPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1052474186, data2: 33418, data3: 19287, data4: [157, 34, 47, 234, 22, 50, 237, 13] };
pub const IsSelectionItemPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2347554349, data2: 3011, data3: 16649, data4: [190, 226, 142, 103, 21, 41, 14, 104] };
pub const IsSelectionPattern2Available_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1225262843, data2: 28297, data3: 19015, data4: [131, 25, 210, 102, 229, 17, 240, 33] };
pub const IsSelectionPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4119375038, data2: 51049, data3: 18488, data4: [154, 96, 38, 134, 220, 17, 136, 196] };
pub const IsSpreadsheetItemPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2682755882, data2: 12180, data3: 17405, data4: [153, 107, 84, 158, 49, 111, 74, 205] };
pub const IsSpreadsheetPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1878275890, data2: 58548, data3: 17749, data4: [151, 188, 236, 219, 188, 77, 24, 136] };
pub const IsStructuredMarkupPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2966733206, data2: 11275, data3: 18588, data4: [177, 101, 164, 5, 146, 140, 111, 61] };
pub const IsStylesPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 670258131, data2: 17820, data3: 19289, data4: [164, 144, 80, 97, 29, 172, 175, 181] };
pub const IsSynchronizedInputPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1976999109, data2: 53951, data3: 18755, data4: [135, 110, 180, 91, 98, 166, 204, 102] };
pub const IsTableItemPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3946230797, data2: 36516, data3: 18587, data4: [160, 19, 230, 13, 89, 81, 254, 52] };
pub const IsTablePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3414382431, data2: 17858, data3: 16456, data4: [156, 118, 21, 151, 21, 161, 57, 223] };
pub const IsTextChildPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1436444127, data2: 12543, data3: 17333, data4: [181, 237, 91, 40, 59, 128, 199, 233] };
pub const IsTextEditPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2017673820, data2: 35634, data3: 18508, data4: [154, 181, 227, 32, 5, 113, 255, 218] };
pub const IsTextPattern2Available_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1104122397, data2: 58353, data3: 19234, data4: [156, 129, 225, 195, 237, 51, 28, 34] };
pub const IsTextPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4225947293, data2: 45046, data3: 19013, data4: [130, 226, 252, 146, 168, 47, 89, 23] };
pub const IsTogglePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2020109651, data2: 64720, data3: 19331, data4: [155, 120, 88, 50, 206, 99, 187, 91] };
pub const IsTransformPattern2Available_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 630721355, data2: 48644, data3: 18192, data4: [171, 74, 253, 163, 29, 189, 40, 149] };
pub const IsTransformPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2818017284, data2: 54923, data3: 16503, data4: [165, 198, 122, 94, 161, 172, 49, 197] };
pub const IsValuePatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 189800615, data2: 8473, data3: 18235, data4: [190, 55, 92, 235, 152, 187, 251, 34] };
pub const IsVirtualizedItemPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 808235345, data2: 10952, data3: 17878, data4: [151, 123, 210, 179, 165, 165, 63, 32] };
pub const IsWindowPatternAvailable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3886382001, data2: 22664, data3: 16725, data4: [152, 220, 180, 34, 253, 87, 242, 188] };
pub const ItemContainer_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1024711183, data2: 35738, data3: 19097, data4: [133, 250, 197, 201, 166, 159, 30, 212] };
pub const ItemStatus_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1373504289, data2: 14707, data3: 17383, data4: [137, 19, 11, 8, 232, 19, 195, 127] };
pub const ItemType_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3453633357, data2: 25122, data3: 16699, data4: [166, 138, 50, 93, 209, 212, 15, 57] };
pub const LIBID_Accessibility: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 514120688, data2: 15419, data3: 4559, data4: [129, 12, 0, 170, 0, 56, 155, 113] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub type LPFNACCESSIBLECHILDREN = ::core::option::Option<unsafe extern "system" fn(pacccontainer: IAccessible, ichildstart: i32, cchildren: i32, rgvarchildren: *mut super::super::System::Com::VARIANT, pcobtained: *mut i32) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub type LPFNACCESSIBLEOBJECTFROMPOINT = ::core::option::Option<unsafe extern "system" fn(ptscreen: super::super::Foundation::POINT, ppacc: *mut IAccessible, pvarchild: *mut super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNACCESSIBLEOBJECTFROMWINDOW = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, dwid: u32, riid: *const ::windows_sys::core::GUID, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNCREATESTDACCESSIBLEOBJECT = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, idobject: i32, riid: *const ::windows_sys::core::GUID, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNLRESULTFROMOBJECT = ::core::option::Option<unsafe extern "system" fn(riid: *const ::windows_sys::core::GUID, wparam: super::super::Foundation::WPARAM, punk: ::windows_sys::core::IUnknown) -> super::super::Foundation::LRESULT>;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type LPFNOBJECTFROMLRESULT = ::core::option::Option<unsafe extern "system" fn(lresult: super::super::Foundation::LRESULT, riid: *const ::windows_sys::core::GUID, wparam: super::super::Foundation::WPARAM, ppvobject: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT>;
pub const LabeledBy_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3854078539, data2: 64650, data3: 18997, data4: [128, 49, 207, 120, 172, 67, 229, 94] };
pub const LandmarkType_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1161840114, data2: 28513, data3: 18935, data4: [164, 248, 181, 240, 207, 130, 218, 30] };
pub const LayoutInvalidated_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3984418116, data2: 42685, data3: 17813, data4: [155, 174, 61, 40, 148, 108, 199, 21] };
pub const LegacyIAccessible_ChildId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2585336669, data2: 40690, data3: 18311, data4: [164, 89, 220, 222, 136, 93, 212, 232] };
pub const LegacyIAccessible_DefaultAction_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 993204009, data2: 60077, data3: 17666, data4: [184, 95, 146, 97, 86, 34, 145, 60] };
pub const LegacyIAccessible_Description_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1178895384, data2: 32112, data3: 20137, data4: [157, 39, 183, 231, 117, 207, 42, 215] };
pub const LegacyIAccessible_Help_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2487231314, data2: 5660, data3: 19319, data4: [169, 141, 168, 114, 204, 51, 148, 122] };
pub const LegacyIAccessible_KeyboardShortcut_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2406025644, data2: 184, data3: 16985, data4: [164, 28, 150, 98, 102, 212, 58, 138] };
pub const LegacyIAccessible_Name_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3404400189, data2: 16558, data3: 18537, data4: [170, 90, 27, 142, 93, 102, 103, 57] };
pub const LegacyIAccessible_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1422658207, data2: 13205, data3: 18607, data4: [186, 141, 115, 248, 86, 144, 243, 224] };
pub const LegacyIAccessible_Role_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1750525343, data2: 52143, data3: 20017, data4: [147, 232, 188, 191, 111, 126, 73, 28] };
pub const LegacyIAccessible_Selection_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2326311392, data2: 2193, data3: 16588, data4: [139, 6, 144, 215, 212, 22, 98, 25] };
pub const LegacyIAccessible_State_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3751303252, data2: 8833, data3: 17216, data4: [171, 156, 198, 14, 44, 88, 3, 246] };
pub const LegacyIAccessible_Value_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3049631926, data2: 33303, data3: 19063, data4: [151, 165, 25, 10, 133, 237, 1, 86] };
pub const Level_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 606782761, data2: 52534, data3: 16399, data4: [170, 217, 120, 118, 239, 58, 246, 39] };
pub const ListItem_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2067208178, data2: 17617, data3: 19032, data4: [152, 168, 241, 42, 155, 143, 120, 226] };
pub const List_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2601819873, data2: 31946, data3: 19708, data4: [154, 241, 202, 199, 189, 221, 48, 49] };
pub const LiveRegionChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 271408784, data2: 59049, data3: 16822, data4: [177, 197, 169, 177, 146, 157, 149, 16] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type LiveSetting = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const Off: LiveSetting = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const Polite: LiveSetting = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const Assertive: LiveSetting = 2i32;
pub const LiveSetting_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3240873358, data2: 10894, data3: 18768, data4: [138, 231, 54, 37, 17, 29, 88, 235] };
pub const LocalizedControlType_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2271428687, data2: 41405, data3: 17706, data4: [137, 196, 63, 1, 211, 131, 56, 6] };
pub const LocalizedLandmarkType_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2059934080, data2: 60155, data3: 20402, data4: [191, 145, 244, 133, 190, 245, 232, 225] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct MOUSEKEYS {
    pub cbSize: u32,
    pub dwFlags: u32,
    pub iMaxSpeed: u32,
    pub iTimeToMaxSpeed: u32,
    pub iCtrlSpeed: u32,
    pub dwReserved1: u32,
    pub dwReserved2: u32,
}
impl ::core::marker::Copy for MOUSEKEYS {}
impl ::core::clone::Clone for MOUSEKEYS {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct MSAAMENUINFO {
    pub dwMSAASignature: u32,
    pub cchWText: u32,
    pub pszWText: ::windows_sys::core::PWSTR,
}
impl ::core::marker::Copy for MSAAMENUINFO {}
impl ::core::clone::Clone for MSAAMENUINFO {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const MSAA_MENU_SIG: i32 = -1441927155i32;
pub const MenuBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3426239056, data2: 3707, data3: 19176, data4: [149, 174, 160, 143, 38, 27, 82, 238] };
pub const MenuClosed_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1022436974, data2: 5506, data3: 16449, data4: [172, 215, 136, 163, 90, 150, 82, 151] };
pub const MenuItem_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4099024339, data2: 53408, data3: 18904, data4: [152, 52, 154, 0, 13, 42, 237, 220] };
pub const MenuModeEnd_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2664254623, data2: 32989, data3: 18360, data4: [130, 103, 90, 236, 6, 187, 44, 255] };
pub const MenuModeStart_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 416794161, data2: 5738, data3: 19145, data4: [174, 59, 239, 75, 84, 32, 230, 129] };
pub const MenuOpened_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3957516613, data2: 26314, data3: 20177, data4: [159, 248, 42, 215, 223, 10, 27, 8] };
pub const Menu_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 781915200, data2: 3752, data3: 16893, data4: [179, 116, 193, 234, 111, 80, 60, 209] };
pub const MultipleView_CurrentView_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2055317114, data2: 47439, data3: 18549, data4: [145, 139, 101, 200, 210, 249, 152, 229] };
pub const MultipleView_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1417308900, data2: 4415, data3: 18372, data4: [133, 15, 219, 77, 250, 70, 107, 29] };
pub const MultipleView_SupportedViews_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2371729917, data2: 52796, data3: 19175, data4: [183, 136, 64, 10, 60, 100, 85, 71] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_DOWN: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_FIRSTCHILD: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_LASTCHILD: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_LEFT: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_MAX: u32 = 9u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_MIN: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_NEXT: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_PREVIOUS: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_RIGHT: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NAVDIR_UP: u32 = 1u32;
pub const Name_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3282473499, data2: 19097, data3: 17649, data4: [188, 166, 97, 24, 112, 82, 196, 49] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type NavigateDirection = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NavigateDirection_Parent: NavigateDirection = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NavigateDirection_NextSibling: NavigateDirection = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NavigateDirection_PreviousSibling: NavigateDirection = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NavigateDirection_FirstChild: NavigateDirection = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NavigateDirection_LastChild: NavigateDirection = 4i32;
pub const NewNativeWindowHandle_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1368830779, data2: 14346, data3: 18818, data4: [149, 225, 145, 243, 239, 96, 224, 36] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type NormalizeState = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NormalizeState_None: NormalizeState = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NormalizeState_View: NormalizeState = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NormalizeState_Custom: NormalizeState = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type NotificationKind = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationKind_ItemAdded: NotificationKind = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationKind_ItemRemoved: NotificationKind = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationKind_ActionCompleted: NotificationKind = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationKind_ActionAborted: NotificationKind = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationKind_Other: NotificationKind = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type NotificationProcessing = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationProcessing_ImportantAll: NotificationProcessing = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationProcessing_ImportantMostRecent: NotificationProcessing = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationProcessing_All: NotificationProcessing = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationProcessing_MostRecent: NotificationProcessing = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const NotificationProcessing_CurrentThenMostRecent: NotificationProcessing = 4i32;
pub const Notification_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1925554935, data2: 38792, data3: 18447, data4: [184, 235, 77, 238, 0, 246, 24, 111] };
pub const ObjectModel_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1040493822, data2: 2300, data3: 18412, data4: [150, 188, 53, 63, 163, 179, 74, 167] };
pub const OptimizeForVisualContent_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1787109968, data2: 51034, data3: 20061, data4: [184, 88, 227, 129, 176, 247, 136, 97] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type OrientationType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OrientationType_None: OrientationType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OrientationType_Horizontal: OrientationType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OrientationType_Vertical: OrientationType = 2i32;
pub const Orientation_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2686381666, data2: 14468, data3: 17429, data4: [136, 126, 103, 142, 194, 30, 57, 186] };
pub const OutlineColor_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3281376960, data2: 19285, data3: 18274, data4: [160, 115, 253, 48, 58, 99, 79, 82] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type OutlineStyles = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OutlineStyles_None: OutlineStyles = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OutlineStyles_Outline: OutlineStyles = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OutlineStyles_Shadow: OutlineStyles = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OutlineStyles_Engraved: OutlineStyles = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const OutlineStyles_Embossed: OutlineStyles = 8i32;
pub const OutlineThickness_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 333872327, data2: 56002, data3: 18568, data4: [189, 211, 55, 92, 98, 250, 150, 24] };
pub const PROPID_ACC_DEFAULTACTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 403441451, data2: 49791, data3: 17351, data4: [153, 34, 246, 53, 98, 164, 99, 43] };
pub const PROPID_ACC_DESCRIPTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1296621540, data2: 48447, data3: 18719, data4: [166, 72, 73, 45, 111, 32, 197, 136] };
pub const PROPID_ACC_DESCRIPTIONMAP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 535905119, data2: 35348, data3: 18299, data4: [178, 38, 160, 171, 226, 121, 151, 93] };
pub const PROPID_ACC_DODEFAULTACTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 463508771, data2: 11835, data3: 18854, data4: [160, 89, 89, 104, 42, 60, 72, 253] };
pub const PROPID_ACC_FOCUS: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1857238495, data2: 7209, data3: 16679, data4: [177, 44, 222, 233, 253, 21, 127, 43] };
pub const PROPID_ACC_HELP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3358712095, data2: 17627, data3: 19097, data4: [151, 104, 203, 143, 151, 139, 114, 49] };
pub const PROPID_ACC_HELPTOPIC: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2021462905, data2: 36574, data3: 17419, data4: [138, 236, 17, 247, 191, 144, 48, 179] };
pub const PROPID_ACC_KEYBOARDSHORTCUT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2107363054, data2: 32030, data3: 18809, data4: [147, 130, 81, 128, 244, 23, 44, 52] };
pub const PROPID_ACC_NAME: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1619869176, data2: 33064, data3: 19111, data4: [164, 40, 245, 94, 73, 38, 114, 145] };
pub const PROPID_ACC_NAV_DOWN: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 51802349, data2: 15583, data3: 18642, data4: [150, 19, 19, 143, 45, 216, 166, 104] };
pub const PROPID_ACC_NAV_FIRSTCHILD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3486524760, data2: 21883, data3: 19559, data4: [132, 249, 42, 9, 252, 228, 7, 73] };
pub const PROPID_ACC_NAV_LASTCHILD: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 808372901, data2: 18645, data3: 20365, data4: [182, 113, 26, 141, 32, 167, 120, 50] };
pub const PROPID_ACC_NAV_LEFT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 578848459, data2: 33521, data3: 19001, data4: [135, 5, 220, 220, 15, 255, 146, 245] };
pub const PROPID_ACC_NAV_NEXT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 484201557, data2: 36057, data3: 19602, data4: [163, 113, 57, 57, 162, 254, 62, 238] };
pub const PROPID_ACC_NAV_PREV: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2003646609, data2: 51003, data3: 17536, data4: [179, 246, 7, 106, 22, 161, 90, 246] };
pub const PROPID_ACC_NAV_RIGHT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3441499551, data2: 57803, data3: 20453, data4: [167, 124, 146, 11, 136, 77, 9, 91] };
pub const PROPID_ACC_NAV_UP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 23992875, data2: 6734, data3: 18279, data4: [134, 18, 51, 134, 246, 105, 53, 236] };
pub const PROPID_ACC_PARENT: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1196171958, data2: 65474, data3: 18042, data4: [177, 181, 233, 88, 180, 101, 115, 48] };
pub const PROPID_ACC_ROLE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3415236594, data2: 31697, data3: 19461, data4: [179, 200, 230, 194, 65, 54, 77, 112] };
pub const PROPID_ACC_ROLEMAP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4154117538, data2: 5133, data3: 20454, data4: [137, 20, 32, 132, 118, 50, 130, 105] };
pub const PROPID_ACC_SELECTION: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3114075964, data2: 55089, data3: 16475, data4: [144, 97, 217, 94, 143, 132, 41, 132] };
pub const PROPID_ACC_STATE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2832520624, data2: 2593, data3: 17104, data4: [165, 192, 81, 78, 152, 79, 69, 123] };
pub const PROPID_ACC_STATEMAP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1133800542, data2: 2752, data3: 16450, data4: [181, 37, 7, 187, 219, 225, 127, 167] };
pub const PROPID_ACC_VALUE: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 306177091, data2: 8474, data3: 17941, data4: [149, 39, 196, 90, 126, 147, 113, 122] };
pub const PROPID_ACC_VALUEMAP: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3659283833, data2: 64604, data3: 16910, data4: [179, 153, 157, 21, 51, 84, 158, 117] };
pub const Pane_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1546338139, data2: 37250, data3: 17059, data4: [141, 236, 140, 4, 193, 238, 99, 77] };
pub const PositionInSet_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 869391444, data2: 25630, data3: 19830, data4: [166, 177, 19, 243, 65, 193, 248, 150] };
pub const ProcessId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1078565272, data2: 39985, data3: 16965, data4: [164, 3, 135, 50, 14, 89, 234, 246] };
pub const ProgressBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 579641222, data2: 50028, data3: 18363, data4: [159, 182, 165, 131, 75, 252, 83, 164] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type PropertyConditionFlags = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const PropertyConditionFlags_None: PropertyConditionFlags = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const PropertyConditionFlags_IgnoreCase: PropertyConditionFlags = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const PropertyConditionFlags_MatchSubstring: PropertyConditionFlags = 2i32;
pub const ProviderDescription_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3701829770, data2: 49515, data3: 19673, data4: [184, 137, 190, 177, 106, 128, 73, 4] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ProviderOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_ClientSideProvider: ProviderOptions = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_ServerSideProvider: ProviderOptions = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_NonClientAreaProvider: ProviderOptions = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_OverrideProvider: ProviderOptions = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_ProviderOwnsSetFocus: ProviderOptions = 16i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_UseComThreading: ProviderOptions = 32i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_RefuseNonClientSupport: ProviderOptions = 64i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_HasNativeIAccessible: ProviderOptions = 128i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderOptions_UseClientCoordinates: ProviderOptions = 256i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ProviderType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderType_BaseHwnd: ProviderType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderType_Proxy: ProviderType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ProviderType_NonClientArea: ProviderType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_ALERT: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_ANIMATION: u32 = 54u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_APPLICATION: u32 = 14u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_BORDER: u32 = 19u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_BUTTONDROPDOWN: u32 = 56u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_BUTTONDROPDOWNGRID: u32 = 58u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_BUTTONMENU: u32 = 57u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CARET: u32 = 7u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CELL: u32 = 29u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CHARACTER: u32 = 32u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CHART: u32 = 17u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CHECKBUTTON: u32 = 44u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CLIENT: u32 = 10u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CLOCK: u32 = 61u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_COLUMN: u32 = 27u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_COLUMNHEADER: u32 = 25u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_COMBOBOX: u32 = 46u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_CURSOR: u32 = 6u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_DIAGRAM: u32 = 53u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_DIAL: u32 = 49u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_DIALOG: u32 = 18u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_DOCUMENT: u32 = 15u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_DROPLIST: u32 = 47u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_EQUATION: u32 = 55u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_GRAPHIC: u32 = 40u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_GRIP: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_GROUPING: u32 = 20u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_HELPBALLOON: u32 = 31u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_HOTKEYFIELD: u32 = 50u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_INDICATOR: u32 = 39u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_IPADDRESS: u32 = 63u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_LINK: u32 = 30u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_LIST: u32 = 33u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_LISTITEM: u32 = 34u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_MENUBAR: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_MENUITEM: u32 = 12u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_MENUPOPUP: u32 = 11u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_OUTLINE: u32 = 35u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_OUTLINEBUTTON: u32 = 64u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_OUTLINEITEM: u32 = 36u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_PAGETAB: u32 = 37u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_PAGETABLIST: u32 = 60u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_PANE: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_PROGRESSBAR: u32 = 48u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_PROPERTYPAGE: u32 = 38u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_PUSHBUTTON: u32 = 43u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_RADIOBUTTON: u32 = 45u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_ROW: u32 = 28u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_ROWHEADER: u32 = 26u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_SCROLLBAR: u32 = 3u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_SEPARATOR: u32 = 21u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_SLIDER: u32 = 51u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_SOUND: u32 = 5u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_SPINBUTTON: u32 = 52u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_SPLITBUTTON: u32 = 62u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_STATICTEXT: u32 = 41u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_STATUSBAR: u32 = 23u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_TABLE: u32 = 24u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_TEXT: u32 = 42u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_TITLEBAR: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_TOOLBAR: u32 = 22u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_TOOLTIP: u32 = 13u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_WHITESPACE: u32 = 59u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ROLE_SYSTEM_WINDOW: u32 = 9u32;
pub const RadioButton_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1004227035, data2: 65068, data3: 17539, data4: [179, 225, 229, 127, 33, 148, 64, 198] };
pub const RangeValue_IsReadOnly_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 637145173, data2: 57023, data3: 17267, data4: [167, 158, 31, 26, 25, 8, 211, 196] };
pub const RangeValue_LargeChange_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2717475621, data2: 14909, data3: 19268, data4: [142, 31, 74, 70, 217, 132, 64, 25] };
pub const RangeValue_Maximum_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 422680852, data2: 63865, data3: 19253, data4: [161, 166, 211, 126, 5, 67, 52, 115] };
pub const RangeValue_Minimum_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2026623922, data2: 26701, data3: 18528, data4: [175, 147, 209, 249, 92, 176, 34, 253] };
pub const RangeValue_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 414190983, data2: 45513, data3: 18282, data4: [191, 189, 95, 11, 219, 146, 111, 99] };
pub const RangeValue_SmallChange_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2177025111, data2: 14657, data3: 16647, data4: [153, 117, 19, 151, 96, 247, 192, 114] };
pub const RangeValue_Value_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 320822680, data2: 50444, data3: 18589, data4: [171, 229, 174, 34, 8, 152, 197, 247] };
pub const Rotation_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1987894397, data2: 44736, data3: 16656, data4: [173, 50, 48, 237, 212, 3, 73, 46] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type RowOrColumnMajor = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const RowOrColumnMajor_RowMajor: RowOrColumnMajor = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const RowOrColumnMajor_ColumnMajor: RowOrColumnMajor = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const RowOrColumnMajor_Indeterminate: RowOrColumnMajor = 2i32;
pub const RuntimeId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2745101306, data2: 32698, data3: 19593, data4: [180, 212, 185, 158, 45, 231, 209, 96] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_ADDSELECTION: u32 = 8u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_EXTENDSELECTION: u32 = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_NONE: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_REMOVESELECTION: u32 = 16u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_TAKEFOCUS: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_TAKESELECTION: u32 = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SELFLAG_VALID: u32 = 31u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct SERIALKEYSA {
    pub cbSize: u32,
    pub dwFlags: SERIALKEYS_FLAGS,
    pub lpszActivePort: ::windows_sys::core::PSTR,
    pub lpszPort: ::windows_sys::core::PSTR,
    pub iBaudRate: u32,
    pub iPortState: u32,
    pub iActive: u32,
}
impl ::core::marker::Copy for SERIALKEYSA {}
impl ::core::clone::Clone for SERIALKEYSA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct SERIALKEYSW {
    pub cbSize: u32,
    pub dwFlags: SERIALKEYS_FLAGS,
    pub lpszActivePort: ::windows_sys::core::PWSTR,
    pub lpszPort: ::windows_sys::core::PWSTR,
    pub iBaudRate: u32,
    pub iPortState: u32,
    pub iActive: u32,
}
impl ::core::marker::Copy for SERIALKEYSW {}
impl ::core::clone::Clone for SERIALKEYSW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SERIALKEYS_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SERKF_AVAILABLE: SERIALKEYS_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SERKF_INDICATOR: SERIALKEYS_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SERKF_SERIALKEYSON: SERIALKEYS_FLAGS = 1u32;
pub const SID_ControlElementProvider: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4101578088, data2: 57940, data3: 19363, data4: [154, 83, 38, 165, 197, 73, 121, 70] };
pub const SID_IsUIAutomationObject: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3111115653, data2: 29188, data3: 18212, data4: [132, 43, 199, 5, 157, 237, 185, 208] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct SOUNDSENTRYA {
    pub cbSize: u32,
    pub dwFlags: SOUNDSENTRY_FLAGS,
    pub iFSTextEffect: SOUNDSENTRY_TEXT_EFFECT,
    pub iFSTextEffectMSec: u32,
    pub iFSTextEffectColorBits: u32,
    pub iFSGrafEffect: SOUND_SENTRY_GRAPHICS_EFFECT,
    pub iFSGrafEffectMSec: u32,
    pub iFSGrafEffectColor: u32,
    pub iWindowsEffect: SOUNDSENTRY_WINDOWS_EFFECT,
    pub iWindowsEffectMSec: u32,
    pub lpszWindowsEffectDLL: ::windows_sys::core::PSTR,
    pub iWindowsEffectOrdinal: u32,
}
impl ::core::marker::Copy for SOUNDSENTRYA {}
impl ::core::clone::Clone for SOUNDSENTRYA {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct SOUNDSENTRYW {
    pub cbSize: u32,
    pub dwFlags: SOUNDSENTRY_FLAGS,
    pub iFSTextEffect: SOUNDSENTRY_TEXT_EFFECT,
    pub iFSTextEffectMSec: u32,
    pub iFSTextEffectColorBits: u32,
    pub iFSGrafEffect: SOUND_SENTRY_GRAPHICS_EFFECT,
    pub iFSGrafEffectMSec: u32,
    pub iFSGrafEffectColor: u32,
    pub iWindowsEffect: SOUNDSENTRY_WINDOWS_EFFECT,
    pub iWindowsEffectMSec: u32,
    pub lpszWindowsEffectDLL: ::windows_sys::core::PWSTR,
    pub iWindowsEffectOrdinal: u32,
}
impl ::core::marker::Copy for SOUNDSENTRYW {}
impl ::core::clone::Clone for SOUNDSENTRYW {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SOUNDSENTRY_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSF_SOUNDSENTRYON: SOUNDSENTRY_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSF_AVAILABLE: SOUNDSENTRY_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSF_INDICATOR: SOUNDSENTRY_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SOUNDSENTRY_TEXT_EFFECT = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSTF_BORDER: SOUNDSENTRY_TEXT_EFFECT = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSTF_CHARS: SOUNDSENTRY_TEXT_EFFECT = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSTF_DISPLAY: SOUNDSENTRY_TEXT_EFFECT = 3u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSTF_NONE: SOUNDSENTRY_TEXT_EFFECT = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SOUNDSENTRY_WINDOWS_EFFECT = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSWF_CUSTOM: SOUNDSENTRY_WINDOWS_EFFECT = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSWF_DISPLAY: SOUNDSENTRY_WINDOWS_EFFECT = 3u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSWF_NONE: SOUNDSENTRY_WINDOWS_EFFECT = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSWF_TITLE: SOUNDSENTRY_WINDOWS_EFFECT = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSWF_WINDOW: SOUNDSENTRY_WINDOWS_EFFECT = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SOUND_SENTRY_GRAPHICS_EFFECT = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSGF_DISPLAY: SOUND_SENTRY_GRAPHICS_EFFECT = 3u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SSGF_NONE: SOUND_SENTRY_GRAPHICS_EFFECT = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const STATE_SYSTEM_HASPOPUP: u32 = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const STATE_SYSTEM_NORMAL: u32 = 0u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct STICKYKEYS {
    pub cbSize: u32,
    pub dwFlags: STICKYKEYS_FLAGS,
}
impl ::core::marker::Copy for STICKYKEYS {}
impl ::core::clone::Clone for STICKYKEYS {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type STICKYKEYS_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_STICKYKEYSON: STICKYKEYS_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_AVAILABLE: STICKYKEYS_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_HOTKEYACTIVE: STICKYKEYS_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_CONFIRMHOTKEY: STICKYKEYS_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_HOTKEYSOUND: STICKYKEYS_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_INDICATOR: STICKYKEYS_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_AUDIBLEFEEDBACK: STICKYKEYS_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_TRISTATE: STICKYKEYS_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_TWOKEYSOFF: STICKYKEYS_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LALTLATCHED: STICKYKEYS_FLAGS = 268435456u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LCTLLATCHED: STICKYKEYS_FLAGS = 67108864u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LSHIFTLATCHED: STICKYKEYS_FLAGS = 16777216u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RALTLATCHED: STICKYKEYS_FLAGS = 536870912u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RCTLLATCHED: STICKYKEYS_FLAGS = 134217728u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RSHIFTLATCHED: STICKYKEYS_FLAGS = 33554432u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LWINLATCHED: STICKYKEYS_FLAGS = 1073741824u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RWINLATCHED: STICKYKEYS_FLAGS = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LALTLOCKED: STICKYKEYS_FLAGS = 1048576u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LCTLLOCKED: STICKYKEYS_FLAGS = 262144u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LSHIFTLOCKED: STICKYKEYS_FLAGS = 65536u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RALTLOCKED: STICKYKEYS_FLAGS = 2097152u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RCTLLOCKED: STICKYKEYS_FLAGS = 524288u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RSHIFTLOCKED: STICKYKEYS_FLAGS = 131072u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_LWINLOCKED: STICKYKEYS_FLAGS = 4194304u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SKF_RWINLOCKED: STICKYKEYS_FLAGS = 8388608u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SayAsInterpretAs = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_None: SayAsInterpretAs = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Spell: SayAsInterpretAs = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Cardinal: SayAsInterpretAs = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Ordinal: SayAsInterpretAs = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Number: SayAsInterpretAs = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date: SayAsInterpretAs = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Time: SayAsInterpretAs = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Telephone: SayAsInterpretAs = 7i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Currency: SayAsInterpretAs = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Net: SayAsInterpretAs = 9i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Url: SayAsInterpretAs = 10i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Address: SayAsInterpretAs = 11i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Alphanumeric: SayAsInterpretAs = 12i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Name: SayAsInterpretAs = 13i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Media: SayAsInterpretAs = 14i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_MonthDayYear: SayAsInterpretAs = 15i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_DayMonthYear: SayAsInterpretAs = 16i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_YearMonthDay: SayAsInterpretAs = 17i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_YearMonth: SayAsInterpretAs = 18i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_MonthYear: SayAsInterpretAs = 19i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_DayMonth: SayAsInterpretAs = 20i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_MonthDay: SayAsInterpretAs = 21i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Date_Year: SayAsInterpretAs = 22i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Time_HoursMinutesSeconds12: SayAsInterpretAs = 23i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Time_HoursMinutes12: SayAsInterpretAs = 24i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Time_HoursMinutesSeconds24: SayAsInterpretAs = 25i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SayAsInterpretAs_Time_HoursMinutes24: SayAsInterpretAs = 26i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ScrollAmount = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ScrollAmount_LargeDecrement: ScrollAmount = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ScrollAmount_SmallDecrement: ScrollAmount = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ScrollAmount_NoAmount: ScrollAmount = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ScrollAmount_LargeIncrement: ScrollAmount = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ScrollAmount_SmallIncrement: ScrollAmount = 4i32;
pub const ScrollBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3673377590, data2: 20581, data3: 18758, data4: [178, 47, 146, 89, 95, 192, 117, 26] };
pub const ScrollItem_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1167183877, data2: 43011, data3: 19804, data4: [180, 213, 141, 40, 0, 249, 6, 167] };
pub const Scroll_HorizontalScrollPercent_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3351329806, data2: 60193, data3: 18431, data4: [172, 196, 181, 163, 53, 15, 81, 145] };
pub const Scroll_HorizontalViewSize_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1891821012, data2: 64688, data3: 18195, data4: [169, 170, 175, 146, 255, 121, 228, 205] };
pub const Scroll_HorizontallyScrollable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2341622087, data2: 10445, data3: 18862, data4: [189, 99, 244, 65, 24, 210, 231, 25] };
pub const Scroll_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2304746676, data2: 30109, data3: 19536, data4: [142, 21, 3, 70, 6, 114, 0, 60] };
pub const Scroll_VerticalScrollPercent_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1821208729, data2: 45736, data3: 18760, data4: [191, 247, 60, 249, 5, 139, 254, 251] };
pub const Scroll_VerticalViewSize_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3731500578, data2: 55495, data3: 16581, data4: [131, 186, 229, 246, 129, 213, 49, 8] };
pub const Scroll_VerticallyScrollable_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2299938712, data2: 104, data3: 17173, data4: [184, 154, 30, 124, 251, 188, 61, 252] };
pub const Selection2_CurrentSelectedItem_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 874871846, data2: 33717, data3: 16806, data4: [147, 156, 174, 132, 28, 19, 98, 54] };
pub const Selection2_FirstSelectedItem_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3424971367, data2: 13980, data3: 20053, data4: [159, 247, 56, 218, 105, 84, 12, 41] };
pub const Selection2_ItemCount_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3142183839, data2: 17773, data3: 16456, data4: [181, 145, 156, 32, 38, 184, 70, 54] };
pub const Selection2_LastSelectedItem_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3481000592, data2: 11651, data3: 18936, data4: [134, 12, 156, 227, 148, 207, 137, 180] };
pub const SelectionItem_ElementAddedToSelectionEvent_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1015164369, data2: 50183, data3: 19898, data4: [145, 221, 121, 212, 174, 208, 174, 198] };
pub const SelectionItem_ElementRemovedFromSelectionEvent_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 159361193, data2: 28793, data3: 16815, data4: [139, 156, 9, 52, 216, 48, 94, 92] };
pub const SelectionItem_ElementSelectedEvent_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3116882939, data2: 20158, data3: 17714, data4: [170, 244, 0, 140, 246, 71, 35, 60] };
pub const SelectionItem_IsSelected_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4045570911, data2: 52575, data3: 17375, data4: [183, 157, 75, 132, 158, 158, 96, 32] };
pub const SelectionItem_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2613464811, data2: 34759, data3: 19240, data4: [148, 187, 77, 159, 164, 55, 182, 239] };
pub const SelectionItem_SelectionContainer_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2755025774, data2: 39966, data3: 19299, data4: [139, 83, 194, 66, 29, 209, 232, 251] };
pub const Selection_CanSelectMultiple_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1238842789, data2: 51331, data3: 17664, data4: [136, 61, 143, 207, 141, 175, 108, 190] };
pub const Selection_InvalidatedEvent_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3401664772, data2: 5812, data3: 19283, data4: [142, 71, 76, 177, 223, 38, 123, 183] };
pub const Selection_IsSelectionRequired_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2980987938, data2: 25598, data3: 17639, data4: [165, 165, 167, 56, 200, 41, 177, 154] };
pub const Selection_Pattern2_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4221721771, data2: 43928, data3: 18935, data4: [167, 220, 254, 83, 157, 193, 91, 231] };
pub const Selection_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1726199784, data2: 55329, data3: 19749, data4: [135, 97, 67, 93, 44, 139, 37, 63] };
pub const Selection_Selection_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2859319970, data2: 3627, data3: 19768, data4: [150, 213, 52, 228, 112, 184, 24, 83] };
pub const SemanticZoom_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1607682627, data2: 1566, data3: 17096, data4: [181, 137, 157, 204, 247, 75, 196, 58] };
pub const Separator_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2271734691, data2: 10851, data3: 19120, data4: [172, 141, 170, 80, 226, 61, 233, 120] };
pub const SizeOfSet_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 369152828, data2: 15263, data3: 17257, data4: [148, 49, 170, 41, 63, 52, 76, 241] };
pub const Size_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 727676445, data2: 63621, data3: 17412, data4: [151, 63, 155, 29, 152, 227, 109, 143] };
pub const Slider_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2956182091, data2: 15157, data3: 19690, data4: [182, 9, 118, 54, 130, 250, 102, 11] };
pub const Spinner_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1624001336, data2: 15537, data3: 16737, data4: [180, 66, 198, 183, 38, 193, 120, 37] };
pub const SplitButton_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1880223775, data2: 19150, data3: 18689, data4: [180, 97, 146, 10, 111, 28, 166, 80] };
pub const SpreadsheetItem_AnnotationObjects_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2736344120, data2: 51644, data3: 17924, data4: [147, 150, 174, 63, 159, 69, 127, 123] };
pub const SpreadsheetItem_AnnotationTypes_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3339473360, data2: 54786, data3: 19269, data4: [175, 188, 180, 113, 43, 150, 215, 43] };
pub const SpreadsheetItem_Formula_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3858949245, data2: 6983, data3: 19434, data4: [135, 207, 59, 11, 11, 92, 21, 182] };
pub const SpreadsheetItem_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 852460543, data2: 61864, data3: 19084, data4: [134, 88, 212, 123, 167, 78, 32, 186] };
pub const Spreadsheet_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1784358089, data2: 40222, data3: 19333, data4: [158, 68, 192, 46, 49, 105, 177, 11] };
pub const StatusBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3562962203, data2: 22643, data3: 18271, data4: [149, 164, 4, 51, 225, 241, 176, 10] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type StructureChangeType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StructureChangeType_ChildAdded: StructureChangeType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StructureChangeType_ChildRemoved: StructureChangeType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StructureChangeType_ChildrenInvalidated: StructureChangeType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StructureChangeType_ChildrenBulkAdded: StructureChangeType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StructureChangeType_ChildrenBulkRemoved: StructureChangeType = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StructureChangeType_ChildrenReordered: StructureChangeType = 5i32;
pub const StructureChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1503099233, data2: 16093, data3: 19217, data4: [177, 59, 103, 107, 42, 42, 108, 169] };
pub const StructuredMarkup_CompositionComplete_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3297393687, data2: 26490, data3: 16455, data4: [166, 141, 252, 18, 87, 82, 138, 239] };
pub const StructuredMarkup_Deleted_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4191199264, data2: 57793, data3: 20175, data4: [185, 170, 82, 239, 222, 126, 65, 225] };
pub const StructuredMarkup_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2881292408, data2: 34405, data3: 20316, data4: [148, 252, 54, 231, 216, 187, 112, 107] };
pub const StructuredMarkup_SelectionChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2814907895, data2: 65439, data3: 16839, data4: [163, 167, 171, 108, 191, 219, 73, 3] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_BulletedList: i32 = 70015i32;
pub const StyleId_BulletedList_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1499721060, data2: 25638, data3: 17970, data4: [140, 175, 163, 42, 212, 2, 217, 26] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Custom: i32 = 70000i32;
pub const StyleId_Custom_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4012825918, data2: 43417, data3: 19324, data4: [163, 120, 9, 187, 213, 42, 53, 22] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Emphasis: i32 = 70013i32;
pub const StyleId_Emphasis_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3396238782, data2: 13662, data3: 18464, data4: [149, 160, 146, 95, 4, 29, 52, 112] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading1: i32 = 70001i32;
pub const StyleId_Heading1_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2139000681, data2: 26726, data3: 17953, data4: [147, 12, 154, 93, 12, 165, 150, 28] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading2: i32 = 70002i32;
pub const StyleId_Heading2_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3131683393, data2: 23657, data3: 18077, data4: [133, 173, 71, 71, 55, 181, 43, 20] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading3: i32 = 70003i32;
pub const StyleId_Heading3_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3213617618, data2: 55480, data3: 20165, data4: [140, 82, 156, 251, 13, 3, 89, 112] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading4: i32 = 70004i32;
pub const StyleId_Heading4_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2218196928, data2: 38264, data3: 17916, data4: [131, 164, 255, 64, 5, 51, 21, 221] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading5: i32 = 70005i32;
pub const StyleId_Heading5_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2426356301, data2: 3519, data3: 16494, data4: [151, 187, 78, 119, 61, 151, 152, 247] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading6: i32 = 70006i32;
pub const StyleId_Heading6_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2312254553, data2: 23899, data3: 18468, data4: [164, 32, 17, 211, 237, 130, 228, 15] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading7: i32 = 70007i32;
pub const StyleId_Heading7_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2742617203, data2: 59822, data3: 16941, data4: [184, 227, 59, 103, 92, 97, 129, 164] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading8: i32 = 70008i32;
pub const StyleId_Heading8_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 734085445, data2: 41996, data3: 18561, data4: [132, 174, 242, 35, 86, 133, 56, 12] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Heading9: i32 = 70009i32;
pub const StyleId_Heading9_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3339555123, data2: 47914, data3: 17363, data4: [138, 198, 51, 101, 120, 132, 176, 240] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Normal: i32 = 70012i32;
pub const StyleId_Normal_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3440694313, data2: 58462, data3: 17525, data4: [161, 197, 127, 158, 107, 233, 110, 186] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_NumberedList: i32 = 70016i32;
pub const StyleId_NumberedList_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 513203157, data2: 25795, data3: 17360, data4: [177, 238, 181, 59, 6, 227, 237, 223] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Quote: i32 = 70014i32;
pub const StyleId_Quote_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1562124778, data2: 33173, data3: 20332, data4: [135, 234, 93, 171, 236, 230, 76, 29] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Subtitle: i32 = 70011i32;
pub const StyleId_Subtitle_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3050961943, data2: 23919, data3: 17440, data4: [180, 57, 124, 177, 154, 212, 52, 226] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const StyleId_Title: i32 = 70010i32;
pub const StyleId_Title_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 366485530, data2: 65487, data3: 18463, data4: [176, 161, 48, 182, 59, 233, 143, 7] };
pub const Styles_ExtendedProperties_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4099001760, data2: 47626, data3: 18049, data4: [176, 176, 13, 189, 181, 62, 88, 243] };
pub const Styles_FillColor_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1676671354, data2: 41413, data3: 19229, data4: [132, 235, 183, 101, 242, 237, 214, 50] };
pub const Styles_FillPatternColor_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2476366334, data2: 36797, data3: 20085, data4: [162, 113, 172, 69, 149, 25, 81, 99] };
pub const Styles_FillPatternStyle_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2177852703, data2: 18475, data3: 17489, data4: [163, 10, 225, 84, 94, 85, 79, 184] };
pub const Styles_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 451290709, data2: 55922, data3: 19808, data4: [161, 83, 229, 170, 105, 136, 227, 191] };
pub const Styles_Shape_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3340379128, data2: 30604, data3: 16397, data4: [132, 88, 59, 84, 62, 82, 105, 132] };
pub const Styles_StyleId_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3665986863, data2: 14359, data3: 16947, data4: [130, 175, 2, 39, 158, 114, 204, 119] };
pub const Styles_StyleName_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 470986805, data2: 1489, data3: 20309, data4: [158, 142, 20, 137, 243, 255, 85, 13] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SupportedTextSelection = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SupportedTextSelection_None: SupportedTextSelection = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SupportedTextSelection_Single: SupportedTextSelection = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SupportedTextSelection_Multiple: SupportedTextSelection = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type SynchronizedInputType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SynchronizedInputType_KeyUp: SynchronizedInputType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SynchronizedInputType_KeyDown: SynchronizedInputType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SynchronizedInputType_LeftMouseUp: SynchronizedInputType = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SynchronizedInputType_LeftMouseDown: SynchronizedInputType = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SynchronizedInputType_RightMouseUp: SynchronizedInputType = 16i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const SynchronizedInputType_RightMouseDown: SynchronizedInputType = 32i32;
pub const SynchronizedInput_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 96635046, data2: 50299, data3: 18571, data4: [182, 83, 51, 151, 122, 85, 27, 139] };
pub const SystemAlert_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3530642525, data2: 31290, data3: 18343, data4: [132, 116, 129, 210, 154, 36, 81, 201] };
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct TOGGLEKEYS {
    pub cbSize: u32,
    pub dwFlags: u32,
}
impl ::core::marker::Copy for TOGGLEKEYS {}
impl ::core::clone::Clone for TOGGLEKEYS {
    fn clone(&self) -> Self {
        *self
    }
}
pub const TabItem_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 745169743, data2: 37403, data3: 20078, data4: [178, 110, 8, 252, 176, 121, 143, 76] };
pub const Tab_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 952966957, data2: 13178, data3: 19410, data4: [165, 227, 173, 180, 105, 227, 11, 211] };
pub const TableItem_ColumnHeaderItems_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2524599971, data2: 29878, data3: 17182, data4: [141, 230, 153, 196, 17, 3, 28, 88] };
pub const TableItem_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3742581693, data2: 6280, data3: 18985, data4: [165, 12, 185, 46, 109, 227, 127, 111] };
pub const TableItem_RowHeaderItems_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3019396000, data2: 1396, data3: 19672, data4: [188, 215, 237, 89, 35, 87, 45, 151] };
pub const Table_ColumnHeaders_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2951862059, data2: 38541, data3: 17073, data4: [180, 89, 21, 11, 41, 157, 166, 100] };
pub const Table_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2000419342, data2: 23492, data3: 19947, data4: [146, 27, 222, 123, 50, 6, 34, 158] };
pub const Table_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3289719182, data2: 41000, data3: 17950, data4: [170, 146, 143, 146, 92, 247, 147, 81] };
pub const Table_RowHeaders_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3655555975, data2: 28344, data3: 17762, data4: [170, 198, 168, 169, 7, 82, 54, 168] };
pub const Table_RowOrColumnMajor_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2210297283, data2: 10750, data3: 18992, data4: [133, 225, 42, 98, 119, 253, 16, 110] };
pub const TextChild_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1966328503, data2: 15358, data3: 16879, data4: [158, 133, 226, 99, 140, 190, 22, 158] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type TextDecorationLineStyle = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_None: TextDecorationLineStyle = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_Single: TextDecorationLineStyle = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_WordsOnly: TextDecorationLineStyle = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_Double: TextDecorationLineStyle = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_Dot: TextDecorationLineStyle = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_Dash: TextDecorationLineStyle = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_DashDot: TextDecorationLineStyle = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_DashDotDot: TextDecorationLineStyle = 7i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_Wavy: TextDecorationLineStyle = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickSingle: TextDecorationLineStyle = 9i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_DoubleWavy: TextDecorationLineStyle = 11i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickWavy: TextDecorationLineStyle = 12i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_LongDash: TextDecorationLineStyle = 13i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickDash: TextDecorationLineStyle = 14i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickDashDot: TextDecorationLineStyle = 15i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickDashDotDot: TextDecorationLineStyle = 16i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickDot: TextDecorationLineStyle = 17i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_ThickLongDash: TextDecorationLineStyle = 18i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextDecorationLineStyle_Other: TextDecorationLineStyle = -1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type TextEditChangeType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextEditChangeType_None: TextEditChangeType = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextEditChangeType_AutoCorrect: TextEditChangeType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextEditChangeType_Composition: TextEditChangeType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextEditChangeType_CompositionFinalized: TextEditChangeType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextEditChangeType_AutoComplete: TextEditChangeType = 4i32;
pub const TextEdit_ConversionTargetChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 864600451, data2: 60751, data3: 19595, data4: [155, 170, 54, 77, 81, 216, 132, 127] };
pub const TextEdit_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1777598345, data2: 23289, data3: 19573, data4: [147, 64, 242, 222, 41, 46, 69, 145] };
pub const TextEdit_TextChanged_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 302711560, data2: 60450, data3: 20152, data4: [156, 152, 152, 103, 205, 161, 177, 101] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type TextPatternRangeEndpoint = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextPatternRangeEndpoint_Start: TextPatternRangeEndpoint = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextPatternRangeEndpoint_End: TextPatternRangeEndpoint = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type TextUnit = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Character: TextUnit = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Format: TextUnit = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Word: TextUnit = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Line: TextUnit = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Paragraph: TextUnit = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Page: TextUnit = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TextUnit_Document: TextUnit = 6i32;
pub const Text_AfterParagraphSpacing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1485617976, data2: 58927, data3: 18812, data4: [181, 209, 204, 223, 14, 232, 35, 216] };
pub const Text_AfterSpacing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1485617976, data2: 58927, data3: 18812, data4: [181, 209, 204, 223, 14, 232, 35, 216] };
pub const Text_AnimationStyle_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1652689392, data2: 31898, data3: 19799, data4: [190, 100, 31, 24, 54, 87, 31, 245] };
pub const Text_AnnotationObjects_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4282503016, data2: 59307, data3: 16569, data4: [140, 114, 114, 168, 237, 148, 1, 125] };
pub const Text_AnnotationTypes_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2905519153, data2: 61006, data3: 19425, data4: [167, 186, 85, 89, 21, 90, 115, 239] };
pub const Text_BackgroundColor_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4257520135, data2: 22589, data3: 20247, data4: [173, 39, 119, 252, 131, 42, 60, 11] };
pub const Text_BeforeParagraphSpacing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3195734705, data2: 51234, data3: 18980, data4: [133, 233, 200, 242, 101, 15, 199, 156] };
pub const Text_BeforeSpacing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3195734705, data2: 51234, data3: 18980, data4: [133, 233, 200, 242, 101, 15, 199, 156] };
pub const Text_BulletStyle_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3238624400, data2: 54724, data3: 16951, data4: [151, 129, 59, 236, 139, 165, 78, 72] };
pub const Text_CapStyle_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4211448912, data2: 37580, data3: 18853, data4: [186, 143, 10, 168, 114, 187, 162, 243] };
pub const Text_CaretBidiMode_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2459887526, data2: 20947, data3: 18197, data4: [150, 220, 182, 148, 250, 36, 161, 104] };
pub const Text_CaretPosition_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2988945713, data2: 39049, data3: 18258, data4: [169, 27, 115, 62, 253, 197, 197, 160] };
pub const Text_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2929160924, data2: 54065, data3: 20233, data4: [190, 32, 126, 109, 250, 240, 123, 10] };
pub const Text_Culture_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3254934265, data2: 42029, data3: 19693, data4: [161, 251, 198, 116, 99, 21, 34, 46] };
pub const Text_FontName_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1692810152, data2: 62181, data3: 18286, data4: [164, 119, 23, 52, 254, 170, 247, 38] };
pub const Text_FontSize_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3697209087, data2: 1286, data3: 18035, data4: [147, 242, 55, 126, 74, 142, 1, 241] };
pub const Text_FontWeight_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1874862937, data2: 45846, data3: 20319, data4: [180, 1, 241, 206, 85, 116, 24, 83] };
pub const Text_ForegroundColor_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1926351197, data2: 24160, data3: 18202, data4: [150, 177, 108, 27, 59, 119, 164, 54] };
pub const Text_HorizontalTextAlignment_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 82469217, data2: 64419, data3: 18298, data4: [149, 42, 187, 50, 109, 2, 106, 91] };
pub const Text_IndentationFirstLine_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 544185045, data2: 49619, data3: 16970, data4: [129, 130, 109, 169, 167, 243, 214, 50] };
pub const Text_IndentationLeading_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1559653292, data2: 11589, data3: 19019, data4: [182, 201, 247, 34, 29, 40, 21, 176] };
pub const Text_IndentationTrailing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2550098959, data2: 7396, data3: 16522, data4: [182, 123, 148, 216, 62, 182, 155, 242] };
pub const Text_IsActive_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4121224499, data2: 57784, data3: 17259, data4: [147, 93, 181, 122, 163, 245, 88, 196] };
pub const Text_IsHidden_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 906068731, data2: 48599, data3: 18422, data4: [171, 105, 25, 227, 63, 138, 51, 68] };
pub const Text_IsItalic_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4242614870, data2: 4918, data3: 18996, data4: [150, 99, 27, 171, 71, 35, 147, 32] };
pub const Text_IsReadOnly_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2805470571, data2: 51774, data3: 18782, data4: [149, 20, 131, 60, 68, 15, 235, 17] };
pub const Text_IsSubscript_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4041922648, data2: 36691, data3: 16700, data4: [135, 63, 26, 125, 127, 94, 13, 228] };
pub const Text_IsSuperscript_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3664801508, data2: 45994, data3: 17989, data4: [164, 31, 205, 37, 21, 125, 234, 118] };
pub const Text_LineSpacing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1677684910, data2: 55619, data3: 19271, data4: [138, 183, 167, 160, 51, 211, 33, 75] };
pub const Text_Link_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3012490525, data2: 40589, data3: 20038, data4: [145, 68, 86, 235, 225, 119, 50, 155] };
pub const Text_MarginBottom_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2128974788, data2: 29364, data3: 19628, data4: [146, 113, 62, 210, 75, 14, 77, 66] };
pub const Text_MarginLeading_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2660385488, data2: 24272, data3: 18688, data4: [142, 138, 238, 204, 3, 131, 90, 252] };
pub const Text_MarginTop_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1748865903, data2: 51641, data3: 19098, data4: [179, 217, 210, 13, 51, 49, 30, 42] };
pub const Text_MarginTrailing_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2941398936, data2: 39325, data3: 16559, data4: [165, 178, 1, 105, 208, 52, 32, 2] };
pub const Text_OutlineStyles_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1533500199, data2: 56201, data3: 18174, data4: [151, 12, 97, 77, 82, 59, 185, 125] };
pub const Text_OverlineColor_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2209036346, data2: 64835, data3: 16602, data4: [171, 62, 236, 248, 22, 92, 187, 109] };
pub const Text_OverlineStyle_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 170085734, data2: 24958, data3: 17023, data4: [135, 29, 225, 255, 30, 12, 33, 63] };
pub const Text_Pattern2_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1233418658, data2: 23330, data3: 17549, data4: [182, 228, 100, 116, 144, 134, 6, 152] };
pub const Text_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2249584733, data2: 32229, data3: 17661, data4: [166, 121, 44, 164, 180, 96, 51, 168] };
pub const Text_SayAsInterpretAs_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3012220588, data2: 61153, data3: 19310, data4: [136, 204, 1, 76, 239, 169, 63, 203] };
pub const Text_SelectionActiveEnd_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 526814403, data2: 39871, data3: 16747, data4: [176, 162, 248, 159, 134, 246, 97, 44] };
pub const Text_StrikethroughColor_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3219216920, data2: 35905, data3: 19546, data4: [154, 11, 4, 175, 14, 7, 244, 135] };
pub const Text_StrikethroughStyle_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1922121457, data2: 55808, data3: 20225, data4: [137, 156, 172, 90, 133, 119, 163, 7] };
pub const Text_StyleId_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 348324062, data2: 49963, data3: 17563, data4: [171, 124, 176, 224, 120, 154, 234, 93] };
pub const Text_StyleName_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 583655569, data2: 19814, data3: 17880, data4: [168, 40, 115, 123, 171, 76, 152, 167] };
pub const Text_Tabs_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 778620939, data2: 37630, data3: 17112, data4: [137, 154, 167, 132, 170, 68, 84, 161] };
pub const Text_TextChangedEvent_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1244930178, data2: 62595, data3: 18628, data4: [172, 17, 168, 75, 67, 94, 42, 132] };
pub const Text_TextFlowDirections_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2346682169, data2: 62496, data3: 16958, data4: [175, 119, 32, 165, 217, 115, 169, 7] };
pub const Text_TextSelectionChangedEvent_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2442058401, data2: 29107, data3: 18862, data4: [151, 65, 121, 190, 184, 211, 88, 243] };
pub const Text_UnderlineColor_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3215010931, data2: 64994, data3: 17523, data4: [191, 100, 16, 54, 214, 170, 15, 69] };
pub const Text_UnderlineStyle_Attribute_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1597710784, data2: 60900, data3: 17597, data4: [156, 54, 56, 83, 3, 140, 191, 235] };
pub const Thumb_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1880926327, data2: 58128, data3: 19926, data4: [182, 68, 121, 126, 79, 174, 162, 19] };
pub const TitleBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2561299903, data2: 15280, data3: 19301, data4: [131, 110, 46, 163, 13, 188, 23, 31] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ToggleState = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ToggleState_Off: ToggleState = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ToggleState_On: ToggleState = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ToggleState_Indeterminate: ToggleState = 2i32;
pub const Toggle_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 188847968, data2: 58100, data3: 17407, data4: [140, 95, 148, 87, 200, 43, 86, 233] };
pub const Toggle_ToggleState_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2990333010, data2: 8898, data3: 19564, data4: [157, 237, 245, 196, 34, 71, 158, 222] };
pub const ToolBar_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2399582033, data2: 57730, data3: 20120, data4: [136, 147, 34, 132, 84, 58, 125, 206] };
pub const ToolTipClosed_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 661484015, data2: 9385, data3: 18870, data4: [142, 151, 218, 152, 180, 1, 187, 205] };
pub const ToolTipOpened_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1061918719, data2: 11996, data3: 17693, data4: [188, 164, 149, 163, 24, 141, 91, 3] };
pub const ToolTip_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 98420433, data2: 8503, data3: 18280, data4: [152, 234, 115, 245, 47, 113, 52, 243] };
pub const Tranform_Pattern2_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2331835655, data2: 41833, data3: 17630, data4: [152, 139, 47, 127, 244, 159, 184, 168] };
pub const Transform2_CanZoom_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4082624656, data2: 42838, data3: 17241, data4: [156, 166, 134, 112, 43, 248, 243, 129] };
pub const Transform2_ZoomLevel_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4007829274, data2: 62626, data3: 19291, data4: [172, 101, 149, 207, 147, 40, 51, 135] };
pub const Transform2_ZoomMaximum_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1118530423, data2: 52912, data3: 20170, data4: [184, 42, 108, 250, 95, 161, 252, 8] };
pub const Transform2_ZoomMinimum_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1949092886, data2: 19153, data3: 19975, data4: [150, 254, 177, 34, 198, 230, 178, 43] };
pub const Transform_CanMove_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 460685901, data2: 8331, data3: 20447, data4: [188, 205, 241, 244, 229, 116, 31, 79] };
pub const Transform_CanResize_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3147357349, data2: 19482, data3: 16852, data4: [164, 246, 235, 193, 40, 100, 65, 128] };
pub const Transform_CanRotate_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 268933960, data2: 14409, data3: 18287, data4: [172, 150, 68, 169, 92, 132, 64, 217] };
pub const Transform_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 615804891, data2: 22654, data3: 18929, data4: [156, 74, 216, 233, 139, 102, 75, 123] };
pub const TreeItem_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1657405113, data2: 36860, data3: 18552, data4: [163, 164, 150, 176, 48, 49, 92, 24] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type TreeScope = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_None: TreeScope = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_Element: TreeScope = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_Children: TreeScope = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_Descendants: TreeScope = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_Parent: TreeScope = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_Ancestors: TreeScope = 16i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeScope_Subtree: TreeScope = 7i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type TreeTraversalOptions = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeTraversalOptions_Default: TreeTraversalOptions = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeTraversalOptions_PostOrder: TreeTraversalOptions = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const TreeTraversalOptions_LastToFirstOrder: TreeTraversalOptions = 2i32;
pub const Tree_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1969304732, data2: 53825, data3: 17396, data4: [153, 8, 181, 240, 145, 190, 230, 17] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AcceleratorKeyPropertyId: i32 = 30006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AccessKeyPropertyId: i32 = 30007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ActiveTextPositionChangedEventId: i32 = 20036i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AfterParagraphSpacingAttributeId: i32 = 40042i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnimationStyleAttributeId: i32 = 40000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationAnnotationTypeIdPropertyId: i32 = 30113i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationAnnotationTypeNamePropertyId: i32 = 30114i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationAuthorPropertyId: i32 = 30115i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationDateTimePropertyId: i32 = 30116i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationObjectsAttributeId: i32 = 40032i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationObjectsPropertyId: i32 = 30156i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationPatternId: i32 = 10023i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationTargetPropertyId: i32 = 30117i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationTypesAttributeId: i32 = 40031i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AnnotationTypesPropertyId: i32 = 30155i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AppBarControlTypeId: i32 = 50040i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AriaPropertiesPropertyId: i32 = 30102i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AriaRolePropertyId: i32 = 30101i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AsyncContentLoadedEventId: i32 = 20006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AutomationFocusChangedEventId: i32 = 20005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AutomationIdPropertyId: i32 = 30011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_AutomationPropertyChangedEventId: i32 = 20004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_BackgroundColorAttributeId: i32 = 40001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_BeforeParagraphSpacingAttributeId: i32 = 40041i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_BoundingRectanglePropertyId: i32 = 30001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_BulletStyleAttributeId: i32 = 40002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ButtonControlTypeId: i32 = 50000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CalendarControlTypeId: i32 = 50001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CapStyleAttributeId: i32 = 40003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CaretBidiModeAttributeId: i32 = 40039i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CaretPositionAttributeId: i32 = 40038i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CenterPointPropertyId: i32 = 30165i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ChangesEventId: i32 = 20034i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CheckBoxControlTypeId: i32 = 50002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ClassNamePropertyId: i32 = 30012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ClickablePointPropertyId: i32 = 30014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ComboBoxControlTypeId: i32 = 50003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ControlTypePropertyId: i32 = 30003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ControllerForPropertyId: i32 = 30104i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CultureAttributeId: i32 = 40004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CulturePropertyId: i32 = 30015i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CustomControlTypeId: i32 = 50025i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CustomLandmarkTypeId: i32 = 80000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_CustomNavigationPatternId: i32 = 10033i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DataGridControlTypeId: i32 = 50028i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DataItemControlTypeId: i32 = 50029i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DescribedByPropertyId: i32 = 30105i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DockDockPositionPropertyId: i32 = 30069i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DockPatternId: i32 = 10011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DocumentControlTypeId: i32 = 50030i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DragDropEffectPropertyId: i32 = 30139i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DragDropEffectsPropertyId: i32 = 30140i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DragGrabbedItemsPropertyId: i32 = 30144i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DragIsGrabbedPropertyId: i32 = 30138i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DragPatternId: i32 = 10030i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Drag_DragCancelEventId: i32 = 20027i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Drag_DragCompleteEventId: i32 = 20028i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Drag_DragStartEventId: i32 = 20026i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DropTargetDropTargetEffectPropertyId: i32 = 30142i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DropTargetDropTargetEffectsPropertyId: i32 = 30143i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DropTargetPatternId: i32 = 10031i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DropTarget_DragEnterEventId: i32 = 20029i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DropTarget_DragLeaveEventId: i32 = 20030i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_DropTarget_DroppedEventId: i32 = 20031i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_ELEMENTNOTAVAILABLE: u32 = 2147746305u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_ELEMENTNOTENABLED: u32 = 2147746304u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_INVALIDOPERATION: u32 = 2148734217u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_NOCLICKABLEPOINT: u32 = 2147746306u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_NOTSUPPORTED: u32 = 2147746308u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_PROXYASSEMBLYNOTLOADED: u32 = 2147746307u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_E_TIMEOUT: u32 = 2148734213u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_EditControlTypeId: i32 = 50004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ExpandCollapseExpandCollapseStatePropertyId: i32 = 30070i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ExpandCollapsePatternId: i32 = 10005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FillColorPropertyId: i32 = 30160i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FillTypePropertyId: i32 = 30162i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FlowsFromPropertyId: i32 = 30148i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FlowsToPropertyId: i32 = 30106i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FontNameAttributeId: i32 = 40005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FontSizeAttributeId: i32 = 40006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FontWeightAttributeId: i32 = 40007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ForegroundColorAttributeId: i32 = 40008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FormLandmarkTypeId: i32 = 80001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FrameworkIdPropertyId: i32 = 30024i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_FullDescriptionPropertyId: i32 = 30159i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridColumnCountPropertyId: i32 = 30063i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridItemColumnPropertyId: i32 = 30065i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridItemColumnSpanPropertyId: i32 = 30067i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridItemContainingGridPropertyId: i32 = 30068i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridItemPatternId: i32 = 10007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridItemRowPropertyId: i32 = 30064i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridItemRowSpanPropertyId: i32 = 30066i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridPatternId: i32 = 10006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GridRowCountPropertyId: i32 = 30062i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_GroupControlTypeId: i32 = 50026i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HasKeyboardFocusPropertyId: i32 = 30008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HeaderControlTypeId: i32 = 50034i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HeaderItemControlTypeId: i32 = 50035i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HeadingLevelPropertyId: i32 = 30173i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HelpTextPropertyId: i32 = 30013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HorizontalTextAlignmentAttributeId: i32 = 40009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HostedFragmentRootsInvalidatedEventId: i32 = 20025i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_HyperlinkControlTypeId: i32 = 50005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IAFP_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IAFP_UNWRAP_BRIDGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ImageControlTypeId: i32 = 50006i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IndentationFirstLineAttributeId: i32 = 40010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IndentationLeadingAttributeId: i32 = 40011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IndentationTrailingAttributeId: i32 = 40012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_InputDiscardedEventId: i32 = 20022i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_InputReachedOtherElementEventId: i32 = 20021i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_InputReachedTargetEventId: i32 = 20020i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_InvokePatternId: i32 = 10000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Invoke_InvokedEventId: i32 = 20009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsActiveAttributeId: i32 = 40036i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsAnnotationPatternAvailablePropertyId: i32 = 30118i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsContentElementPropertyId: i32 = 30017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsControlElementPropertyId: i32 = 30016i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsCustomNavigationPatternAvailablePropertyId: i32 = 30151i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsDataValidForFormPropertyId: i32 = 30103i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsDialogPropertyId: i32 = 30174i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsDockPatternAvailablePropertyId: i32 = 30027i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsDragPatternAvailablePropertyId: i32 = 30137i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsDropTargetPatternAvailablePropertyId: i32 = 30141i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsEnabledPropertyId: i32 = 30010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsExpandCollapsePatternAvailablePropertyId: i32 = 30028i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsGridItemPatternAvailablePropertyId: i32 = 30029i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsGridPatternAvailablePropertyId: i32 = 30030i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsHiddenAttributeId: i32 = 40013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsInvokePatternAvailablePropertyId: i32 = 30031i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsItalicAttributeId: i32 = 40014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsItemContainerPatternAvailablePropertyId: i32 = 30108i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsKeyboardFocusablePropertyId: i32 = 30009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsLegacyIAccessiblePatternAvailablePropertyId: i32 = 30090i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsMultipleViewPatternAvailablePropertyId: i32 = 30032i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsObjectModelPatternAvailablePropertyId: i32 = 30112i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsOffscreenPropertyId: i32 = 30022i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsPasswordPropertyId: i32 = 30019i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsPeripheralPropertyId: i32 = 30150i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsRangeValuePatternAvailablePropertyId: i32 = 30033i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsReadOnlyAttributeId: i32 = 40015i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsRequiredForFormPropertyId: i32 = 30025i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsScrollItemPatternAvailablePropertyId: i32 = 30035i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsScrollPatternAvailablePropertyId: i32 = 30034i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSelectionItemPatternAvailablePropertyId: i32 = 30036i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSelectionPattern2AvailablePropertyId: i32 = 30168i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSelectionPatternAvailablePropertyId: i32 = 30037i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSpreadsheetItemPatternAvailablePropertyId: i32 = 30132i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSpreadsheetPatternAvailablePropertyId: i32 = 30128i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsStylesPatternAvailablePropertyId: i32 = 30127i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSubscriptAttributeId: i32 = 40016i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSuperscriptAttributeId: i32 = 40017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsSynchronizedInputPatternAvailablePropertyId: i32 = 30110i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTableItemPatternAvailablePropertyId: i32 = 30039i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTablePatternAvailablePropertyId: i32 = 30038i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTextChildPatternAvailablePropertyId: i32 = 30136i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTextEditPatternAvailablePropertyId: i32 = 30149i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTextPattern2AvailablePropertyId: i32 = 30119i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTextPatternAvailablePropertyId: i32 = 30040i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTogglePatternAvailablePropertyId: i32 = 30041i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTransformPattern2AvailablePropertyId: i32 = 30134i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsTransformPatternAvailablePropertyId: i32 = 30042i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsValuePatternAvailablePropertyId: i32 = 30043i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsVirtualizedItemPatternAvailablePropertyId: i32 = 30109i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_IsWindowPatternAvailablePropertyId: i32 = 30044i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ItemContainerPatternId: i32 = 10019i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ItemStatusPropertyId: i32 = 30026i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ItemTypePropertyId: i32 = 30021i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LabeledByPropertyId: i32 = 30018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LandmarkTypePropertyId: i32 = 30157i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LayoutInvalidatedEventId: i32 = 20008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleChildIdPropertyId: i32 = 30091i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleDefaultActionPropertyId: i32 = 30100i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleDescriptionPropertyId: i32 = 30094i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleHelpPropertyId: i32 = 30097i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleKeyboardShortcutPropertyId: i32 = 30098i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleNamePropertyId: i32 = 30092i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessiblePatternId: i32 = 10018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleRolePropertyId: i32 = 30095i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleSelectionPropertyId: i32 = 30099i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleStatePropertyId: i32 = 30096i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LegacyIAccessibleValuePropertyId: i32 = 30093i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LevelPropertyId: i32 = 30154i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LineSpacingAttributeId: i32 = 40040i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LinkAttributeId: i32 = 40035i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ListControlTypeId: i32 = 50008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ListItemControlTypeId: i32 = 50007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LiveRegionChangedEventId: i32 = 20024i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LiveSettingPropertyId: i32 = 30135i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LocalizedControlTypePropertyId: i32 = 30004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_LocalizedLandmarkTypePropertyId: i32 = 30158i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MainLandmarkTypeId: i32 = 80002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MarginBottomAttributeId: i32 = 40018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MarginLeadingAttributeId: i32 = 40019i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MarginTopAttributeId: i32 = 40020i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MarginTrailingAttributeId: i32 = 40021i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuBarControlTypeId: i32 = 50010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuClosedEventId: i32 = 20007i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuControlTypeId: i32 = 50009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuItemControlTypeId: i32 = 50011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuModeEndEventId: i32 = 20019i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuModeStartEventId: i32 = 20018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MenuOpenedEventId: i32 = 20003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MultipleViewCurrentViewPropertyId: i32 = 30071i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MultipleViewPatternId: i32 = 10008i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_MultipleViewSupportedViewsPropertyId: i32 = 30072i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_NamePropertyId: i32 = 30005i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_NativeWindowHandlePropertyId: i32 = 30020i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_NavigationLandmarkTypeId: i32 = 80003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_NotificationEventId: i32 = 20035i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ObjectModelPatternId: i32 = 10022i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OptimizeForVisualContentPropertyId: i32 = 30111i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OrientationPropertyId: i32 = 30023i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OutlineColorPropertyId: i32 = 30161i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OutlineStylesAttributeId: i32 = 40022i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OutlineThicknessPropertyId: i32 = 30164i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OverlineColorAttributeId: i32 = 40023i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_OverlineStyleAttributeId: i32 = 40024i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_PFIA_DEFAULT: u32 = 0u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_PFIA_UNWRAP_BRIDGE: u32 = 1u32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_PaneControlTypeId: i32 = 50033i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_PositionInSetPropertyId: i32 = 30152i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ProcessIdPropertyId: i32 = 30002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ProgressBarControlTypeId: i32 = 50012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ProviderDescriptionPropertyId: i32 = 30107i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RadioButtonControlTypeId: i32 = 50013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValueIsReadOnlyPropertyId: i32 = 30048i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValueLargeChangePropertyId: i32 = 30051i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValueMaximumPropertyId: i32 = 30050i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValueMinimumPropertyId: i32 = 30049i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValuePatternId: i32 = 10003i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValueSmallChangePropertyId: i32 = 30052i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RangeValueValuePropertyId: i32 = 30047i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RotationPropertyId: i32 = 30166i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_RuntimeIdPropertyId: i32 = 30000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SayAsInterpretAsAttributeId: i32 = 40043i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SayAsInterpretAsMetadataId: i32 = 100000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollBarControlTypeId: i32 = 50014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollHorizontalScrollPercentPropertyId: i32 = 30053i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollHorizontalViewSizePropertyId: i32 = 30054i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollHorizontallyScrollablePropertyId: i32 = 30057i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollItemPatternId: i32 = 10017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollPatternId: i32 = 10004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollPatternNoScroll: f64 = -1f64;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollVerticalScrollPercentPropertyId: i32 = 30055i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollVerticalViewSizePropertyId: i32 = 30056i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ScrollVerticallyScrollablePropertyId: i32 = 30058i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SearchLandmarkTypeId: i32 = 80004i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Selection2CurrentSelectedItemPropertyId: i32 = 30171i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Selection2FirstSelectedItemPropertyId: i32 = 30169i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Selection2ItemCountPropertyId: i32 = 30172i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Selection2LastSelectedItemPropertyId: i32 = 30170i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionActiveEndAttributeId: i32 = 40037i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionCanSelectMultiplePropertyId: i32 = 30060i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionIsSelectionRequiredPropertyId: i32 = 30061i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionItemIsSelectedPropertyId: i32 = 30079i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionItemPatternId: i32 = 10010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionItemSelectionContainerPropertyId: i32 = 30080i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionItem_ElementAddedToSelectionEventId: i32 = 20010i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionItem_ElementRemovedFromSelectionEventId: i32 = 20011i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionItem_ElementSelectedEventId: i32 = 20012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionPattern2Id: i32 = 10034i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionPatternId: i32 = 10001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SelectionSelectionPropertyId: i32 = 30059i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Selection_InvalidatedEventId: i32 = 20013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SemanticZoomControlTypeId: i32 = 50039i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SeparatorControlTypeId: i32 = 50038i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SizeOfSetPropertyId: i32 = 30153i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SizePropertyId: i32 = 30167i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SliderControlTypeId: i32 = 50015i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SpinnerControlTypeId: i32 = 50016i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SplitButtonControlTypeId: i32 = 50031i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SpreadsheetItemAnnotationObjectsPropertyId: i32 = 30130i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SpreadsheetItemAnnotationTypesPropertyId: i32 = 30131i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SpreadsheetItemFormulaPropertyId: i32 = 30129i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SpreadsheetItemPatternId: i32 = 10027i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SpreadsheetPatternId: i32 = 10026i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StatusBarControlTypeId: i32 = 50017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StrikethroughColorAttributeId: i32 = 40025i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StrikethroughStyleAttributeId: i32 = 40026i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StructureChangedEventId: i32 = 20002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StyleIdAttributeId: i32 = 40034i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StyleNameAttributeId: i32 = 40033i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesExtendedPropertiesPropertyId: i32 = 30126i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesFillColorPropertyId: i32 = 30122i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesFillPatternColorPropertyId: i32 = 30125i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesFillPatternStylePropertyId: i32 = 30123i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesPatternId: i32 = 10025i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesShapePropertyId: i32 = 30124i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesStyleIdPropertyId: i32 = 30120i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_StylesStyleNamePropertyId: i32 = 30121i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SummaryChangeId: i32 = 90000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SynchronizedInputPatternId: i32 = 10021i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_SystemAlertEventId: i32 = 20023i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TabControlTypeId: i32 = 50018i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TabItemControlTypeId: i32 = 50019i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableColumnHeadersPropertyId: i32 = 30082i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableControlTypeId: i32 = 50036i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableItemColumnHeaderItemsPropertyId: i32 = 30085i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableItemPatternId: i32 = 10013i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableItemRowHeaderItemsPropertyId: i32 = 30084i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TablePatternId: i32 = 10012i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableRowHeadersPropertyId: i32 = 30081i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TableRowOrColumnMajorPropertyId: i32 = 30083i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TabsAttributeId: i32 = 40027i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextChildPatternId: i32 = 10029i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextControlTypeId: i32 = 50020i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextEditPatternId: i32 = 10032i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextEdit_ConversionTargetChangedEventId: i32 = 20033i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextEdit_TextChangedEventId: i32 = 20032i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextFlowDirectionsAttributeId: i32 = 40028i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextPattern2Id: i32 = 10024i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TextPatternId: i32 = 10014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Text_TextChangedEventId: i32 = 20015i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Text_TextSelectionChangedEventId: i32 = 20014i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ThumbControlTypeId: i32 = 50027i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TitleBarControlTypeId: i32 = 50037i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TogglePatternId: i32 = 10015i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ToggleToggleStatePropertyId: i32 = 30086i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ToolBarControlTypeId: i32 = 50021i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ToolTipClosedEventId: i32 = 20001i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ToolTipControlTypeId: i32 = 50022i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ToolTipOpenedEventId: i32 = 20000i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Transform2CanZoomPropertyId: i32 = 30133i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Transform2ZoomLevelPropertyId: i32 = 30145i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Transform2ZoomMaximumPropertyId: i32 = 30147i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Transform2ZoomMinimumPropertyId: i32 = 30146i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TransformCanMovePropertyId: i32 = 30087i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TransformCanResizePropertyId: i32 = 30088i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TransformCanRotatePropertyId: i32 = 30089i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TransformPattern2Id: i32 = 10028i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TransformPatternId: i32 = 10016i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TreeControlTypeId: i32 = 50023i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_TreeItemControlTypeId: i32 = 50024i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_UnderlineColorAttributeId: i32 = 40029i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_UnderlineStyleAttributeId: i32 = 40030i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ValueIsReadOnlyPropertyId: i32 = 30046i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ValuePatternId: i32 = 10002i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_ValueValuePropertyId: i32 = 30045i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_VirtualizedItemPatternId: i32 = 10020i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_VisualEffectsPropertyId: i32 = 30163i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowCanMaximizePropertyId: i32 = 30073i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowCanMinimizePropertyId: i32 = 30074i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowControlTypeId: i32 = 50032i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowIsModalPropertyId: i32 = 30077i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowIsTopmostPropertyId: i32 = 30078i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowPatternId: i32 = 10009i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowWindowInteractionStatePropertyId: i32 = 30076i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_WindowWindowVisualStatePropertyId: i32 = 30075i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Window_WindowClosedEventId: i32 = 20017i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIA_Window_WindowOpenedEventId: i32 = 20016i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UIAutomationEventInfo {
    pub guid: ::windows_sys::core::GUID,
    pub pProgrammaticName: ::windows_sys::core::PCWSTR,
}
impl ::core::marker::Copy for UIAutomationEventInfo {}
impl ::core::clone::Clone for UIAutomationEventInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct UIAutomationMethodInfo {
    pub pProgrammaticName: ::windows_sys::core::PCWSTR,
    pub doSetFocus: super::super::Foundation::BOOL,
    pub cInParameters: u32,
    pub cOutParameters: u32,
    pub pParameterTypes: *mut UIAutomationType,
    pub pParameterNames: *mut ::windows_sys::core::PWSTR,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for UIAutomationMethodInfo {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for UIAutomationMethodInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UIAutomationParameter {
    pub r#type: UIAutomationType,
    pub pData: *mut ::core::ffi::c_void,
}
impl ::core::marker::Copy for UIAutomationParameter {}
impl ::core::clone::Clone for UIAutomationParameter {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct UIAutomationPatternInfo {
    pub guid: ::windows_sys::core::GUID,
    pub pProgrammaticName: ::windows_sys::core::PCWSTR,
    pub providerInterfaceId: ::windows_sys::core::GUID,
    pub clientInterfaceId: ::windows_sys::core::GUID,
    pub cProperties: u32,
    pub pProperties: *mut UIAutomationPropertyInfo,
    pub cMethods: u32,
    pub pMethods: *mut UIAutomationMethodInfo,
    pub cEvents: u32,
    pub pEvents: *mut UIAutomationEventInfo,
    pub pPatternHandler: IUIAutomationPatternHandler,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for UIAutomationPatternInfo {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for UIAutomationPatternInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UIAutomationPropertyInfo {
    pub guid: ::windows_sys::core::GUID,
    pub pProgrammaticName: ::windows_sys::core::PCWSTR,
    pub r#type: UIAutomationType,
}
impl ::core::marker::Copy for UIAutomationPropertyInfo {}
impl ::core::clone::Clone for UIAutomationPropertyInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type UIAutomationType = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Int: UIAutomationType = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Bool: UIAutomationType = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_String: UIAutomationType = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Double: UIAutomationType = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Point: UIAutomationType = 5i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Rect: UIAutomationType = 6i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Element: UIAutomationType = 7i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Array: UIAutomationType = 65536i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_Out: UIAutomationType = 131072i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_IntArray: UIAutomationType = 65537i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_BoolArray: UIAutomationType = 65538i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_StringArray: UIAutomationType = 65539i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_DoubleArray: UIAutomationType = 65540i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_PointArray: UIAutomationType = 65541i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_RectArray: UIAutomationType = 65542i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_ElementArray: UIAutomationType = 65543i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutInt: UIAutomationType = 131073i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutBool: UIAutomationType = 131074i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutString: UIAutomationType = 131075i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutDouble: UIAutomationType = 131076i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutPoint: UIAutomationType = 131077i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutRect: UIAutomationType = 131078i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutElement: UIAutomationType = 131079i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutIntArray: UIAutomationType = 196609i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutBoolArray: UIAutomationType = 196610i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutStringArray: UIAutomationType = 196611i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutDoubleArray: UIAutomationType = 196612i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutPointArray: UIAutomationType = 196613i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutRectArray: UIAutomationType = 196614i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UIAutomationType_OutElementArray: UIAutomationType = 196615i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaAndOrCondition {
    pub ConditionType: ConditionType,
    pub ppConditions: *mut *mut UiaCondition,
    pub cConditions: i32,
}
impl ::core::marker::Copy for UiaAndOrCondition {}
impl ::core::clone::Clone for UiaAndOrCondition {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UiaAppendRuntimeId: u32 = 3u32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaAsyncContentLoadedEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
    pub AsyncContentLoadedState: AsyncContentLoadedState,
    pub PercentComplete: f64,
}
impl ::core::marker::Copy for UiaAsyncContentLoadedEventArgs {}
impl ::core::clone::Clone for UiaAsyncContentLoadedEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaCacheRequest {
    pub pViewCondition: *mut UiaCondition,
    pub Scope: TreeScope,
    pub pProperties: *mut i32,
    pub cProperties: i32,
    pub pPatterns: *mut i32,
    pub cPatterns: i32,
    pub automationElementMode: AutomationElementMode,
}
impl ::core::marker::Copy for UiaCacheRequest {}
impl ::core::clone::Clone for UiaCacheRequest {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub struct UiaChangeInfo {
    pub uiaId: i32,
    pub payload: super::super::System::Com::VARIANT,
    pub extraInfo: super::super::System::Com::VARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for UiaChangeInfo {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for UiaChangeInfo {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub struct UiaChangesEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
    pub EventIdCount: i32,
    pub pUiaChanges: *mut UiaChangeInfo,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for UiaChangesEventArgs {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for UiaChangesEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaCondition {
    pub ConditionType: ConditionType,
}
impl ::core::marker::Copy for UiaCondition {}
impl ::core::clone::Clone for UiaCondition {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
}
impl ::core::marker::Copy for UiaEventArgs {}
impl ::core::clone::Clone for UiaEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
pub type UiaEventCallback = ::core::option::Option<unsafe extern "system" fn(pargs: *mut UiaEventArgs, prequesteddata: *mut super::super::System::Com::SAFEARRAY, ptreestructure: super::super::Foundation::BSTR)>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct UiaFindParams {
    pub MaxDepth: i32,
    pub FindFirst: super::super::Foundation::BOOL,
    pub ExcludeRoot: super::super::Foundation::BOOL,
    pub pFindCondition: *mut UiaCondition,
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for UiaFindParams {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for UiaFindParams {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaNotCondition {
    pub ConditionType: ConditionType,
    pub pCondition: *mut UiaCondition,
}
impl ::core::marker::Copy for UiaNotCondition {}
impl ::core::clone::Clone for UiaNotCondition {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaPoint {
    pub x: f64,
    pub y: f64,
}
impl ::core::marker::Copy for UiaPoint {}
impl ::core::clone::Clone for UiaPoint {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub struct UiaPropertyChangedEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
    pub PropertyId: i32,
    pub OldValue: super::super::System::Com::VARIANT,
    pub NewValue: super::super::System::Com::VARIANT,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for UiaPropertyChangedEventArgs {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for UiaPropertyChangedEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
pub struct UiaPropertyCondition {
    pub ConditionType: ConditionType,
    pub PropertyId: i32,
    pub Value: super::super::System::Com::VARIANT,
    pub Flags: PropertyConditionFlags,
}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::marker::Copy for UiaPropertyCondition {}
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
impl ::core::clone::Clone for UiaPropertyCondition {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`*"]
#[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com"))]
pub type UiaProviderCallback = ::core::option::Option<unsafe extern "system" fn(hwnd: super::super::Foundation::HWND, providertype: ProviderType) -> *mut super::super::System::Com::SAFEARRAY>;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaRect {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}
impl ::core::marker::Copy for UiaRect {}
impl ::core::clone::Clone for UiaRect {
    fn clone(&self) -> Self {
        *self
    }
}
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const UiaRootObjectId: i32 = -25i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaStructureChangedEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
    pub StructureChangeType: StructureChangeType,
    pub pRuntimeId: *mut i32,
    pub cRuntimeIdLen: i32,
}
impl ::core::marker::Copy for UiaStructureChangedEventArgs {}
impl ::core::clone::Clone for UiaStructureChangedEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_System_Com\"`*"]
#[cfg(feature = "Win32_System_Com")]
pub struct UiaTextEditTextChangedEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
    pub TextEditChangeType: TextEditChangeType,
    pub pTextChange: *mut super::super::System::Com::SAFEARRAY,
}
#[cfg(feature = "Win32_System_Com")]
impl ::core::marker::Copy for UiaTextEditTextChangedEventArgs {}
#[cfg(feature = "Win32_System_Com")]
impl ::core::clone::Clone for UiaTextEditTextChangedEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub struct UiaWindowClosedEventArgs {
    pub Type: EventArgsType,
    pub EventId: i32,
    pub pRuntimeId: *mut i32,
    pub cRuntimeIdLen: i32,
}
impl ::core::marker::Copy for UiaWindowClosedEventArgs {}
impl ::core::clone::Clone for UiaWindowClosedEventArgs {
    fn clone(&self) -> Self {
        *self
    }
}
pub const Value_IsReadOnly_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3943239472, data2: 57932, data3: 18329, data4: [167, 5, 13, 36, 123, 192, 55, 248] };
pub const Value_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 402304414, data2: 51319, data3: 18267, data4: [185, 51, 119, 51, 39, 121, 182, 55] };
pub const Value_Value_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3915341412, data2: 9887, data3: 19077, data4: [186, 153, 64, 146, 195, 234, 41, 134] };
pub const VirtualizedItem_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4111472446, data2: 11889, data3: 17897, data4: [166, 229, 98, 246, 237, 130, 137, 213] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type VisualEffects = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const VisualEffects_None: VisualEffects = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const VisualEffects_Shadow: VisualEffects = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const VisualEffects_Reflection: VisualEffects = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const VisualEffects_Glow: VisualEffects = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const VisualEffects_SoftEdges: VisualEffects = 8i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const VisualEffects_Bevel: VisualEffects = 16i32;
pub const VisualEffects_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3860497765, data2: 43737, data3: 18135, data4: [158, 112, 78, 138, 132, 32, 212, 32] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub type WINEVENTPROC = ::core::option::Option<unsafe extern "system" fn(hwineventhook: HWINEVENTHOOK, event: u32, hwnd: super::super::Foundation::HWND, idobject: i32, idchild: i32, ideventthread: u32, dwmseventtime: u32)>;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type WindowInteractionState = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowInteractionState_Running: WindowInteractionState = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowInteractionState_Closing: WindowInteractionState = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowInteractionState_ReadyForUserInteraction: WindowInteractionState = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowInteractionState_BlockedByModalWindow: WindowInteractionState = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowInteractionState_NotResponding: WindowInteractionState = 4i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type WindowVisualState = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowVisualState_Normal: WindowVisualState = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowVisualState_Maximized: WindowVisualState = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const WindowVisualState_Minimized: WindowVisualState = 2i32;
pub const Window_CanMaximize_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1694496063, data2: 25437, data3: 16833, data4: [149, 12, 203, 90, 223, 190, 40, 227] };
pub const Window_CanMinimize_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3074115109, data2: 22920, data3: 19351, data4: [180, 194, 166, 254, 110, 120, 200, 198] };
pub const Window_Control_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3778703938, data2: 62562, data3: 20301, data4: [174, 193, 83, 178, 141, 108, 50, 144] };
pub const Window_IsModal_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4283328658, data2: 14265, data3: 20426, data4: [133, 50, 255, 230, 116, 236, 254, 237] };
pub const Window_IsTopmost_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 4017980883, data2: 2359, data3: 18786, data4: [146, 65, 182, 35, 69, 242, 64, 65] };
pub const Window_Pattern_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 663754549, data2: 51040, data3: 18836, data4: [173, 17, 89, 25, 230, 6, 177, 16] };
pub const Window_WindowClosed_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3992011256, data2: 64103, data3: 20002, data4: [187, 247, 148, 78, 5, 115, 94, 226] };
pub const Window_WindowInteractionState_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1340941988, data2: 1109, data3: 20386, data4: [178, 28, 196, 218, 45, 177, 255, 156] };
pub const Window_WindowOpened_Event_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3555204358, data2: 56901, data3: 20271, data4: [150, 51, 222, 158, 2, 251, 101, 175] };
pub const Window_WindowVisualState_Property_GUID: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 1253544031, data2: 59488, data3: 17726, data4: [163, 10, 246, 67, 30, 93, 170, 213] };
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub type ZoomUnit = i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ZoomUnit_NoAmount: ZoomUnit = 0i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ZoomUnit_LargeDecrement: ZoomUnit = 1i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ZoomUnit_SmallDecrement: ZoomUnit = 2i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ZoomUnit_LargeIncrement: ZoomUnit = 3i32;
#[doc = "*Required features: `\"Win32_UI_Accessibility\"`*"]
pub const ZoomUnit_SmallIncrement: ZoomUnit = 4i32;
