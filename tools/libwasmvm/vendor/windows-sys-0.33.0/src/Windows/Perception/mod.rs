#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Perception_Automation")]
pub mod Automation;
#[cfg(feature = "Perception_People")]
pub mod People;
#[cfg(feature = "Perception_Spatial")]
pub mod Spatial;
pub type PerceptionTimestamp = *mut ::core::ffi::c_void;
