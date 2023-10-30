#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: 'Win32_System_SetupAndMigration', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn OOBEComplete(isoobecomplete: *mut super::super::Foundation::BOOL) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: 'Win32_System_SetupAndMigration', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn RegisterWaitUntilOOBECompleted(oobecompletedcallback: OOBE_COMPLETED_CALLBACK, callbackcontext: *const ::core::ffi::c_void, waithandle: *mut *mut ::core::ffi::c_void) -> super::super::Foundation::BOOL;
    #[doc = "*Required features: 'Win32_System_SetupAndMigration', 'Win32_Foundation'*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn UnregisterWaitUntilOOBECompleted(waithandle: *const ::core::ffi::c_void) -> super::super::Foundation::BOOL;
}
#[doc = "*Required features: 'Win32_System_SetupAndMigration'*"]
pub type OOBE_COMPLETED_CALLBACK = ::core::option::Option<unsafe extern "system" fn(callbackcontext: *const ::core::ffi::c_void)>;
