/*!
Learn more about Rust for Windows here: <https://github.com/microsoft/windows-rs>
*/

#![no_std]
#![doc(html_no_source)]

extern crate self as windows_sys;
mod Windows;
pub mod core;
pub use Windows::*;
