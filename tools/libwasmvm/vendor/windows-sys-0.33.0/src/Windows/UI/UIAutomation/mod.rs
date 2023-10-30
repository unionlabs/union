#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "UI_UIAutomation_Core")]
pub mod Core;
pub type AutomationConnection = *mut ::core::ffi::c_void;
pub type AutomationConnectionBoundObject = *mut ::core::ffi::c_void;
pub type AutomationElement = *mut ::core::ffi::c_void;
pub type AutomationTextRange = *mut ::core::ffi::c_void;
