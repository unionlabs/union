#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Security_Authentication_Identity")]
pub mod Identity;
#[cfg(feature = "Security_Authentication_OnlineId")]
pub mod OnlineId;
#[cfg(feature = "Security_Authentication_Web")]
pub mod Web;
