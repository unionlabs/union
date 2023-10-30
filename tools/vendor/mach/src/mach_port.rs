//! This module corresponds to `mach/mach_port.h`

use kern_return::kern_return_t;
use mach_types::ipc_space_t;
use message::mach_msg_type_name_t;
use port::{mach_port_name_t, mach_port_right_t, mach_port_t};

extern "C" {
    pub fn mach_port_allocate(
        task: ipc_space_t,
        right: mach_port_right_t,
        name: *mut mach_port_name_t,
    ) -> kern_return_t;
    pub fn mach_port_destroy(task: ipc_space_t, name: mach_port_name_t) -> kern_return_t;
    pub fn mach_port_deallocate(task: ipc_space_t, name: mach_port_name_t) -> kern_return_t;
    pub fn mach_port_insert_right(
        task: ipc_space_t,
        name: mach_port_name_t,
        poly: mach_port_t,
        polyPoly: mach_msg_type_name_t,
    ) -> kern_return_t;
    pub fn mach_port_extract_right(
        task: ipc_space_t,
        name: mach_port_name_t,
        msgt_name: mach_msg_type_name_t,
        poly: *mut mach_port_t,
        polyPoly: *mut mach_msg_type_name_t,
    ) -> kern_return_t;
}
