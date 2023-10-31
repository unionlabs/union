#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[doc = "*Required features: 'Win32_System_WinRT_Shell'*"]
pub type CreateProcessMethod = i32;
#[doc = "*Required features: 'Win32_System_WinRT_Shell'*"]
pub const CpCreateProcess: CreateProcessMethod = 0i32;
#[doc = "*Required features: 'Win32_System_WinRT_Shell'*"]
pub const CpCreateProcessAsUser: CreateProcessMethod = 1i32;
#[doc = "*Required features: 'Win32_System_WinRT_Shell'*"]
pub const CpAicLaunchAdminProcess: CreateProcessMethod = 2i32;
pub type IDDEInitializer = *mut ::core::ffi::c_void;
