#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Win32_Storage_Packaging_Appx")]
pub mod Appx;
#[cfg(feature = "Win32_Storage_Packaging_Opc")]
pub mod Opc;
