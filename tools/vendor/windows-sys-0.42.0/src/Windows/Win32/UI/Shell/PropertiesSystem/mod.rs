#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn ClearPropVariantArray(rgpropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT, cvars: u32);
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn ClearVariantArray(pvars: *mut super::super::super::System::Com::VARIANT, cvars: u32);
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromBooleanVector(prgf: *const super::super::super::Foundation::BOOL, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromBuffer(pv: *const ::core::ffi::c_void, cb: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromCLSID(clsid: *const ::windows_sys::core::GUID, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromDoubleVector(prgn: *const f64, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromFileTime(pftin: *const super::super::super::Foundation::FILETIME, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromFileTimeVector(prgft: *const super::super::super::Foundation::FILETIME, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromGUIDAsString(guid: *const ::windows_sys::core::GUID, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromInt16Vector(prgn: *const i16, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromInt32Vector(prgn: *const i32, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromInt64Vector(prgn: *const i64, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromPropVariantVectorElem(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromResource(hinst: super::super::super::Foundation::HINSTANCE, id: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_Common"))]
    pub fn InitPropVariantFromStrRet(pstrret: *mut super::Common::STRRET, pidl: *const super::Common::ITEMIDLIST, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromStringAsVector(psz: ::windows_sys::core::PCWSTR, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromStringVector(prgsz: *const ::windows_sys::core::PWSTR, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromUInt16Vector(prgn: *const u16, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromUInt32Vector(prgn: *const u32, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantFromUInt64Vector(prgn: *const u64, celems: u32, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn InitPropVariantVectorFromPropVariant(propvarsingle: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ppropvarvector: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromBooleanArray(prgf: *const super::super::super::Foundation::BOOL, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromBuffer(pv: *const ::core::ffi::c_void, cb: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromDoubleArray(prgn: *const f64, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromFileTime(pft: *const super::super::super::Foundation::FILETIME, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromFileTimeArray(prgft: *const super::super::super::Foundation::FILETIME, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromGUIDAsString(guid: *const ::windows_sys::core::GUID, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromInt16Array(prgn: *const i16, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromInt32Array(prgn: *const i32, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromInt64Array(prgn: *const i64, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromResource(hinst: super::super::super::Foundation::HINSTANCE, id: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
    pub fn InitVariantFromStrRet(pstrret: *const super::Common::STRRET, pidl: *const super::Common::ITEMIDLIST, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromStringArray(prgsz: *const ::windows_sys::core::PWSTR, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromUInt16Array(prgn: *const u16, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromUInt32Array(prgn: *const u32, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromUInt64Array(prgn: *const u64, celems: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn InitVariantFromVariantArrayElem(varin: *const super::super::super::System::Com::VARIANT, ielem: u32, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSCoerceToCanonicalValue(key: *const PROPERTYKEY, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSCreateAdapterFromPropertyStore(pps: IPropertyStore, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSCreateDelayedMultiplexPropertyStore(flags: GETPROPERTYSTOREFLAGS, pdpsf: IDelayedPropertyStoreFactory, rgstoreids: *const u32, cstores: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSCreateMemoryPropertyStore(riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSCreateMultiplexPropertyStore(prgpunkstores: *const ::windows_sys::core::IUnknown, cstores: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSCreatePropertyChangeArray(rgpropkey: *const PROPERTYKEY, rgflags: *const PKA_FLAGS, rgpropvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, cchanges: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSCreatePropertyStoreFromObject(punk: ::windows_sys::core::IUnknown, grfmode: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSCreatePropertyStoreFromPropertySetStorage(ppss: super::super::super::System::Com::StructuredStorage::IPropertySetStorage, grfmode: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSCreateSimplePropertyChange(flags: PKA_FLAGS, key: *const PROPERTYKEY, propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSEnumeratePropertyDescriptions(filteron: PROPDESC_ENUMFILTER, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSFormatForDisplay(propkey: *const PROPERTYKEY, propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pdfflags: PROPDESC_FORMAT_FLAGS, pwsztext: ::windows_sys::core::PWSTR, cchtext: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSFormatForDisplayAlloc(key: *const PROPERTYKEY, propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pdff: PROPDESC_FORMAT_FLAGS, ppszdisplay: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSFormatPropertyValue(pps: IPropertyStore, ppd: IPropertyDescription, pdff: PROPDESC_FORMAT_FLAGS, ppszdisplay: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSGetImageReferenceForValue(propkey: *const PROPERTYKEY, propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ppszimageres: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PSGetItemPropertyHandler(punkitem: ::windows_sys::core::IUnknown, freadwrite: super::super::super::Foundation::BOOL, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PSGetItemPropertyHandlerWithCreateObject(punkitem: ::windows_sys::core::IUnknown, freadwrite: super::super::super::Foundation::BOOL, punkcreateobject: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSGetNameFromPropertyKey(propkey: *const PROPERTYKEY, ppszcanonicalname: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSGetNamedPropertyFromPropertyStorage(psps: *const SERIALIZEDPROPSTORAGE, cb: u32, pszname: ::windows_sys::core::PCWSTR, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSGetPropertyDescription(propkey: *const PROPERTYKEY, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSGetPropertyDescriptionByName(pszcanonicalname: ::windows_sys::core::PCWSTR, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSGetPropertyDescriptionListFromString(pszproplist: ::windows_sys::core::PCWSTR, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSGetPropertyFromPropertyStorage(psps: *const SERIALIZEDPROPSTORAGE, cb: u32, rpkey: *const PROPERTYKEY, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSGetPropertyKeyFromName(pszname: ::windows_sys::core::PCWSTR, ppropkey: *mut PROPERTYKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSGetPropertySystem(riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSGetPropertyValue(pps: IPropertyStore, ppd: IPropertyDescription, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSLookupPropertyHandlerCLSID(pszfilepath: ::windows_sys::core::PCWSTR, pclsid: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_Delete(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_ReadBOOL(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadBSTR(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut ::windows_sys::core::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadDWORD(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadGUID(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadInt(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadLONG(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_ReadPOINTL(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut super::super::super::Foundation::POINTL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_ReadPOINTS(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut super::super::super::Foundation::POINTS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadPropertyKey(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut PROPERTYKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_ReadRECTL(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut super::super::super::Foundation::RECTL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadSHORT(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut i16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadStr(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: ::windows_sys::core::PWSTR, charactercount: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadStrAlloc(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadStream(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut super::super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_System_Ole"))]
    pub fn PSPropertyBag_ReadType(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, var: *mut super::super::super::System::Com::VARIANT, r#type: super::super::super::System::Com::VARENUM) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadULONGLONG(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_ReadUnknown(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_WriteBOOL(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteBSTR(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: ::windows_sys::core::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteDWORD(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteGUID(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *const ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteInt(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteLONG(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_WritePOINTL(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *const super::super::super::Foundation::POINTL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_WritePOINTS(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *const super::super::super::Foundation::POINTS) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WritePropertyKey(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *const PROPERTYKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSPropertyBag_WriteRECTL(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: *const super::super::super::Foundation::RECTL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteSHORT(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: i16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteStr(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteStream(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: super::super::super::System::Com::IStream) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteULONGLONG(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, value: u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn PSPropertyBag_WriteUnknown(propbag: super::super::super::System::Com::StructuredStorage::IPropertyBag, propname: ::windows_sys::core::PCWSTR, punk: ::windows_sys::core::IUnknown) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSPropertyKeyFromString(pszstring: ::windows_sys::core::PCWSTR, pkey: *mut PROPERTYKEY) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSRefreshPropertySchema() -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSRegisterPropertySchema(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PSSetPropertyValue(pps: IPropertyStore, ppd: IPropertyDescription, propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSStringFromPropertyKey(pkey: *const PROPERTYKEY, psz: ::windows_sys::core::PWSTR, cch: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn PSUnregisterPropertySchema(pszpath: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PifMgr_CloseProperties(hprops: super::super::super::Foundation::HANDLE, flopt: u32) -> super::super::super::Foundation::HANDLE;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PifMgr_GetProperties(hprops: super::super::super::Foundation::HANDLE, pszgroup: ::windows_sys::core::PCSTR, lpprops: *mut ::core::ffi::c_void, cbprops: i32, flopt: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PifMgr_OpenProperties(pszapp: ::windows_sys::core::PCWSTR, pszpif: ::windows_sys::core::PCWSTR, hinf: u32, flopt: u32) -> super::super::super::Foundation::HANDLE;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn PifMgr_SetProperties(hprops: super::super::super::Foundation::HANDLE, pszgroup: ::windows_sys::core::PCSTR, lpprops: *const ::core::ffi::c_void, cbprops: i32, flopt: u32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantChangeType(ppropvardest: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT, propvarsrc: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, flags: PROPVAR_CHANGE_FLAGS, vt: super::super::super::System::Com::VARENUM) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantCompareEx(propvar1: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, propvar2: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, unit: PROPVAR_COMPARE_UNIT, flags: PROPVAR_COMPARE_FLAGS) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetBooleanElem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pfval: *mut super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetDoubleElem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetElementCount(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetFileTimeElem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pftval: *mut super::super::super::Foundation::FILETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetInt16Elem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut i16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetInt32Elem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetInt64Elem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut i64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetStringElem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, ppszval: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetUInt16Elem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut u16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetUInt32Elem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantGetUInt64Elem(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ielem: u32, pnval: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToBSTR(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pbstrout: *mut ::windows_sys::core::BSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToBoolean(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pfret: *mut super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToBooleanVector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgf: *mut super::super::super::Foundation::BOOL, crgf: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToBooleanVectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgf: *mut *mut super::super::super::Foundation::BOOL, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToBooleanWithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, fdefault: super::super::super::Foundation::BOOL) -> super::super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToBuffer(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pv: *mut ::core::ffi::c_void, cb: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToDouble(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pdblret: *mut f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToDoubleVector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut f64, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToDoubleVectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut f64, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToDoubleWithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, dbldefault: f64) -> f64;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToFileTime(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pstfout: PSTIME_FLAGS, pftout: *mut super::super::super::Foundation::FILETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToFileTimeVector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgft: *mut super::super::super::Foundation::FILETIME, crgft: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToFileTimeVectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgft: *mut *mut super::super::super::Foundation::FILETIME, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToGUID(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pguid: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt16(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, piret: *mut i16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt16Vector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut i16, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt16VectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut i16, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt16WithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, idefault: i16) -> i16;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt32(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, plret: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt32Vector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut i32, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt32VectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut i32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt32WithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ldefault: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt64(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pllret: *mut i64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt64Vector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut i64, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt64VectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut i64, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToInt64WithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, lldefault: i64) -> i64;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_UI_Shell_Common"))]
    pub fn PropVariantToStrRet(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pstrret: *mut super::Common::STRRET) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToString(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, psz: ::windows_sys::core::PWSTR, cch: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToStringAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ppszout: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToStringVector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgsz: *mut ::windows_sys::core::PWSTR, crgsz: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToStringVectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgsz: *mut *mut ::windows_sys::core::PWSTR, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToStringWithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pszdefault: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt16(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, puiret: *mut u16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt16Vector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut u16, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt16VectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut u16, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt16WithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, uidefault: u16) -> u16;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt32(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pulret: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt32Vector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut u32, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt32VectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt32WithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, uldefault: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt64(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pullret: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt64Vector(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, prgn: *mut u64, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt64VectorAlloc(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pprgn: *mut *mut u64, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToUInt64WithDefault(propvarin: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, ulldefault: u64) -> u64;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_System_Ole"))]
    pub fn PropVariantToVariant(ppropvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, pvar: *mut super::super::super::System::Com::VARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn PropVariantToWinRTPropertyValue(propvar: *const super::super::super::System::Com::StructuredStorage::PROPVARIANT, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
    pub fn SHAddDefaultPropertiesByExt(pszext: ::windows_sys::core::PCWSTR, ppropstore: IPropertyStore) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn SHGetPropertyStoreForWindow(hwnd: super::super::super::Foundation::HWND, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(feature = "Win32_UI_Shell_Common")]
    pub fn SHGetPropertyStoreFromIDList(pidl: *const super::Common::ITEMIDLIST, flags: GETPROPERTYSTOREFLAGS, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn SHGetPropertyStoreFromParsingName(pszpath: ::windows_sys::core::PCWSTR, pbc: super::super::super::System::Com::IBindCtx, flags: GETPROPERTYSTOREFLAGS, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(feature = "Win32_System_Com_StructuredStorage")]
    pub fn SHPropStgCreate(psstg: super::super::super::System::Com::StructuredStorage::IPropertySetStorage, fmtid: *const ::windows_sys::core::GUID, pclsid: *const ::windows_sys::core::GUID, grfflags: u32, grfmode: u32, dwdisposition: u32, ppstg: *mut super::super::super::System::Com::StructuredStorage::IPropertyStorage, pucodepage: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn SHPropStgReadMultiple(pps: super::super::super::System::Com::StructuredStorage::IPropertyStorage, ucodepage: u32, cpspec: u32, rgpspec: *const super::super::super::System::Com::StructuredStorage::PROPSPEC, rgvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn SHPropStgWriteMultiple(pps: super::super::super::System::Com::StructuredStorage::IPropertyStorage, pucodepage: *mut u32, cpspec: u32, rgpspec: *const super::super::super::System::Com::StructuredStorage::PROPSPEC, rgvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT, propidnamefirst: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantCompare(var1: *const super::super::super::System::Com::VARIANT, var2: *const super::super::super::System::Com::VARIANT) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetBooleanElem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pfval: *mut super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetDoubleElem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetElementCount(varin: *const super::super::super::System::Com::VARIANT) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetInt16Elem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut i16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetInt32Elem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetInt64Elem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut i64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetStringElem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, ppszval: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetUInt16Elem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut u16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetUInt32Elem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantGetUInt64Elem(var: *const super::super::super::System::Com::VARIANT, ielem: u32, pnval: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToBoolean(varin: *const super::super::super::System::Com::VARIANT, pfret: *mut super::super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToBooleanArray(var: *const super::super::super::System::Com::VARIANT, prgf: *mut super::super::super::Foundation::BOOL, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToBooleanArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgf: *mut *mut super::super::super::Foundation::BOOL, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToBooleanWithDefault(varin: *const super::super::super::System::Com::VARIANT, fdefault: super::super::super::Foundation::BOOL) -> super::super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToBuffer(varin: *const super::super::super::System::Com::VARIANT, pv: *mut ::core::ffi::c_void, cb: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToDosDateTime(varin: *const super::super::super::System::Com::VARIANT, pwdate: *mut u16, pwtime: *mut u16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToDouble(varin: *const super::super::super::System::Com::VARIANT, pdblret: *mut f64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToDoubleArray(var: *const super::super::super::System::Com::VARIANT, prgn: *mut f64, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToDoubleArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut f64, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToDoubleWithDefault(varin: *const super::super::super::System::Com::VARIANT, dbldefault: f64) -> f64;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToFileTime(varin: *const super::super::super::System::Com::VARIANT, stfout: PSTIME_FLAGS, pftout: *mut super::super::super::Foundation::FILETIME) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToGUID(varin: *const super::super::super::System::Com::VARIANT, pguid: *mut ::windows_sys::core::GUID) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt16(varin: *const super::super::super::System::Com::VARIANT, piret: *mut i16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt16Array(var: *const super::super::super::System::Com::VARIANT, prgn: *mut i16, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt16ArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut i16, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt16WithDefault(varin: *const super::super::super::System::Com::VARIANT, idefault: i16) -> i16;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt32(varin: *const super::super::super::System::Com::VARIANT, plret: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt32Array(var: *const super::super::super::System::Com::VARIANT, prgn: *mut i32, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt32ArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut i32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt32WithDefault(varin: *const super::super::super::System::Com::VARIANT, ldefault: i32) -> i32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt64(varin: *const super::super::super::System::Com::VARIANT, pllret: *mut i64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt64Array(var: *const super::super::super::System::Com::VARIANT, prgn: *mut i64, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt64ArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut i64, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToInt64WithDefault(varin: *const super::super::super::System::Com::VARIANT, lldefault: i64) -> i64;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage", feature = "Win32_System_Ole"))]
    pub fn VariantToPropVariant(pvar: *const super::super::super::System::Com::VARIANT, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`, `\"Win32_UI_Shell_Common\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole", feature = "Win32_UI_Shell_Common"))]
    pub fn VariantToStrRet(varin: *const super::super::super::System::Com::VARIANT, pstrret: *mut super::Common::STRRET) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToString(varin: *const super::super::super::System::Com::VARIANT, pszbuf: ::windows_sys::core::PWSTR, cchbuf: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToStringAlloc(varin: *const super::super::super::System::Com::VARIANT, ppszbuf: *mut ::windows_sys::core::PWSTR) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToStringArray(var: *const super::super::super::System::Com::VARIANT, prgsz: *mut ::windows_sys::core::PWSTR, crgsz: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToStringArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgsz: *mut *mut ::windows_sys::core::PWSTR, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToStringWithDefault(varin: *const super::super::super::System::Com::VARIANT, pszdefault: ::windows_sys::core::PCWSTR) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt16(varin: *const super::super::super::System::Com::VARIANT, puiret: *mut u16) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt16Array(var: *const super::super::super::System::Com::VARIANT, prgn: *mut u16, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt16ArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut u16, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt16WithDefault(varin: *const super::super::super::System::Com::VARIANT, uidefault: u16) -> u16;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt32(varin: *const super::super::super::System::Com::VARIANT, pulret: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt32Array(var: *const super::super::super::System::Com::VARIANT, prgn: *mut u32, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt32ArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt32WithDefault(varin: *const super::super::super::System::Com::VARIANT, uldefault: u32) -> u32;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt64(varin: *const super::super::super::System::Com::VARIANT, pullret: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt64Array(var: *const super::super::super::System::Com::VARIANT, prgn: *mut u64, crgn: u32, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt64ArrayAlloc(var: *const super::super::super::System::Com::VARIANT, pprgn: *mut *mut u64, pcelem: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com\"`, `\"Win32_System_Ole\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com", feature = "Win32_System_Ole"))]
    pub fn VariantToUInt64WithDefault(varin: *const super::super::super::System::Com::VARIANT, ulldefault: u64) -> u64;
    #[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`, `\"Win32_System_Com_StructuredStorage\"`*"]
    #[cfg(all(feature = "Win32_Foundation", feature = "Win32_System_Com_StructuredStorage"))]
    pub fn WinRTPropertyValueToPropVariant(punkpropertyvalue: ::windows_sys::core::IUnknown, ppropvar: *mut super::super::super::System::Com::StructuredStorage::PROPVARIANT) -> ::windows_sys::core::HRESULT;
}
pub type ICreateObject = *mut ::core::ffi::c_void;
pub type IDelayedPropertyStoreFactory = *mut ::core::ffi::c_void;
pub type IInitializeWithFile = *mut ::core::ffi::c_void;
pub type IInitializeWithStream = *mut ::core::ffi::c_void;
pub type INamedPropertyStore = *mut ::core::ffi::c_void;
pub type IObjectWithPropertyKey = *mut ::core::ffi::c_void;
pub type IPersistSerializedPropStorage = *mut ::core::ffi::c_void;
pub type IPersistSerializedPropStorage2 = *mut ::core::ffi::c_void;
pub type IPropertyChange = *mut ::core::ffi::c_void;
pub type IPropertyChangeArray = *mut ::core::ffi::c_void;
pub type IPropertyDescription = *mut ::core::ffi::c_void;
pub type IPropertyDescription2 = *mut ::core::ffi::c_void;
pub type IPropertyDescriptionAliasInfo = *mut ::core::ffi::c_void;
pub type IPropertyDescriptionList = *mut ::core::ffi::c_void;
pub type IPropertyDescriptionRelatedPropertyInfo = *mut ::core::ffi::c_void;
pub type IPropertyDescriptionSearchInfo = *mut ::core::ffi::c_void;
pub type IPropertyEnumType = *mut ::core::ffi::c_void;
pub type IPropertyEnumType2 = *mut ::core::ffi::c_void;
pub type IPropertyEnumTypeList = *mut ::core::ffi::c_void;
pub type IPropertyStore = *mut ::core::ffi::c_void;
pub type IPropertyStoreCache = *mut ::core::ffi::c_void;
pub type IPropertyStoreCapabilities = *mut ::core::ffi::c_void;
pub type IPropertyStoreFactory = *mut ::core::ffi::c_void;
pub type IPropertySystem = *mut ::core::ffi::c_void;
pub type IPropertySystemChangeNotify = *mut ::core::ffi::c_void;
pub type IPropertyUI = *mut ::core::ffi::c_void;
pub const InMemoryPropertyStore: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 2583879698, data2: 25347, data3: 19998, data4: [185, 161, 99, 15, 128, 37, 146, 197] };
pub const InMemoryPropertyStoreMarshalByValue: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3570011693, data2: 28071, data3: 19317, data4: [169, 124, 95, 48, 111, 14, 174, 220] };
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PKEY_PIDSTR_MAX: u32 = 10u32;
pub const PropertySystem: ::windows_sys::core::GUID = ::windows_sys::core::GUID { data1: 3096870789, data2: 22702, data3: 20294, data4: [159, 178, 93, 121, 4, 121, 143, 75] };
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type DRAWPROGRESSFLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const DPF_NONE: DRAWPROGRESSFLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const DPF_MARQUEE: DRAWPROGRESSFLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const DPF_MARQUEE_COMPLETE: DRAWPROGRESSFLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const DPF_ERROR: DRAWPROGRESSFLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const DPF_WARNING: DRAWPROGRESSFLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const DPF_STOPPED: DRAWPROGRESSFLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type GETPROPERTYSTOREFLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_DEFAULT: GETPROPERTYSTOREFLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_HANDLERPROPERTIESONLY: GETPROPERTYSTOREFLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_READWRITE: GETPROPERTYSTOREFLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_TEMPORARY: GETPROPERTYSTOREFLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_FASTPROPERTIESONLY: GETPROPERTYSTOREFLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_OPENSLOWITEM: GETPROPERTYSTOREFLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_DELAYCREATION: GETPROPERTYSTOREFLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_BESTEFFORT: GETPROPERTYSTOREFLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_NO_OPLOCK: GETPROPERTYSTOREFLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_PREFERQUERYPROPERTIES: GETPROPERTYSTOREFLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_EXTRINSICPROPERTIES: GETPROPERTYSTOREFLAGS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_EXTRINSICPROPERTIESONLY: GETPROPERTYSTOREFLAGS = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_VOLATILEPROPERTIES: GETPROPERTYSTOREFLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_VOLATILEPROPERTIESONLY: GETPROPERTYSTOREFLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const GPS_MASK_VALID: GETPROPERTYSTOREFLAGS = 8191u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PDOPSTATUS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDOPS_RUNNING: PDOPSTATUS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDOPS_PAUSED: PDOPSTATUS = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDOPS_CANCELLED: PDOPSTATUS = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDOPS_STOPPED: PDOPSTATUS = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDOPS_ERRORS: PDOPSTATUS = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PKA_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PKA_SET: PKA_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PKA_APPEND: PKA_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PKA_DELETE: PKA_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PLACEHOLDER_STATES = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_NONE: PLACEHOLDER_STATES = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_MARKED_FOR_OFFLINE_AVAILABILITY: PLACEHOLDER_STATES = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_FULL_PRIMARY_STREAM_AVAILABLE: PLACEHOLDER_STATES = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_CREATE_FILE_ACCESSIBLE: PLACEHOLDER_STATES = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_CLOUDFILE_PLACEHOLDER: PLACEHOLDER_STATES = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_DEFAULT: PLACEHOLDER_STATES = 7u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PS_ALL: PLACEHOLDER_STATES = 15u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_AGGREGATION_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_DEFAULT: PROPDESC_AGGREGATION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_FIRST: PROPDESC_AGGREGATION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_SUM: PROPDESC_AGGREGATION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_AVERAGE: PROPDESC_AGGREGATION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_DATERANGE: PROPDESC_AGGREGATION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_UNION: PROPDESC_AGGREGATION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_MAX: PROPDESC_AGGREGATION_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDAT_MIN: PROPDESC_AGGREGATION_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_COLUMNINDEX_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCIT_NONE: PROPDESC_COLUMNINDEX_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCIT_ONDISK: PROPDESC_COLUMNINDEX_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCIT_INMEMORY: PROPDESC_COLUMNINDEX_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCIT_ONDEMAND: PROPDESC_COLUMNINDEX_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCIT_ONDISKALL: PROPDESC_COLUMNINDEX_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCIT_ONDISKVECTOR: PROPDESC_COLUMNINDEX_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_CONDITION_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCOT_NONE: PROPDESC_CONDITION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCOT_STRING: PROPDESC_CONDITION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCOT_SIZE: PROPDESC_CONDITION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCOT_DATETIME: PROPDESC_CONDITION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCOT_BOOLEAN: PROPDESC_CONDITION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDCOT_NUMBER: PROPDESC_CONDITION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_DISPLAYTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDDT_STRING: PROPDESC_DISPLAYTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDDT_NUMBER: PROPDESC_DISPLAYTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDDT_BOOLEAN: PROPDESC_DISPLAYTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDDT_DATETIME: PROPDESC_DISPLAYTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDDT_ENUMERATED: PROPDESC_DISPLAYTYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_ENUMFILTER = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_ALL: PROPDESC_ENUMFILTER = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_SYSTEM: PROPDESC_ENUMFILTER = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_NONSYSTEM: PROPDESC_ENUMFILTER = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_VIEWABLE: PROPDESC_ENUMFILTER = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_QUERYABLE: PROPDESC_ENUMFILTER = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_INFULLTEXTQUERY: PROPDESC_ENUMFILTER = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDEF_COLUMN: PROPDESC_ENUMFILTER = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_FORMAT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_DEFAULT: PROPDESC_FORMAT_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_PREFIXNAME: PROPDESC_FORMAT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_FILENAME: PROPDESC_FORMAT_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_ALWAYSKB: PROPDESC_FORMAT_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_RESERVED_RIGHTTOLEFT: PROPDESC_FORMAT_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_SHORTTIME: PROPDESC_FORMAT_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_LONGTIME: PROPDESC_FORMAT_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_HIDETIME: PROPDESC_FORMAT_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_SHORTDATE: PROPDESC_FORMAT_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_LONGDATE: PROPDESC_FORMAT_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_HIDEDATE: PROPDESC_FORMAT_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_RELATIVEDATE: PROPDESC_FORMAT_FLAGS = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_USEEDITINVITATION: PROPDESC_FORMAT_FLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_READONLY: PROPDESC_FORMAT_FLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDFF_NOAUTOREADINGORDER: PROPDESC_FORMAT_FLAGS = 8192u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_GROUPING_RANGE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_DISCRETE: PROPDESC_GROUPING_RANGE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_ALPHANUMERIC: PROPDESC_GROUPING_RANGE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_SIZE: PROPDESC_GROUPING_RANGE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_DYNAMIC: PROPDESC_GROUPING_RANGE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_DATE: PROPDESC_GROUPING_RANGE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_PERCENT: PROPDESC_GROUPING_RANGE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDGR_ENUMERATED: PROPDESC_GROUPING_RANGE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_RELATIVEDESCRIPTION_TYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_GENERAL: PROPDESC_RELATIVEDESCRIPTION_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_DATE: PROPDESC_RELATIVEDESCRIPTION_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_SIZE: PROPDESC_RELATIVEDESCRIPTION_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_COUNT: PROPDESC_RELATIVEDESCRIPTION_TYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_REVISION: PROPDESC_RELATIVEDESCRIPTION_TYPE = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_LENGTH: PROPDESC_RELATIVEDESCRIPTION_TYPE = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_DURATION: PROPDESC_RELATIVEDESCRIPTION_TYPE = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_SPEED: PROPDESC_RELATIVEDESCRIPTION_TYPE = 7i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_RATE: PROPDESC_RELATIVEDESCRIPTION_TYPE = 8i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_RATING: PROPDESC_RELATIVEDESCRIPTION_TYPE = 9i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDRDT_PRIORITY: PROPDESC_RELATIVEDESCRIPTION_TYPE = 10i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_SEARCHINFO_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSIF_DEFAULT: PROPDESC_SEARCHINFO_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSIF_ININVERTEDINDEX: PROPDESC_SEARCHINFO_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSIF_ISCOLUMN: PROPDESC_SEARCHINFO_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSIF_ISCOLUMNSPARSE: PROPDESC_SEARCHINFO_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSIF_ALWAYSINCLUDE: PROPDESC_SEARCHINFO_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSIF_USEFORTYPEAHEAD: PROPDESC_SEARCHINFO_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_SORTDESCRIPTION = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSD_GENERAL: PROPDESC_SORTDESCRIPTION = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSD_A_Z: PROPDESC_SORTDESCRIPTION = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSD_LOWEST_HIGHEST: PROPDESC_SORTDESCRIPTION = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSD_SMALLEST_BIGGEST: PROPDESC_SORTDESCRIPTION = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDSD_OLDEST_NEWEST: PROPDESC_SORTDESCRIPTION = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_TYPE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_DEFAULT: PROPDESC_TYPE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_MULTIPLEVALUES: PROPDESC_TYPE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ISINNATE: PROPDESC_TYPE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ISGROUP: PROPDESC_TYPE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_CANGROUPBY: PROPDESC_TYPE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_CANSTACKBY: PROPDESC_TYPE_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ISTREEPROPERTY: PROPDESC_TYPE_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_INCLUDEINFULLTEXTQUERY: PROPDESC_TYPE_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ISVIEWABLE: PROPDESC_TYPE_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ISQUERYABLE: PROPDESC_TYPE_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_CANBEPURGED: PROPDESC_TYPE_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_SEARCHRAWVALUE: PROPDESC_TYPE_FLAGS = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_DONTCOERCEEMPTYSTRINGS: PROPDESC_TYPE_FLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ALWAYSINSUPPLEMENTALSTORE: PROPDESC_TYPE_FLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_ISSYSTEMPROPERTY: PROPDESC_TYPE_FLAGS = 2147483648u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDTF_MASK_ALL: PROPDESC_TYPE_FLAGS = 2147491839u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPDESC_VIEW_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_DEFAULT: PROPDESC_VIEW_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_CENTERALIGN: PROPDESC_VIEW_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_RIGHTALIGN: PROPDESC_VIEW_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_BEGINNEWGROUP: PROPDESC_VIEW_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_FILLAREA: PROPDESC_VIEW_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_SORTDESCENDING: PROPDESC_VIEW_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_SHOWONLYIFPRESENT: PROPDESC_VIEW_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_SHOWBYDEFAULT: PROPDESC_VIEW_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_SHOWINPRIMARYLIST: PROPDESC_VIEW_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_SHOWINSECONDARYLIST: PROPDESC_VIEW_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_HIDELABEL: PROPDESC_VIEW_FLAGS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_HIDDEN: PROPDESC_VIEW_FLAGS = 2048u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_CANWRAP: PROPDESC_VIEW_FLAGS = 4096u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PDVF_MASK_ALL: PROPDESC_VIEW_FLAGS = 7167u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPENUMTYPE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PET_DISCRETEVALUE: PROPENUMTYPE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PET_RANGEDVALUE: PROPENUMTYPE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PET_DEFAULTVALUE: PROPENUMTYPE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PET_ENDRANGE: PROPENUMTYPE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPERTYUI_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIF_DEFAULT: PROPERTYUI_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIF_RIGHTALIGN: PROPERTYUI_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIF_NOLABELININFOTIP: PROPERTYUI_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPERTYUI_FORMAT_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFFDF_DEFAULT: PROPERTYUI_FORMAT_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFFDF_RIGHTTOLEFT: PROPERTYUI_FORMAT_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFFDF_SHORTFORMAT: PROPERTYUI_FORMAT_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFFDF_NOTIME: PROPERTYUI_FORMAT_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFFDF_FRIENDLYDATE: PROPERTYUI_FORMAT_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPERTYUI_NAME_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFNF_DEFAULT: PROPERTYUI_NAME_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PUIFNF_MNEMONIC: PROPERTYUI_NAME_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPVAR_CHANGE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCHF_DEFAULT: PROPVAR_CHANGE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCHF_NOVALUEPROP: PROPVAR_CHANGE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCHF_ALPHABOOL: PROPVAR_CHANGE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCHF_NOUSEROVERRIDE: PROPVAR_CHANGE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCHF_LOCALBOOL: PROPVAR_CHANGE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCHF_NOHEXSTRING: PROPVAR_CHANGE_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPVAR_COMPARE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_DEFAULT: PROPVAR_COMPARE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_TREATEMPTYASGREATERTHAN: PROPVAR_COMPARE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_USESTRCMP: PROPVAR_COMPARE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_USESTRCMPC: PROPVAR_COMPARE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_USESTRCMPI: PROPVAR_COMPARE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_USESTRCMPIC: PROPVAR_COMPARE_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCF_DIGITSASNUMBERS_CASESENSITIVE: PROPVAR_COMPARE_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PROPVAR_COMPARE_UNIT = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_DEFAULT: PROPVAR_COMPARE_UNIT = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_SECOND: PROPVAR_COMPARE_UNIT = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_MINUTE: PROPVAR_COMPARE_UNIT = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_HOUR: PROPVAR_COMPARE_UNIT = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_DAY: PROPVAR_COMPARE_UNIT = 4i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_MONTH: PROPVAR_COMPARE_UNIT = 5i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PVCU_YEAR: PROPVAR_COMPARE_UNIT = 6i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PSC_STATE = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PSC_NORMAL: PSC_STATE = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PSC_NOTINSOURCE: PSC_STATE = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PSC_DIRTY: PSC_STATE = 2i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PSC_READONLY: PSC_STATE = 3i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type PSTIME_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PSTF_UTC: PSTIME_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const PSTF_LOCAL: PSTIME_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type SYNC_ENGINE_STATE_FLAGS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_NONE: SYNC_ENGINE_STATE_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_SERVICE_QUOTA_NEARING_LIMIT: SYNC_ENGINE_STATE_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_SERVICE_QUOTA_EXCEEDED_LIMIT: SYNC_ENGINE_STATE_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_AUTHENTICATION_ERROR: SYNC_ENGINE_STATE_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_PAUSED_DUE_TO_METERED_NETWORK: SYNC_ENGINE_STATE_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_PAUSED_DUE_TO_DISK_SPACE_FULL: SYNC_ENGINE_STATE_FLAGS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_PAUSED_DUE_TO_CLIENT_POLICY: SYNC_ENGINE_STATE_FLAGS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_PAUSED_DUE_TO_SERVICE_POLICY: SYNC_ENGINE_STATE_FLAGS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_SERVICE_UNAVAILABLE: SYNC_ENGINE_STATE_FLAGS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_PAUSED_DUE_TO_USER_REQUEST: SYNC_ENGINE_STATE_FLAGS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const SESF_ALL_FLAGS: SYNC_ENGINE_STATE_FLAGS = 511u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type SYNC_TRANSFER_STATUS = u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_NONE: SYNC_TRANSFER_STATUS = 0u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_NEEDSUPLOAD: SYNC_TRANSFER_STATUS = 1u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_NEEDSDOWNLOAD: SYNC_TRANSFER_STATUS = 2u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_TRANSFERRING: SYNC_TRANSFER_STATUS = 4u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_PAUSED: SYNC_TRANSFER_STATUS = 8u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_HASERROR: SYNC_TRANSFER_STATUS = 16u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_FETCHING_METADATA: SYNC_TRANSFER_STATUS = 32u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_USER_REQUESTED_REFRESH: SYNC_TRANSFER_STATUS = 64u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_HASWARNING: SYNC_TRANSFER_STATUS = 128u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_EXCLUDED: SYNC_TRANSFER_STATUS = 256u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_INCOMPLETE: SYNC_TRANSFER_STATUS = 512u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const STS_PLACEHOLDER_IFEMPTY: SYNC_TRANSFER_STATUS = 1024u32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub type _PERSIST_SPROPSTORE_FLAGS = i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const FPSPS_DEFAULT: _PERSIST_SPROPSTORE_FLAGS = 0i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const FPSPS_READONLY: _PERSIST_SPROPSTORE_FLAGS = 1i32;
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub const FPSPS_TREAT_NEW_VALUES_AS_DIRTY: _PERSIST_SPROPSTORE_FLAGS = 2i32;
#[repr(C)]
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`*"]
pub struct PROPERTYKEY {
    pub fmtid: ::windows_sys::core::GUID,
    pub pid: u32,
}
impl ::core::marker::Copy for PROPERTYKEY {}
impl ::core::clone::Clone for PROPERTYKEY {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C, packed(1))]
#[doc = "*Required features: `\"Win32_UI_Shell_PropertiesSystem\"`, `\"Win32_Foundation\"`*"]
#[cfg(feature = "Win32_Foundation")]
pub struct PROPPRG {
    pub flPrg: u16,
    pub flPrgInit: u16,
    pub achTitle: [super::super::super::Foundation::CHAR; 30],
    pub achCmdLine: [super::super::super::Foundation::CHAR; 128],
    pub achWorkDir: [super::super::super::Foundation::CHAR; 64],
    pub wHotKey: u16,
    pub achIconFile: [super::super::super::Foundation::CHAR; 80],
    pub wIconIndex: u16,
    pub dwEnhModeFlags: u32,
    pub dwRealModeFlags: u32,
    pub achOtherFile: [super::super::super::Foundation::CHAR; 80],
    pub achPIFFile: [super::super::super::Foundation::CHAR; 260],
}
#[cfg(feature = "Win32_Foundation")]
impl ::core::marker::Copy for PROPPRG {}
#[cfg(feature = "Win32_Foundation")]
impl ::core::clone::Clone for PROPPRG {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct SERIALIZEDPROPSTORAGE(pub u8);
