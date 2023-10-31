#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Data_Html")]
pub mod Html;
#[cfg(feature = "Data_Json")]
pub mod Json;
#[cfg(feature = "Data_Pdf")]
pub mod Pdf;
#[cfg(feature = "Data_Text")]
pub mod Text;
#[cfg(feature = "Data_Xml")]
pub mod Xml;
