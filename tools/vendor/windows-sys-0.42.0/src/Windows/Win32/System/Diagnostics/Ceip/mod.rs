#[cfg_attr(windows, link(name = "windows"))]
extern "system" {
    #[doc = "*Required features: `\"Win32_System_Diagnostics_Ceip\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn CeipIsOptedIn() -> super::super::super::Foundation::BOOL;
}
