pub use base::{CFAllocatorRef, CFIndex, CFTypeID};
use runloop::CFRunLoopSourceRef;
use std::os::raw::c_void;

#[repr(C)]
pub struct __CFMachPort(c_void);
pub type CFMachPortRef = *const __CFMachPort;

extern "C" {
    /*
     * CFMachPort.h
     */
    pub fn CFMachPortCreateRunLoopSource(
        allocator: CFAllocatorRef,
        port: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    
    pub fn CFMachPortGetTypeID() -> CFTypeID;
}
