#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Win32_Data_HtmlHelp")]
pub mod HtmlHelp;
#[cfg(feature = "Win32_Data_RightsManagement")]
pub mod RightsManagement;
#[cfg(feature = "Win32_Data_Xml")]
pub mod Xml;
