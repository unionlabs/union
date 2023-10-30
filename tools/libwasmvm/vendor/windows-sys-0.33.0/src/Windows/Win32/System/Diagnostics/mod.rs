#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clashing_extern_declarations, clippy::all)]
#[cfg(feature = "Win32_System_Diagnostics_Ceip")]
pub mod Ceip;
#[cfg(feature = "Win32_System_Diagnostics_Debug")]
pub mod Debug;
#[cfg(feature = "Win32_System_Diagnostics_Etw")]
pub mod Etw;
#[cfg(feature = "Win32_System_Diagnostics_ProcessSnapshotting")]
pub mod ProcessSnapshotting;
#[cfg(feature = "Win32_System_Diagnostics_ToolHelp")]
pub mod ToolHelp;
