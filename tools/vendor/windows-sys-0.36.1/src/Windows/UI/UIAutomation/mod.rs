#[cfg(feature = "UI_UIAutomation_Core")]
pub mod Core;
pub type AutomationConnection = *mut ::core::ffi::c_void;
pub type AutomationConnectionBoundObject = *mut ::core::ffi::c_void;
pub type AutomationElement = *mut ::core::ffi::c_void;
pub type AutomationTextRange = *mut ::core::ffi::c_void;
