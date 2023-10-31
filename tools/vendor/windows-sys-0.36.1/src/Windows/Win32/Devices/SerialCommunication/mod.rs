#[link(name = "windows")]
extern "system" {
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
    pub fn ComDBClaimNextFreePort(hcomdb: HCOMDB, comnumber: *mut u32) -> i32;
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`, `\"Win32_Foundation\"`*"]
    #[cfg(feature = "Win32_Foundation")]
    pub fn ComDBClaimPort(hcomdb: HCOMDB, comnumber: u32, forceclaim: super::super::Foundation::BOOL, forced: *mut super::super::Foundation::BOOL) -> i32;
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
    pub fn ComDBClose(hcomdb: HCOMDB) -> i32;
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
    pub fn ComDBGetCurrentPortUsage(hcomdb: HCOMDB, buffer: *mut u8, buffersize: u32, reporttype: u32, maxportsreported: *mut u32) -> i32;
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
    pub fn ComDBOpen(phcomdb: *mut isize) -> i32;
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
    pub fn ComDBReleasePort(hcomdb: HCOMDB, comnumber: u32) -> i32;
    #[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
    pub fn ComDBResizeDatabase(hcomdb: HCOMDB, newsize: u32) -> i32;
}
#[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
pub const CDB_REPORT_BITS: u32 = 0u32;
#[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
pub const CDB_REPORT_BYTES: u32 = 1u32;
#[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
pub const COMDB_MAX_PORTS_ARBITRATED: u32 = 4096u32;
#[doc = "*Required features: `\"Win32_Devices_SerialCommunication\"`*"]
pub const COMDB_MIN_PORTS_ARBITRATED: u32 = 256u32;
pub type HCOMDB = isize;
