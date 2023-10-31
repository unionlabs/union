#[cfg(feature = "Win32_System_WinRT_AllJoyn")]
pub mod AllJoyn;
#[cfg(feature = "Win32_System_WinRT_Composition")]
pub mod Composition;
#[cfg(feature = "Win32_System_WinRT_CoreInputView")]
pub mod CoreInputView;
#[cfg(feature = "Win32_System_WinRT_Direct3D11")]
pub mod Direct3D11;
#[cfg(feature = "Win32_System_WinRT_Display")]
pub mod Display;
#[cfg(feature = "Win32_System_WinRT_Graphics")]
pub mod Graphics;
#[cfg(feature = "Win32_System_WinRT_Holographic")]
pub mod Holographic;
#[cfg(feature = "Win32_System_WinRT_Isolation")]
pub mod Isolation;
#[cfg(feature = "Win32_System_WinRT_ML")]
pub mod ML;
#[cfg(feature = "Win32_System_WinRT_Media")]
pub mod Media;
#[cfg(feature = "Win32_System_WinRT_Pdf")]
pub mod Pdf;
#[cfg(feature = "Win32_System_WinRT_Printing")]
pub mod Printing;
#[cfg(feature = "Win32_System_WinRT_Shell")]
pub mod Shell;
#[cfg(feature = "Win32_System_WinRT_Storage")]
pub mod Storage;
#[cfg_attr(windows, link(name = "windows"))]
extern "cdecl" {
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn CreateControlInput(riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn CreateControlInputEx(pcorewindow: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
}
#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn CoDecodeProxy(dwclientpid: u32, ui64proxyaddress: u64, pserverinformation: *mut ServerInformation) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"System\"`*"]
    #[cfg(feature = "System")]
    pub fn CreateDispatcherQueueController(options: DispatcherQueueOptions, dispatcherqueuecontroller: *mut super::super::super::System::DispatcherQueueController) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn CreateRandomAccessStreamOnFile(filepath: ::windows_sys::core::PCWSTR, accessmode: u32, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_System_Com\"`*"]
    #[cfg(feature = "Win32_System_Com")]
    pub fn CreateRandomAccessStreamOverStream(stream: super::Com::IStream, options: BSOS_OPTIONS, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn CreateStreamOverRandomAccessStream(randomaccessstream: ::windows_sys::core::IUnknown, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn GetRestrictedErrorInfo(pprestrictederrorinfo: *mut IRestrictedErrorInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserFree(param0: *const u32, param1: *const ::windows_sys::core::HSTRING);
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserFree64(param0: *const u32, param1: *const ::windows_sys::core::HSTRING);
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserMarshal(param0: *const u32, param1: *mut u8, param2: *const ::windows_sys::core::HSTRING) -> *mut u8;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserMarshal64(param0: *const u32, param1: *mut u8, param2: *const ::windows_sys::core::HSTRING) -> *mut u8;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserSize(param0: *const u32, param1: u32, param2: *const ::windows_sys::core::HSTRING) -> u32;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserSize64(param0: *const u32, param1: u32, param2: *const ::windows_sys::core::HSTRING) -> u32;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserUnmarshal(param0: *const u32, param1: *const u8, param2: *mut ::windows_sys::core::HSTRING) -> *mut u8;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn HSTRING_UserUnmarshal64(param0: *const u32, param1: *const u8, param2: *mut ::windows_sys::core::HSTRING) -> *mut u8;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn IsErrorPropagationEnabled() -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn MetaDataGetDispenser(rclsid: *const ::windows_sys::core::GUID, riid: *const ::windows_sys::core::GUID, ppv: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoActivateInstance(activatableclassid: ::windows_sys::core::HSTRING, instance: *mut ::windows_sys::core::IInspectable) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoCaptureErrorContext(hr: ::windows_sys::core::HRESULT) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoClearError();
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoFailFastWithErrorContext(hrerror: ::windows_sys::core::HRESULT);
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoFreeParameterizedTypeExtra(extra: ROPARAMIIDHANDLE);
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetActivationFactory(activatableclassid: ::windows_sys::core::HSTRING, iid: *const ::windows_sys::core::GUID, factory: *mut *mut ::core::ffi::c_void) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetAgileReference(options: AgileReferenceOptions, riid: *const ::windows_sys::core::GUID, punk: ::windows_sys::core::IUnknown, ppagilereference: *mut IAgileReference) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetApartmentIdentifier(apartmentidentifier: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_System_Com_Marshal\"`*"]
    #[cfg(feature = "Win32_System_Com_Marshal")]
    pub fn RoGetBufferMarshaler(buffermarshaler: *mut super::Com::Marshal::IMarshal) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetErrorReportingFlags(pflags: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetMatchingRestrictedErrorInfo(hrin: ::windows_sys::core::HRESULT, pprestrictederrorinfo: *mut IRestrictedErrorInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetParameterizedTypeInstanceIID(nameelementcount: u32, nameelements: *const ::windows_sys::core::PWSTR, metadatalocator: IRoMetaDataLocator, iid: *mut ::windows_sys::core::GUID, pextra: *mut ROPARAMIIDHANDLE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoGetServerActivatableClasses(servername: ::windows_sys::core::HSTRING, activatableclassids: *mut *mut ::windows_sys::core::HSTRING, count: *mut u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoInitialize(inittype: RO_INIT_TYPE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoInspectCapturedStackBackTrace(targeterrorinfoaddress: usize, machine: u16, readmemorycallback: PINSPECT_MEMORY_CALLBACK, context: *const ::core::ffi::c_void, framecount: *mut u32, targetbacktraceaddress: *mut usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoInspectThreadErrorInfo(targettebaddress: usize, machine: u16, readmemorycallback: PINSPECT_MEMORY_CALLBACK, context: *const ::core::ffi::c_void, targeterrorinfoaddress: *mut usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RoOriginateError(error: ::windows_sys::core::HRESULT, message: ::windows_sys::core::HSTRING) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RoOriginateErrorW(error: ::windows_sys::core::HRESULT, cchmax: u32, message: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RoOriginateLanguageException(error: ::windows_sys::core::HRESULT, message: ::windows_sys::core::HSTRING, languageexception: ::windows_sys::core::IUnknown) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoParameterizedTypeExtraGetTypeSignature(extra: ROPARAMIIDHANDLE) -> ::windows_sys::core::PSTR;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoRegisterActivationFactories(activatableclassids: *const ::windows_sys::core::HSTRING, activationfactorycallbacks: *const isize, count: u32, cookie: *mut isize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoRegisterForApartmentShutdown(callbackobject: IApartmentShutdown, apartmentidentifier: *mut u64, regcookie: *mut APARTMENT_SHUTDOWN_REGISTRATION_COOKIE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoReportFailedDelegate(punkdelegate: ::windows_sys::core::IUnknown, prestrictederrorinfo: IRestrictedErrorInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoReportUnhandledError(prestrictederrorinfo: IRestrictedErrorInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoResolveRestrictedErrorInfoReference(reference: ::windows_sys::core::PCWSTR, pprestrictederrorinfo: *mut IRestrictedErrorInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoRevokeActivationFactories(cookie: isize);
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoSetErrorReportingFlags(flags: u32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RoTransformError(olderror: ::windows_sys::core::HRESULT, newerror: ::windows_sys::core::HRESULT, message: ::windows_sys::core::HSTRING) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RoTransformErrorW(olderror: ::windows_sys::core::HRESULT, newerror: ::windows_sys::core::HRESULT, cchmax: u32, message: ::windows_sys::core::PCWSTR) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoUninitialize();
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn RoUnregisterForApartmentShutdown(regcookie: APARTMENT_SHUTDOWN_REGISTRATION_COOKIE) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn SetRestrictedErrorInfo(prestrictederrorinfo: IRestrictedErrorInfo) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsCompareStringOrdinal(string1: ::windows_sys::core::HSTRING, string2: ::windows_sys::core::HSTRING, result: *mut i32) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsConcatString(string1: ::windows_sys::core::HSTRING, string2: ::windows_sys::core::HSTRING, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsCreateString(sourcestring: ::windows_sys::core::PCWSTR, length: u32, string: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsCreateStringReference(sourcestring: ::windows_sys::core::PCWSTR, length: u32, hstringheader: *mut HSTRING_HEADER, string: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsDeleteString(string: ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsDeleteStringBuffer(bufferhandle: HSTRING_BUFFER) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsDuplicateString(string: ::windows_sys::core::HSTRING, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsGetStringLen(string: ::windows_sys::core::HSTRING) -> u32;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsGetStringRawBuffer(string: ::windows_sys::core::HSTRING, length: *mut u32) -> ::windows_sys::core::PWSTR;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsInspectString(targethstring: usize, machine: u16, callback: PINSPECT_HSTRING_CALLBACK, context: *const ::core::ffi::c_void, length: *mut u32, targetstringaddress: *mut usize) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsInspectString2(targethstring: u64, machine: u16, callback: PINSPECT_HSTRING_CALLBACK2, context: *const ::core::ffi::c_void, length: *mut u32, targetstringaddress: *mut u64) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WindowsIsStringEmpty(string: ::windows_sys::core::HSTRING) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsPreallocateStringBuffer(length: u32, charbuffer: *mut *mut u16, bufferhandle: *mut HSTRING_BUFFER) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsPromoteStringBuffer(bufferhandle: HSTRING_BUFFER, string: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsReplaceString(string: ::windows_sys::core::HSTRING, stringreplaced: ::windows_sys::core::HSTRING, stringreplacewith: ::windows_sys::core::HSTRING, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn WindowsStringHasEmbeddedNull(string: ::windows_sys::core::HSTRING, hasembednull: *mut super::super::Foundation::BOOL) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsSubstring(string: ::windows_sys::core::HSTRING, startindex: u32, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsSubstringWithSpecifiedLength(string: ::windows_sys::core::HSTRING, startindex: u32, length: u32, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsTrimStringEnd(string: ::windows_sys::core::HSTRING, trimstring: ::windows_sys::core::HSTRING, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
    #[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
    pub fn WindowsTrimStringStart(string: ::windows_sys::core::HSTRING, trimstring: ::windows_sys::core::HSTRING, newstring: *mut ::windows_sys::core::HSTRING) -> ::windows_sys::core::HRESULT;
}
pub type IAccountsSettingsPaneInterop = *mut ::core::ffi::c_void;
pub type IActivationFactory = *mut ::core::ffi::c_void;
pub type IAgileReference = *mut ::core::ffi::c_void;
pub type IApartmentShutdown = *mut ::core::ffi::c_void;
pub type IAppServiceConnectionExtendedExecution = *mut ::core::ffi::c_void;
pub type IBufferByteAccess = *mut ::core::ffi::c_void;
pub type ICastingController = *mut ::core::ffi::c_void;
pub type ICastingEventHandler = *mut ::core::ffi::c_void;
pub type ICastingSourceInfo = *mut ::core::ffi::c_void;
pub type ICoreInputInterop = *mut ::core::ffi::c_void;
pub type ICoreWindowAdapterInterop = *mut ::core::ffi::c_void;
pub type ICoreWindowComponentInterop = *mut ::core::ffi::c_void;
pub type ICoreWindowInterop = *mut ::core::ffi::c_void;
pub type ICorrelationVectorInformation = *mut ::core::ffi::c_void;
pub type ICorrelationVectorSource = *mut ::core::ffi::c_void;
pub type IDragDropManagerInterop = *mut ::core::ffi::c_void;
pub type IHolographicSpaceInterop = *mut ::core::ffi::c_void;
pub type IInputPaneInterop = *mut ::core::ffi::c_void;
pub type ILanguageExceptionErrorInfo = *mut ::core::ffi::c_void;
pub type ILanguageExceptionErrorInfo2 = *mut ::core::ffi::c_void;
pub type ILanguageExceptionStackBackTrace = *mut ::core::ffi::c_void;
pub type ILanguageExceptionTransform = *mut ::core::ffi::c_void;
pub type IMemoryBufferByteAccess = *mut ::core::ffi::c_void;
pub type IMessageDispatcher = *mut ::core::ffi::c_void;
pub type IPlayToManagerInterop = *mut ::core::ffi::c_void;
pub type IRestrictedErrorInfo = *mut ::core::ffi::c_void;
pub type IRoMetaDataLocator = *mut ::core::ffi::c_void;
pub type IRoSimpleMetaDataBuilder = *mut ::core::ffi::c_void;
pub type IShareWindowCommandEventArgsInterop = *mut ::core::ffi::c_void;
pub type IShareWindowCommandSourceInterop = *mut ::core::ffi::c_void;
pub type ISpatialInteractionManagerInterop = *mut ::core::ffi::c_void;
pub type ISystemMediaTransportControlsInterop = *mut ::core::ffi::c_void;
pub type IUIViewSettingsInterop = *mut ::core::ffi::c_void;
pub type IUserActivityInterop = *mut ::core::ffi::c_void;
pub type IUserActivityRequestManagerInterop = *mut ::core::ffi::c_void;
pub type IUserActivitySourceHostInterop = *mut ::core::ffi::c_void;
pub type IUserConsentVerifierInterop = *mut ::core::ffi::c_void;
pub type IWeakReference = *mut ::core::ffi::c_void;
pub type IWeakReferenceSource = *mut ::core::ffi::c_void;
pub type IWebAuthenticationCoreManagerInterop = *mut ::core::ffi::c_void;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CastingSourceInfo_Property_CastingTypes: &str = "CastingTypes";
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CastingSourceInfo_Property_PreferredSourceUriScheme: &str = "PreferredSourceUriScheme";
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CastingSourceInfo_Property_ProtectedMedia: &str = "ProtectedMedia";
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const MAX_ERROR_MESSAGE_CHARS: u32 = 512u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type ACTIVATIONTYPE = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const ACTIVATIONTYPE_UNCATEGORIZED: ACTIVATIONTYPE = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const ACTIVATIONTYPE_FROM_MONIKER: ACTIVATIONTYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const ACTIVATIONTYPE_FROM_DATA: ACTIVATIONTYPE = 2i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const ACTIVATIONTYPE_FROM_STORAGE: ACTIVATIONTYPE = 4i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const ACTIVATIONTYPE_FROM_STREAM: ACTIVATIONTYPE = 8i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const ACTIVATIONTYPE_FROM_FILE: ACTIVATIONTYPE = 16i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type AgileReferenceOptions = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const AGILEREFERENCE_DEFAULT: AgileReferenceOptions = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const AGILEREFERENCE_DELAYEDMARSHAL: AgileReferenceOptions = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type BSOS_OPTIONS = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const BSOS_DEFAULT: BSOS_OPTIONS = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const BSOS_PREFERDESTINATIONSTREAM: BSOS_OPTIONS = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type CASTING_CONNECTION_ERROR_STATUS = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_SUCCEEDED: CASTING_CONNECTION_ERROR_STATUS = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_DEVICE_DID_NOT_RESPOND: CASTING_CONNECTION_ERROR_STATUS = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_DEVICE_ERROR: CASTING_CONNECTION_ERROR_STATUS = 2i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_DEVICE_LOCKED: CASTING_CONNECTION_ERROR_STATUS = 3i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_PROTECTED_PLAYBACK_FAILED: CASTING_CONNECTION_ERROR_STATUS = 4i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_INVALID_CASTING_SOURCE: CASTING_CONNECTION_ERROR_STATUS = 5i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_ERROR_STATUS_UNKNOWN: CASTING_CONNECTION_ERROR_STATUS = 6i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type CASTING_CONNECTION_STATE = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_STATE_DISCONNECTED: CASTING_CONNECTION_STATE = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_STATE_CONNECTED: CASTING_CONNECTION_STATE = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_STATE_RENDERING: CASTING_CONNECTION_STATE = 2i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_STATE_DISCONNECTING: CASTING_CONNECTION_STATE = 3i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const CASTING_CONNECTION_STATE_CONNECTING: CASTING_CONNECTION_STATE = 4i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type DISPATCHERQUEUE_THREAD_APARTMENTTYPE = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const DQTAT_COM_NONE: DISPATCHERQUEUE_THREAD_APARTMENTTYPE = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const DQTAT_COM_ASTA: DISPATCHERQUEUE_THREAD_APARTMENTTYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const DQTAT_COM_STA: DISPATCHERQUEUE_THREAD_APARTMENTTYPE = 2i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type DISPATCHERQUEUE_THREAD_TYPE = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const DQTYPE_THREAD_DEDICATED: DISPATCHERQUEUE_THREAD_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const DQTYPE_THREAD_CURRENT: DISPATCHERQUEUE_THREAD_TYPE = 2i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type RO_ERROR_REPORTING_FLAGS = u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_ERROR_REPORTING_NONE: RO_ERROR_REPORTING_FLAGS = 0u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_ERROR_REPORTING_SUPPRESSEXCEPTIONS: RO_ERROR_REPORTING_FLAGS = 1u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_ERROR_REPORTING_FORCEEXCEPTIONS: RO_ERROR_REPORTING_FLAGS = 2u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_ERROR_REPORTING_USESETERRORINFO: RO_ERROR_REPORTING_FLAGS = 4u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_ERROR_REPORTING_SUPPRESSSETERRORINFO: RO_ERROR_REPORTING_FLAGS = 8u32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type RO_INIT_TYPE = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_INIT_SINGLETHREADED: RO_INIT_TYPE = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const RO_INIT_MULTITHREADED: RO_INIT_TYPE = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type TrustLevel = i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const BaseTrust: TrustLevel = 0i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const PartialTrust: TrustLevel = 1i32;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub const FullTrust: TrustLevel = 2i32;
pub type APARTMENT_SHUTDOWN_REGISTRATION_COOKIE = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub struct DispatcherQueueOptions {
    pub dwSize: u32,
    pub threadType: DISPATCHERQUEUE_THREAD_TYPE,
    pub apartmentType: DISPATCHERQUEUE_THREAD_APARTMENTTYPE,
}
impl ::core::marker::Copy for DispatcherQueueOptions {}
impl ::core::clone::Clone for DispatcherQueueOptions {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub struct EventRegistrationToken {
    pub value: i64,
}
impl ::core::marker::Copy for EventRegistrationToken {}
impl ::core::clone::Clone for EventRegistrationToken {
    fn clone(&self) -> Self {
        *self
    }
}
pub type HSTRING_BUFFER = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub struct HSTRING_HEADER {
    pub flags: u32,
    pub length: u32,
    pub padding1: u32,
    pub padding2: u32,
    pub data: isize,
}
impl ::core::marker::Copy for HSTRING_HEADER {}
impl ::core::clone::Clone for HSTRING_HEADER {
    fn clone(&self) -> Self {
        *self
    }
}
pub type ROPARAMIIDHANDLE = isize;
#[repr(C)]
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub struct ServerInformation {
    pub dwServerPid: u32,
    pub dwServerTid: u32,
    pub ui64ServerAddress: u64,
}
impl ::core::marker::Copy for ServerInformation {}
impl ::core::clone::Clone for ServerInformation {
    fn clone(&self) -> Self {
        *self
    }
}
#[repr(C)]
pub struct _RO_REGISTRATION_COOKIE(pub u8);
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type PINSPECT_HSTRING_CALLBACK = ::core::option::Option<unsafe extern "system" fn(context: *const ::core::ffi::c_void, readaddress: usize, length: u32, buffer: *mut u8) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type PINSPECT_HSTRING_CALLBACK2 = ::core::option::Option<unsafe extern "system" fn(context: *const ::core::ffi::c_void, readaddress: u64, length: u32, buffer: *mut u8) -> ::windows_sys::core::HRESULT>;
#[doc = "*Required features: `\"Win32_System_WinRT\"`*"]
pub type PINSPECT_MEMORY_CALLBACK = ::core::option::Option<unsafe extern "system" fn(context: *const ::core::ffi::c_void, readaddress: usize, length: u32, buffer: *mut u8) -> ::windows_sys::core::HRESULT>;
